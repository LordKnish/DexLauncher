// Main installer orchestration
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use serde::{Deserialize, Serialize};

use crate::core::{Downloader, Extractor, GitHubApi, GameLauncher, Verifier};
use crate::db::DbState;
use crate::error::{LauncherError, Result};
use crate::utils::{ProgressTracker, get_dir_size};
use crate::platform::{create_desktop_shortcut, create_start_menu_shortcut, remove_desktop_shortcut, remove_start_menu_shortcut};

/// Installation progress event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub operation_id: String,
    pub phase: String,
    pub percentage: f64,
    pub message: String,
}

/// Installation complete event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallComplete {
    pub installation_id: i64,
    pub game_id: String,
    pub version: String,
    pub install_path: String,
}

/// Installation error event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallError {
    pub operation_id: String,
    pub error: String,
    pub can_retry: bool,
}

/// Main installer
pub struct Installer {
    db: Arc<DbState>,
    app_handle: AppHandle,
}

impl Installer {
    /// Create a new installer
    pub fn new(db: Arc<DbState>, app_handle: AppHandle) -> Self {
        Self { db, app_handle }
    }

    /// Install or update a game using HTTP downloads
    pub async fn install_game(
        &self,
        operation_id: String,
        game_id: String,
        version: String,
        install_path: PathBuf,
        create_start_menu: bool,
        create_desktop: bool,
    ) -> Result<i64> {
        tracing::info!("Starting installation: {} v{} to {}", game_id, version, install_path.display());
        
        // Emit starting event
        self.emit_progress(&operation_id, "starting", 0.0, "Preparing installation...")?;

        // Create GitHub API client
        let github = GitHubApi::new("infinitefusion", "infinitefusion-e18")?;
        
        // Get download URL for the releases branch archive
        let download_url = github.get_branch_archive_url("releases");
        tracing::info!("Download URL: {}", download_url);

        // Create downloader
        let downloader = Downloader::new()?;

        // Create progress tracker for download
        let download_progress = Arc::new(ProgressTracker::new(operation_id.clone(), 0));
        let download_progress_clone = Arc::clone(&download_progress);

        // Spawn progress emitter task
        let app_handle = self.app_handle.clone();
        let op_id = operation_id.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                
                let current = download_progress_clone.current();
                let total = download_progress_clone.total();
                
                if total > 0 {
                    let percentage = ((current as f64 / total as f64) * 70.0).round(); // Download is 0-70%
                    let mb_current = current as f64 / 1_048_576.0;
                    let mb_total = total as f64 / 1_048_576.0;
                    
                    let _ = app_handle.emit("install-progress", InstallProgress {
                        operation_id: op_id.clone(),
                        phase: "downloading".to_string(),
                        percentage,
                        message: format!("Downloading: {:.1} MB / {:.1} MB ({}%)",
                            mb_current, mb_total, percentage as i32),
                    });
                }
                
                if current >= total && total > 0 {
                    break;
                }
            }
        });

        // Download the archive
        let archive_path = install_path.with_extension("zip");
        self.emit_progress(&operation_id, "downloading", 5.0, "Downloading game archive from GitHub...")?;
        
        tracing::info!("Downloading archive to: {}", archive_path.display());
        
        downloader
            .download_with_resume(&download_url, &archive_path, Some(download_progress))
            .await?;
        
        tracing::info!("Archive download complete, preparing extraction...");

        // Emit extraction phase
        self.emit_progress(&operation_id, "extracting", 70.0, "Extracting files...")?;

        // Create extraction progress tracker
        let extract_progress = Arc::new(ProgressTracker::new(operation_id.clone(), 0));
        let extract_progress_clone = Arc::clone(&extract_progress);

        // Spawn extraction progress emitter
        let app_handle = self.app_handle.clone();
        let op_id = operation_id.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                
                let current = extract_progress_clone.current();
                let total = extract_progress_clone.total();
                
                if total > 0 {
                    let percentage = (70.0 + (current as f64 / total as f64) * 20.0).round(); // Extraction is 70-90%
                    
                    let _ = app_handle.emit("install-progress", InstallProgress {
                        operation_id: op_id.clone(),
                        phase: "extracting".to_string(),
                        percentage,
                        message: format!("Extracting: {} / {} files ({}%)",
                            current, total, percentage as i32),
                    });
                }
                
                if current >= total && total > 0 {
                    break;
                }
            }
        });

        // Extract the archive
        Extractor::extract_zip(&archive_path, &install_path, Some(extract_progress)).await?;

        // Delete the archive file
        tokio::fs::remove_file(&archive_path).await?;

        // Emit verification phase
        self.emit_progress(&operation_id, "verifying", 90.0, "Verifying installation...")?;

        // Verify installation
        if !Verifier::verify_basic(&install_path)? {
            return Err(LauncherError::Installation(
                "Installation verification failed - InfiniteFusion.exe not found".to_string()
            ));
        }

        // Calculate installation size
        self.emit_progress(&operation_id, "finalizing", 95.0, "Calculating installation size...")?;
        let size_bytes = get_dir_size(&install_path).ok();

        // Save to database
        tracing::info!("Saving installation to database...");
        let installation_id = self.db.create_installation(
            &game_id,
            &version,
            &install_path.to_string_lossy(),
            size_bytes.map(|s| s as i64),
        )?;

        tracing::info!("Installation complete! ID: {}", installation_id);

        // Create shortcuts if requested
        let game_exe = install_path.join("InfiniteFusion.exe");
        
        if create_desktop {
            tracing::info!("Creating desktop shortcut...");
            if let Err(e) = create_desktop_shortcut("Pokemon Infinite Fusion", &game_exe, Some(&install_path)) {
                tracing::warn!("Failed to create desktop shortcut: {}", e);
            }
        }

        if create_start_menu {
            tracing::info!("Creating Start Menu shortcut...");
            if let Err(e) = create_start_menu_shortcut("Pokemon Infinite Fusion", &game_exe, Some(&install_path)) {
                tracing::warn!("Failed to create Start Menu shortcut: {}", e);
            }
        }

        // Emit completion
        self.emit_complete(installation_id, &game_id, &version, &install_path)?;

        Ok(installation_id)
    }

    /// Launch a game
    pub async fn launch_game(&self, installation_id: i64) -> Result<()> {
        tracing::info!("=== LAUNCHING GAME ===");
        tracing::info!("Installation ID: {}", installation_id);
        
        // Get installation from database
        let installation = self
            .db
            .get_installation(installation_id)?
            .ok_or_else(|| {
                tracing::error!("Installation {} not found in database", installation_id);
                LauncherError::Installation("Installation not found".to_string())
            })?;

        let install_path = PathBuf::from(&installation.install_path);
        tracing::info!("Installation path: {}", install_path.display());

        // Verify installation still exists
        if !Verifier::verify_basic(&install_path)? {
            tracing::error!("Installation verification failed - Game.exe not found");
            // Mark as invalid in database
            self.db.update_installation_validity(installation_id, false)?;
            return Err(LauncherError::Installation(
                "Installation is no longer valid. Please reinstall.".to_string()
            ));
        }

        tracing::info!("Installation verified, launching game...");

        // Launch the game (this will close the launcher)
        GameLauncher::launch(&install_path, &self.app_handle)?;

        // Update last played time
        self.db.update_last_played(installation_id)?;

        Ok(())
    }

    /// Delete an installation
    pub async fn delete_installation(&self, installation_id: i64) -> Result<()> {
        tracing::info!("=== DELETING INSTALLATION ===");
        tracing::info!("Installation ID: {}", installation_id);
        
        // Get installation from database
        let installation = self
            .db
            .get_installation(installation_id)?
            .ok_or_else(|| {
                tracing::error!("Installation {} not found", installation_id);
                LauncherError::Installation("Installation not found".to_string())
            })?;

        let install_path = PathBuf::from(&installation.install_path);
        tracing::info!("Installation path: {}", install_path.display());

        // Remove shortcuts
        tracing::info!("Removing shortcuts...");
        let _ = remove_desktop_shortcut("Pokemon Infinite Fusion");
        let _ = remove_start_menu_shortcut("Pokemon Infinite Fusion");

        // Delete files
        if install_path.exists() {
            tracing::info!("Deleting game files...");
            std::fs::remove_dir_all(&install_path)?;
            tracing::info!("Game files deleted");
        }

        // Delete from database
        self.db.delete_installation(installation_id)?;
        tracing::info!("Installation removed from database");

        Ok(())
    }

    /// Emit progress event
    fn emit_progress(
        &self,
        operation_id: &str,
        phase: &str,
        percentage: f64,
        message: &str,
    ) -> Result<()> {
        let progress = InstallProgress {
            operation_id: operation_id.to_string(),
            phase: phase.to_string(),
            percentage,
            message: message.to_string(),
        };

        self.app_handle
            .emit("install-progress", progress)
            .map_err(|e| LauncherError::General(format!("Failed to emit progress: {}", e)))?;

        Ok(())
    }

    /// Emit completion event
    fn emit_complete(
        &self,
        installation_id: i64,
        game_id: &str,
        version: &str,
        install_path: &PathBuf,
    ) -> Result<()> {
        let complete = InstallComplete {
            installation_id,
            game_id: game_id.to_string(),
            version: version.to_string(),
            install_path: install_path.to_string_lossy().to_string(),
        };

        self.app_handle
            .emit("install-complete", complete)
            .map_err(|e| LauncherError::General(format!("Failed to emit completion: {}", e)))?;

        Ok(())
    }

    /// Emit error event
    pub fn emit_error(&self, operation_id: &str, error: &str, can_retry: bool) -> Result<()> {
        let error_event = InstallError {
            operation_id: operation_id.to_string(),
            error: error.to_string(),
            can_retry,
        };

        self.app_handle
            .emit("install-error", error_event)
            .map_err(|e| LauncherError::General(format!("Failed to emit error: {}", e)))?;

        Ok(())
    }
}