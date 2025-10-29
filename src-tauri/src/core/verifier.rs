use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Read, BufReader};
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::error::{LauncherError, Result};
use crate::db::DbState;

/// Verification mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationMode {
    /// Quick check - only verify file existence and sizes
    Quick,
    /// Full check - recalculate SHA256 hashes
    Full,
}

/// Verification report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub installation_id: i64,
    pub mode: String,
    pub files_checked: usize,
    pub files_valid: usize,
    pub files_missing: usize,
    pub files_modified: usize,
    pub corrupted_files: Vec<String>,
    pub is_valid: bool,
}

/// File hash information
#[derive(Debug, Clone)]
pub struct FileHash {
    pub relative_path: String,
    pub hash: String,
    pub size: u64,
}

/// Installation verifier
pub struct Verifier {
    db: std::sync::Arc<DbState>,
}

impl Verifier {
    /// Create a new verifier
    pub fn new(db: std::sync::Arc<DbState>) -> Self {
        Self { db }
    }

    /// Calculate SHA256 hash of a file
    pub fn calculate_file_hash(file_path: &Path) -> Result<String> {
        let file = File::open(file_path).map_err(|e| {
            LauncherError::FileSystem(format!("Failed to open file for hashing: {}", e))
        })?;

        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = reader.read(&mut buffer).map_err(|e| {
                LauncherError::FileSystem(format!("Failed to read file for hashing: {}", e))
            })?;

            if bytes_read == 0 {
                break;
            }

            hasher.update(&buffer[..bytes_read]);
        }

