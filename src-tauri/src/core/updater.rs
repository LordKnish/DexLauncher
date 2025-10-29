use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

use crate::error::{LauncherError, Result};
use crate::db::DbState;
use crate::core::{GitHubApi, GitHubRelease, Verifier};

/// Update information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateInfo {
    pub installation_id: i64,
    pub current_version: String,
    pub latest_version: String,
    pub changelog: String,
    pub size_bytes: u64,
    pub update_available: bool,
}

/// Update progress
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateProgress {
    pub operation_id: String,
    pub phase: String,
    pub percentage: f64,
    pub message: String,
}

/// Game updater
pub struct Updater {
    db: Arc<DbState>,
    app_handle: AppHandle,
}

impl Updater {
    /// Create a new updater
    pub fn new(db: Arc<DbState>, app_handle: AppHandle) -> Self {
        Self { db, app_handle }
    }

    /// Check for updates for an installation
    pub async fn check_for_updates(
        &self,
        installation_id: i64,
    ) -> Result<UpdateInfo> {
        tracing::info!("Checking for updates for installation {}", installation_id);

        // Get installation info
        let installation = self.db.get_installation(installation_id)?
            .ok_or_else(|| LauncherError::NotFound(format!("Installation {} not found", installation_id)))?;

        let install_path = PathBuf::from(&installation.install_path);

        // Get current version from git
        let current_version = self.get_git_version(&install_path)?;

        // Get latest release from GitHub
        let github = GitHubApi::new("infinitefusion", "infinitefusion-e18")?;
        let latest_release = github.get_latest_release().await?;

        // Get repository size
        let size_bytes = github.get_repository_info().await?;

        let update_available = current_version != latest_release.tag_name;

        Ok(UpdateInfo {
            installation_id,
            current_version,
            latest_version: latest_release.tag_name.clone(),
            changelog: latest_release.body,
            size_bytes,
            update_available,
        })
    }

