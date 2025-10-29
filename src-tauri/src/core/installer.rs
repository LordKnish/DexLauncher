// Main installer orchestration
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use serde::{Deserialize, Serialize};

use crate::core::{GitHubApi, GameLauncher, Verifier, GitInstaller, GitHubRepo, SteamIntegration};
use crate::db::DbState;
use crate::error::{LauncherError, Result};
use crate::utils::{get_dir_size, SteamArtwork};
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
        add_to_steam: bool,
    ) -> Result<i64> {
        tracing::info!("Starting installation: {} v{} to {}", game_id, version, install_path.display());
        
        // Emit starting event
        self.emit_progress(&operation_id, "starting", 0.0, "Preparing installation...")?;

        // Create GitHub API client
        let github = GitHubApi::new("infinitefusion", "infinitefusion-e18")?;
        
        // Get download URL for releases branch archive
        let download_url = github.get_branch_archive_url("releases");
        tracing::info!("Download URL: {}", download_url);

        // Fetch repository size from GitHub API
        tracing::info!("Fetching repository size from GitHub API...");
        let repo_size = github.get_repository_info().await.ok();
        if let Some(size) = repo_size {
            tracing::info!("Expected download size: {:.2} MB", size as f64 / 1_048_576.0);
        }

        // Create git installer
        let git_installer = GitInstaller::new(
            GitHubRepo::new("infinitefusion", "infinitefusion-e18", "releases"),
            self.app_handle.clone()
        );

        // Install using git (includes submodules)
        // Git installation handles everything: clone, fetch, reset, submodules
        git_installer
            .install_or_update(&install_path, &operation_id)
            .await?;
        
        tracing::info!("Git installation complete!");

        // Emit verification phase
        self.emit_progress(&operation_id, "verifying", 90.0, "Verifying installation...")?;

        // Verify installation
        if !Verifier::verify_basic(&install_path)? {
            return Err(LauncherError::Installation(
                "Installation verification failed - InfiniteFusion.exe not found".to_string()
            ));
        }

        // Calculate installation size
        let size_bytes = get_dir_size(&install_path).ok();

        // Steam Integration (95% - 100%)
        if add_to_steam {
            self.emit_progress(&operation_id, "steam", 95.0, "Adding to Steam...")?;
            
            match self.add_to_steam_internal(&install_path, &operation_id).await {
                Ok(message) => {
                    tracing::info!("✓ Steam integration successful: {}", message);
                    self.app_handle
                        .emit("steam-success", message)
                        .ok();
                }
                Err(e) => {
                    tracing::warn!("Steam integration failed (non-fatal): {}", e);
                    // Mark as pending for retry from settings
                    let _ = self.db.set_setting("steam_pending", "true");
                    self.app_handle
                        .emit("steam-skipped", format!("Install finished. Add to Steam later via Settings. Error: {}", e))
                        .ok();
                }
            }
        }

        self.emit_progress(&operation_id, "finalizing", 99.0, "Finalizing installation...")?;

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

    /// Internal Steam integration logic
    async fn add_to_steam_internal(
        &self,
        install_path: &PathBuf,
        operation_id: &str,
    ) -> Result<String> {
        tracing::info!("=== STEAM INTEGRATION ===");
        
        // Create Steam integration manager
        let steam = SteamIntegration::new()?;
        
        // Check if Steam is running
        let state = steam.get_steam_state();
        
        if !state.steam_path.is_some() {
            return Err(LauncherError::Steam("Steam not installed on this system".to_string()));
        }

        let steam_was_running = state.is_running;
        
        // If Steam is running, emit event to ask user
        if steam_was_running {
            tracing::info!("Steam is running, requesting user action...");
            
            // Emit event to frontend to show modal
            self.app_handle
                .emit("steam-running-prompt", ())
                .map_err(|e| LauncherError::Steam(format!("Failed to emit steam-running event: {}", e)))?;
            
            // Wait for user response via a channel or setting
            // For now, we'll skip if Steam is running (user can retry from settings)
            return Err(LauncherError::Steam("Steam is running. Please close Steam and try again from Settings.".to_string()));
        }

        self.emit_progress(operation_id, "steam", 96.0, "Writing Steam shortcuts...")?;

        // Add to Steam
        let game_exe = install_path.join("InfiniteFusion.exe");
        let icon_path = install_path.join("Game").join("Icon.ico");
        let icon = if icon_path.exists() {
            Some(icon_path)
        } else {
            None
        };

        let result = steam.add_to_steam(
            "Pokémon Infinite Fusion",
            &game_exe,
            install_path,
            icon.as_ref(),
        )?;

        self.emit_progress(operation_id, "steam", 98.5, "Installing Steam grid art...")?;

        // Try to install grid art (non-fatal if it fails)
        if let Err(e) = self.install_steam_grid_art(&steam, &game_exe).await {
            tracing::warn!("Failed to install Steam grid art: {}", e);
        }

        // If we closed Steam, restart it
        if steam_was_running {
            self.emit_progress(operation_id, "steam", 99.5, "Restarting Steam...")?;
            if let Err(e) = steam.restart_steam() {
                tracing::warn!("Failed to restart Steam: {}", e);
            }
        }

        Ok(result.message)
    }

    /// Install Steam grid art
    async fn install_steam_grid_art(
        &self,
        steam: &SteamIntegration,
        game_exe: &PathBuf,
    ) -> Result<()> {
        tracing::info!("Generating Steam artwork from embedded assets...");
        
        // Generate artwork from embedded assets
        let artwork = SteamArtwork::generate()?;
        
        tracing::info!("Installing Steam grid art...");
        
        // Install artwork to Steam
        steam.install_grid_art(
            "Pokémon Infinite Fusion",
            game_exe,
            Some(&artwork.grid_jpeg),
            Some(&artwork.hero_jpeg),
            Some(&artwork.logo_png),
        )?;
        
        tracing::info!("✓ Steam grid art installed successfully");
        Ok(())
    }

    /// Add game to Steam (public method for manual retry)
    pub async fn add_to_steam(
        &self,
        install_path: PathBuf,
    ) -> Result<String> {
        let operation_id = uuid::Uuid::new_v4().to_string();
        self.add_to_steam_internal(&install_path, &operation_id).await
    }
}