        let hash = hasher.finalize();
        Ok(hex::encode(hash))
    }

    /// Calculate hashes for all files in a directory
    pub fn calculate_directory_hashes(
        install_path: &Path,
        exclude_patterns: &[&str],
    ) -> Result<Vec<FileHash>> {
        let mut file_hashes = Vec::new();

        tracing::info!("Calculating hashes for directory: {}", install_path.display());

        for entry in WalkDir::new(install_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                // Skip excluded directories
                let path = e.path();
                let path_str = path.to_string_lossy();
                
                !exclude_patterns.iter().any(|pattern| {
                    path_str.contains(pattern)
                })
            })
        {
            let entry = entry.map_err(|e| {
                LauncherError::FileSystem(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();

            // Only process files
            if !path.is_file() {
                continue;
            }

            // Get relative path
            let relative_path = path
                .strip_prefix(install_path)
                .map_err(|e| {
                    LauncherError::FileSystem(format!("Failed to get relative path: {}", e))
                })?
                .to_string_lossy()
                .to_string();

            // Get file size
            let metadata = path.metadata().map_err(|e| {
                LauncherError::FileSystem(format!("Failed to get file metadata: {}", e))
            })?;
            let size = metadata.len();

            // Calculate hash
            let hash = Self::calculate_file_hash(path)?;

            file_hashes.push(FileHash {
                relative_path,
                hash,
                size,
            });
        }

        tracing::info!("Calculated hashes for {} files", file_hashes.len());
        Ok(file_hashes)
    }

    /// Store file hashes in database
    pub fn store_file_hashes(
        &self,
        installation_id: i64,
        file_hashes: &[FileHash],
    ) -> Result<()> {
        tracing::info!("Storing {} file hashes for installation {}", file_hashes.len(), installation_id);

        // Clear existing hashes for this installation
        self.db.clear_installation_files(installation_id)?;

        // Insert new hashes
        for file_hash in file_hashes {
            self.db.add_installation_file(
                installation_id,
                &file_hash.relative_path,
                &file_hash.hash,
                file_hash.size as i64,
            )?;
        }

        Ok(())
    }

    /// Verify installation integrity
    pub async fn verify_installation(
        &self,
        installation_id: i64,
        mode: VerificationMode,
    ) -> Result<VerificationReport> {
        tracing::info!("Verifying installation {} (mode: {:?})", installation_id, mode);

        // Get installation info
        let installation = self.db.get_installation(installation_id)?
            .ok_or_else(|| LauncherError::NotFound(format!("Installation {} not found", installation_id)))?;

        let install_path = PathBuf::from(&installation.install_path);

        // Get stored file hashes from database
        let stored_files = self.db.get_installation_files(installation_id)?;

        if stored_files.is_empty() {
            tracing::warn!("No file hashes stored for installation {}", installation_id);
            return Ok(VerificationReport {
                installation_id,
                mode: format!("{:?}", mode),
                files_checked: 0,
                files_valid: 0,
                files_missing: 0,
                files_modified: 0,
                corrupted_files: Vec::new(),
                is_valid: false,
            });
        }

        let mut files_checked = 0;
        let mut files_valid = 0;
        let mut files_missing = 0;
        let mut files_modified = 0;
        let mut corrupted_files = Vec::new();

        for stored_file in stored_files {
            files_checked += 1;
            let file_path = install_path.join(&stored_file.file_path);

            // Check if file exists
            if !file_path.exists() {
                files_missing += 1;
                corrupted_files.push(stored_file.file_path.clone());
                tracing::warn!("Missing file: {}", stored_file.file_path);
                continue;
            }

            // Check file size
            let metadata = file_path.metadata().map_err(|e| {
                LauncherError::FileSystem(format!("Failed to get file metadata: {}", e))
            })?;

            if metadata.len() != stored_file.size_bytes as u64 {
                files_modified += 1;
                corrupted_files.push(stored_file.file_path.clone());
                tracing::warn!("Size mismatch for file: {} (expected: {}, actual: {})",
                    stored_file.file_path, stored_file.size_bytes, metadata.len());
                continue;
            }

            // For full verification, check hash
            if mode == VerificationMode::Full {
                let actual_hash = Self::calculate_file_hash(&file_path)?;
                if actual_hash != stored_file.hash {
                    files_modified += 1;
                    corrupted_files.push(stored_file.file_path.clone());
                    tracing::warn!("Hash mismatch for file: {}", stored_file.file_path);
                    continue;
                }
            }

            files_valid += 1;
        }

        let is_valid = corrupted_files.is_empty();

        // Update installation validity in database
        self.db.update_installation_validity(installation_id, is_valid)?;

        let report = VerificationReport {
            installation_id,
            mode: format!("{:?}", mode),
            files_checked,
            files_valid,
            files_missing,
            files_modified,
            corrupted_files,
            is_valid,
        };

        tracing::info!("Verification complete: {} valid, {} missing, {} modified",
            files_valid, files_missing, files_modified);

        Ok(report)
    }

    /// Repair corrupted files by re-extracting from git repository
    pub async fn repair_installation(
        &self,
        installation_id: i64,
        corrupted_files: Vec<String>,
    ) -> Result<usize> {
        let file_count = corrupted_files.len();
        tracing::info!("Repairing {} corrupted files for installation {}", 
            file_count, installation_id);

        let installation = self.db.get_installation(installation_id)?
            .ok_or_else(|| LauncherError::NotFound(format!("Installation {} not found", installation_id)))?;

        let install_path = PathBuf::from(&installation.install_path);

        // For git-based installations, we can use git checkout to restore files
        let mut repaired = 0;

        for file_path in &corrupted_files {
            let full_path = install_path.join(&file_path);
            
            tracing::info!("Attempting to repair: {}", file_path);

            // Use git checkout to restore the file
            let output = std::process::Command::new("git")
                .current_dir(&install_path)
                .args(&["checkout", "HEAD", "--", &file_path])
                .output()
                .map_err(|e| {
                    LauncherError::Git(format!("Failed to run git checkout: {}", e))
                })?;

            if output.status.success() {
                // Recalculate hash for repaired file
                if full_path.exists() {
                    let new_hash = Self::calculate_file_hash(&full_path)?;
                    let metadata = full_path.metadata()?;
                    
                    // Update hash in database
                    self.db.update_installation_file_hash(
                        installation_id,
                        &file_path,
                        &new_hash,
                        metadata.len(),
                    )?;
                    
                    repaired += 1;
                    tracing::info!("Successfully repaired: {}", file_path);
                } else {
                    tracing::warn!("File still missing after repair: {}", file_path);
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!("Failed to repair {}: {}", file_path, stderr);
            }
        }

        tracing::info!("Repaired {} out of {} files", repaired, corrupted_files.len());
        Ok(repaired)
    }

    /// Basic verification - check if installation exists and has required files
    pub fn verify_basic(install_path: &PathBuf) -> Result<bool> {
        // Check if directory exists
        if !install_path.exists() {
            return Ok(false);
        }

        // Check for InfiniteFusion.exe (main executable)
        let game_exe = install_path.join("InfiniteFusion.exe");
        let exists = game_exe.exists();
        
        if !exists {
            tracing::warn!("InfiniteFusion.exe not found at: {}", game_exe.display());
        }
        
        Ok(exists)
    }
}

/// Verify installation (standalone function for backward compatibility)
pub fn verify_installation(install_path: &PathBuf) -> Result<bool> {
    Verifier::verify_basic(install_path)
}