    /// Get current git version
    fn get_git_version(&self, install_path: &Path) -> Result<String> {
        let output = std::process::Command::new("git")
            .current_dir(install_path)
            .args(&["describe", "--tags", "--always"])
            .output()
            .map_err(|e| LauncherError::Git(format!("Failed to get git version: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LauncherError::Git(format!("Failed to get version: {}", stderr)));
        }

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    }

    /// Update an installation in-place
    pub async fn update_game(
        &self,
        operation_id: String,
        installation_id: i64,
        cancel_token: CancellationToken,
    ) -> Result<()> {
        tracing::info!("Starting update for installation {}", installation_id);

        // Get installation info
        let installation = self.db.get_installation(installation_id)?
            .ok_or_else(|| LauncherError::NotFound(format!("Installation {} not found", installation_id)))?;

        let install_path = PathBuf::from(&installation.install_path);

        // Verify installation before update
        self.emit_progress(&operation_id, "verifying", 5.0, "Verifying installation before update...");
        
        let verifier = Verifier::new(Arc::clone(&self.db));
        let verification = verifier.verify_installation(installation_id, crate::core::verifier::VerificationMode::Quick).await?;
        
        if !verification.is_valid {
            return Err(LauncherError::Verification(
                format!("Installation is corrupted. Please repair before updating.")
            ));
        }

        // Check for cancellation
        if cancel_token.is_cancelled() {
            return Err(LauncherError::Cancelled);
        }

        // Create backup
        self.emit_progress(&operation_id, "backup", 10.0, "Creating backup...");
        self.create_backup(&install_path)?;

        // Check for cancellation
        if cancel_token.is_cancelled() {
            self.restore_backup(&install_path)?;
            return Err(LauncherError::Cancelled);
        }

        // Fetch updates
        self.emit_progress(&operation_id, "fetching", 30.0, "Fetching updates from GitHub...");
        
        let fetch_result = std::process::Command::new("git")
            .current_dir(&install_path)
            .args(&["fetch", "origin", "releases"])
            .output()
            .map_err(|e| LauncherError::Git(format!("Failed to fetch updates: {}", e)))?;

        if !fetch_result.status.success() {
            let stderr = String::from_utf8_lossy(&fetch_result.stderr);
            self.restore_backup(&install_path)?;
            return Err(LauncherError::Git(format!("Failed to fetch: {}", stderr)));
        }

        // Check for cancellation
        if cancel_token.is_cancelled() {
            self.restore_backup(&install_path)?;
            return Err(LauncherError::Cancelled);
        }

        // Update submodules
        self.emit_progress(&operation_id, "updating", 50.0, "Updating submodules...");
        
        let submodule_result = std::process::Command::new("git")
            .current_dir(&install_path)
            .args(&["submodule", "update", "--init", "--recursive"])
            .output()
            .map_err(|e| LauncherError::Git(format!("Failed to update submodules: {}", e)))?;

        if !submodule_result.status.success() {
            let stderr = String::from_utf8_lossy(&submodule_result.stderr);
            tracing::warn!("Submodule update warning: {}", stderr);
        }

        // Check for cancellation
        if cancel_token.is_cancelled() {
            self.restore_backup(&install_path)?;
            return Err(LauncherError::Cancelled);
        }

        // Reset to latest
        self.emit_progress(&operation_id, "applying", 70.0, "Applying updates...");
        
        let reset_result = std::process::Command::new("git")
            .current_dir(&install_path)
            .args(&["reset", "--hard", "origin/releases"])
            .output()
            .map_err(|e| LauncherError::Git(format!("Failed to reset: {}", e)))?;

        if !reset_result.status.success() {
            let stderr = String::from_utf8_lossy(&reset_result.stderr);
            self.restore_backup(&install_path)?;
            return Err(LauncherError::Git(format!("Failed to apply updates: {}", stderr)));
        }

        // Check for cancellation
        if cancel_token.is_cancelled() {
            self.restore_backup(&install_path)?;
            return Err(LauncherError::Cancelled);
        }

        // Recalculate file hashes
        self.emit_progress(&operation_id, "verifying", 85.0, "Recalculating file hashes...");
        
        let exclude_patterns = vec![".git", ".backup"];
        let file_hashes = Verifier::calculate_directory_hashes(&install_path, &exclude_patterns)?;
        verifier.store_file_hashes(installation_id, &file_hashes)?;

        // Get new version
        let new_version = self.get_git_version(&install_path)?;

        // Update database
        self.emit_progress(&operation_id, "finalizing", 95.0, "Finalizing update...");
        
        self.db.update_installation_version(installation_id, &new_version)?;

        // Clean up backup
        self.remove_backup(&install_path)?;

        self.emit_progress(&operation_id, "complete", 100.0, "Update complete!");
        
        tracing::info!("Update complete for installation {}", installation_id);
        Ok(())
    }

    /// Create backup of critical files
    fn create_backup(&self, install_path: &Path) -> Result<()> {
        let backup_path = install_path.join(".backup");
        
        // Remove old backup if exists
        if backup_path.exists() {
            std::fs::remove_dir_all(&backup_path)?;
        }

        std::fs::create_dir_all(&backup_path)?;

        // Backup .git directory
        let git_path = install_path.join(".git");
        if git_path.exists() {
            let backup_git = backup_path.join(".git");
            self.copy_dir_recursive(&git_path, &backup_git)?;
        }

        tracing::info!("Backup created at: {}", backup_path.display());
        Ok(())
    }

    /// Restore from backup
    fn restore_backup(&self, install_path: &Path) -> Result<()> {
        let backup_path = install_path.join(".backup");
        
        if !backup_path.exists() {
            return Err(LauncherError::FileSystem("Backup not found".to_string()));
        }

        tracing::info!("Restoring from backup...");

        // Restore .git directory
        let backup_git = backup_path.join(".git");
        if backup_git.exists() {
            let git_path = install_path.join(".git");
            if git_path.exists() {
                std::fs::remove_dir_all(&git_path)?;
            }
            self.copy_dir_recursive(&backup_git, &git_path)?;
        }

        // Reset to HEAD
        let reset_result = std::process::Command::new("git")
            .current_dir(install_path)
            .args(&["reset", "--hard", "HEAD"])
            .output()
            .map_err(|e| LauncherError::Git(format!("Failed to reset: {}", e)))?;

        if !reset_result.status.success() {
            let stderr = String::from_utf8_lossy(&reset_result.stderr);
            tracing::warn!("Reset warning: {}", stderr);
        }

        tracing::info!("Backup restored successfully");
        Ok(())
    }

    /// Remove backup
    fn remove_backup(&self, install_path: &Path) -> Result<()> {
        let backup_path = install_path.join(".backup");
        
        if backup_path.exists() {
            std::fs::remove_dir_all(&backup_path)?;
            tracing::info!("Backup removed");
        }

        Ok(())
    }

    /// Copy directory recursively
    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        std::fs::create_dir_all(dst)?;

        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let dst_path = dst.join(&file_name);

            if path.is_dir() {
                self.copy_dir_recursive(&path, &dst_path)?;
            } else {
                std::fs::copy(&path, &dst_path)?;
            }
        }

        Ok(())
    }

    /// Emit progress event
    fn emit_progress(&self, operation_id: &str, phase: &str, percentage: f64, message: &str) {
        let progress = UpdateProgress {
            operation_id: operation_id.to_string(),
            phase: phase.to_string(),
            percentage,
            message: message.to_string(),
        };

        if let Err(e) = self.app_handle.emit("update-progress", progress) {
            tracing::warn!("Failed to emit update progress: {}", e);
        }
    }
}
