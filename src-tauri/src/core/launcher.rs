// Game launcher module
use std::path::PathBuf;
use std::process::Command;
use tauri::AppHandle;
use crate::error::{LauncherError, Result};

/// Game launcher
pub struct GameLauncher;

impl GameLauncher {
    /// Launch the game and close the launcher
    pub fn launch(install_path: &PathBuf, _app_handle: &AppHandle) -> Result<()> {
        let game_exe = install_path.join("InfiniteFusion.exe");
        
        if !game_exe.exists() {
            tracing::error!("InfiniteFusion.exe not found at: {}", game_exe.display());
            return Err(LauncherError::Installation(
                "InfiniteFusion.exe not found in installation directory".to_string()
            ));
        }

        tracing::info!("=== LAUNCHING GAME ===");
        tracing::info!("Executable: {}", game_exe.display());
        tracing::info!("Working directory: {}", install_path.display());

        #[cfg(target_os = "windows")]
        {
            // On Windows, launch directly
            Command::new(&game_exe)
                .current_dir(install_path)
                .spawn()
                .map_err(|e| {
                    tracing::error!("Failed to launch game: {}", e);
                    LauncherError::General(format!("Failed to launch game: {}", e))
                })?;
            
            tracing::info!("Game launched successfully!");
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On Unix systems, try to launch with Wine
            Command::new("wine")
                .arg(&game_exe)
                .current_dir(install_path)
                .spawn()
                .map_err(|e| {
                    tracing::error!("Failed to launch game with Wine: {}", e);
                    LauncherError::General(format!("Failed to launch game with Wine: {}", e))
                })?;
            
            tracing::info!("Game launched successfully with Wine!");
        }

        // Close the launcher after game starts
        tracing::info!("Closing launcher...");
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(1));
            std::process::exit(0);
        });

        Ok(())
    }

    /// Check if Wine is available (for Unix systems)
    #[cfg(not(target_os = "windows"))]
    pub fn is_wine_available() -> bool {
        Command::new("wine")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[cfg(target_os = "windows")]
    pub fn is_wine_available() -> bool {
        true // Not needed on Windows
    }
}