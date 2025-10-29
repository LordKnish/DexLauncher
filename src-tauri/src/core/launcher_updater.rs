use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;
use tokio::time::sleep;

use crate::error::{LauncherError, Result};

/// Update information for the launcher itself
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LauncherUpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: String,
    pub release_date: String,
    pub download_url: String,
    pub update_available: bool,
}

/// Update download progress
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LauncherUpdateProgress {
    pub downloaded: u64,
    pub total: u64,
    pub percentage: f64,
    pub speed_bps: u64,
}

/// Update status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "status", content = "data")]
pub enum LauncherUpdateStatus {
    Checking,
    Available(LauncherUpdateInfo),
    Downloading(LauncherUpdateProgress),
    Installing,
    ReadyToRestart,
    UpToDate,
    Error(String),
}

/// Launcher updater for self-updates
pub struct LauncherUpdater {
    app_handle: AppHandle,
}

impl LauncherUpdater {
    /// Create a new launcher updater
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    /// Check for launcher updates
    pub async fn check_for_update(&self) -> Result<Option<LauncherUpdateInfo>> {
        tracing::info!("Checking for launcher updates...");
        
        self.emit_status(LauncherUpdateStatus::Checking);

        // Get current version from tauri config
        let current_version = self.app_handle.package_info().version.to_string();

        // Check for updates using Tauri's updater
        let updater = self.app_handle.updater_builder().build()
            .map_err(|e| LauncherError::Update(format!("Failed to build updater: {}", e)))?;

        match updater.check().await {
            Ok(Some(update)) => {
                let update_info = LauncherUpdateInfo {
                    current_version: current_version.clone(),
                    latest_version: update.version.clone(),
                    release_notes: update.body.clone().unwrap_or_default(),
                    release_date: update.date.map(|d| d.to_string()).unwrap_or_default(),
                    download_url: update.download_url.to_string(),
                    update_available: true,
                };

                tracing::info!(
                    "Update available: {} -> {}",
                    current_version,
                    update.version
                );

                self.emit_status(LauncherUpdateStatus::Available(update_info.clone()));
                Ok(Some(update_info))
            }
            Ok(None) => {
                tracing::info!("Launcher is up to date (v{})", current_version);
                self.emit_status(LauncherUpdateStatus::UpToDate);
                Ok(None)
            }
            Err(e) => {
                let error_msg = format!("Failed to check for updates: {}", e);
                tracing::error!("{}", error_msg);
                self.emit_status(LauncherUpdateStatus::Error(error_msg.clone()));
                Err(LauncherError::Update(error_msg))
            }
        }
    }

    /// Download and install launcher update
    pub async fn download_and_install(&self) -> Result<()> {
        tracing::info!("Starting launcher update download and installation...");

        let updater = self.app_handle.updater_builder().build()
            .map_err(|e| LauncherError::Update(format!("Failed to build updater: {}", e)))?;

        let update = match updater.check().await {
            Ok(Some(update)) => update,
            Ok(None) => {
                return Err(LauncherError::Update("No update available".to_string()));
            }
            Err(e) => {
                return Err(LauncherError::Update(format!("Failed to check for updates: {}", e)));
            }
        };

        tracing::info!("Downloading update version {}", update.version);

        // Download with progress tracking
        let mut downloaded: u64 = 0;
        let mut last_emit = std::time::Instant::now();

        let download_result = update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length as u64;
                    let total_size = content_length.unwrap_or(0);

                    // Emit progress every 100ms to avoid overwhelming the frontend
                    if last_emit.elapsed() >= Duration::from_millis(100) {
                        let percentage = if total_size > 0 {
                            (downloaded as f64 / total_size as f64) * 100.0
                        } else {
                            0.0
                        };

                        let progress = LauncherUpdateProgress {
                            downloaded,
                            total: total_size,
                            percentage,
                            speed_bps: 0, // Speed calculation would require more tracking
                        };

                        if let Err(e) = self.app_handle.emit("launcher-update-status", 
                            LauncherUpdateStatus::Downloading(progress)) {
                            tracing::warn!("Failed to emit download progress: {}", e);
                        }

                        last_emit = std::time::Instant::now();
                    }
                },
                || {
                    tracing::info!("Update download completed, installing...");
                    if let Err(e) = self.app_handle.emit("launcher-update-status", 
                        LauncherUpdateStatus::Installing) {
                        tracing::warn!("Failed to emit installing status: {}", e);
                    }
                },
            )
            .await;

        match download_result {
            Ok(_) => {
                tracing::info!("Update installed successfully, ready to restart");
                self.emit_status(LauncherUpdateStatus::ReadyToRestart);
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to download and install update: {}", e);
                tracing::error!("{}", error_msg);
                self.emit_status(LauncherUpdateStatus::Error(error_msg.clone()));
                Err(LauncherError::Update(error_msg))
            }
        }
    }

    /// Start background update checker (runs every 6 hours)
    pub fn start_background_checker(app_handle: AppHandle) {
        tauri::async_runtime::spawn(async move {
            let updater = LauncherUpdater::new(app_handle.clone());
            
            // Initial check after 30 seconds
            sleep(Duration::from_secs(30)).await;
            
            loop {
                if let Err(e) = updater.check_for_update().await {
                    tracing::warn!("Background update check failed: {}", e);
                }

                // Check every 6 hours
                sleep(Duration::from_secs(6 * 60 * 60)).await;
            }
        });

        tracing::info!("Background update checker started");
    }

    /// Emit update status to frontend
    fn emit_status(&self, status: LauncherUpdateStatus) {
        if let Err(e) = self.app_handle.emit("launcher-update-status", status) {
            tracing::warn!("Failed to emit launcher update status: {}", e);
        }
    }
}

/// Check for launcher updates (Tauri command)
#[tauri::command]
pub async fn check_launcher_update(
    app: AppHandle,
) -> std::result::Result<Option<LauncherUpdateInfo>, String> {
    let updater = LauncherUpdater::new(app);
    updater.check_for_update().await.map_err(|e| e.to_string())
}

/// Download and install launcher update (Tauri command)
#[tauri::command]
pub async fn install_launcher_update(
    app: AppHandle,
) -> std::result::Result<(), String> {
    let updater = LauncherUpdater::new(app);
    updater.download_and_install().await.map_err(|e| e.to_string())
}

/// Restart the application after update (Tauri command)
#[tauri::command]
pub async fn restart_launcher(
    app: AppHandle,
) -> std::result::Result<(), String> {
    tracing::info!("Restarting launcher to apply update...");
    app.restart();
    Ok(())
}
