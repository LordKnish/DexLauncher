// Steam integration for adding games as non-Steam shortcuts
use std::path::PathBuf;
use std::fs;
use std::process::Command;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use steam_shortcuts_util::{Shortcut, parse_shortcuts, shortcuts_to_bytes};
use sysinfo::System;
use crate::error::{LauncherError, Result};

#[cfg(windows)]
use winreg::enums::HKEY_CURRENT_USER;
#[cfg(windows)]
use winreg::RegKey;

/// Steam integration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamIntegrationResult {
    pub success: bool,
    pub message: String,
    pub steam_was_running: bool,
    pub shortcuts_added: usize,
}

/// Steam running state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamRunningState {
    pub is_running: bool,
    pub steam_path: Option<PathBuf>,
}

/// Steam integration manager
pub struct SteamIntegration {
    steam_path: Option<PathBuf>,
}

impl SteamIntegration {
    /// Create a new Steam integration manager
    pub fn new() -> Result<Self> {
        let steam_path = Self::detect_steam_path()?;
        Ok(Self { steam_path })
    }

    /// Detect Steam installation path from registry (Windows only)
    #[cfg(windows)]
    fn detect_steam_path() -> Result<Option<PathBuf>> {
        tracing::info!("Detecting Steam installation path from registry...");
        
        match RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey("Software\\Valve\\Steam")
        {
            Ok(key) => {
                match key.get_value::<String, _>("SteamPath") {
                    Ok(path) => {
                        let steam_path = PathBuf::from(path);
                        tracing::info!("Found Steam at: {}", steam_path.display());
                        Ok(Some(steam_path))
                    }
                    Err(e) => {
                        tracing::warn!("Steam registry key exists but SteamPath not found: {}", e);
                        Ok(None)
                    }
                }
            }
            Err(e) => {
                tracing::info!("Steam not found in registry: {}", e);
                Ok(None)
            }
        }
    }

    /// Detect Steam path on non-Windows platforms
    #[cfg(not(windows))]
    fn detect_steam_path() -> Result<Option<PathBuf>> {
        // For Linux/Mac, check common Steam locations
        let possible_paths = vec![
            PathBuf::from(shellexpand::tilde("~/.steam/steam")),
            PathBuf::from(shellexpand::tilde("~/.local/share/Steam")),
            PathBuf::from("/Applications/Steam.app/Contents/MacOS"),
        ];

        for path in possible_paths {
            if path.exists() {
                tracing::info!("Found Steam at: {}", path.display());
                return Ok(Some(path));
            }
        }

        tracing::info!("Steam not found on this system");
        Ok(None)
    }

    /// Check if Steam is currently running
    pub fn is_steam_running() -> bool {
        let system = System::new_all();
        
        let is_running = system.processes_by_name("steam.exe".as_ref())
            .next()
            .is_some();
        
        tracing::info!("Steam running: {}", is_running);
        is_running
    }

    /// Get Steam running state
    pub fn get_steam_state(&self) -> SteamRunningState {
        SteamRunningState {
            is_running: Self::is_steam_running(),
            steam_path: self.steam_path.clone(),
        }
    }

    /// Close Steam gracefully
    #[cfg(windows)]
    pub fn close_steam() -> Result<bool> {
        tracing::info!("Attempting to close Steam...");
        
        let output = Command::new("taskkill")
            .args(&["/IM", "steam.exe", "/F"])
            .output()
            .map_err(|e| LauncherError::Steam(format!("Failed to execute taskkill: {}", e)))?;

        if !output.status.success() {
            tracing::warn!("taskkill failed: {}", String::from_utf8_lossy(&output.stderr));
            return Ok(false);
        }

        // Wait up to 10 seconds for Steam to close
        for i in 0..10 {
            std::thread::sleep(Duration::from_secs(1));
            if !Self::is_steam_running() {
                tracing::info!("Steam closed successfully after {} seconds", i + 1);
                return Ok(true);
            }
        }

        tracing::warn!("Steam did not close within 10 seconds");
        Ok(false)
    }

    /// Close Steam on non-Windows platforms
    #[cfg(not(windows))]
    pub fn close_steam() -> Result<bool> {
        tracing::info!("Attempting to close Steam...");
        
        let output = Command::new("pkill")
            .arg("steam")
            .output()
            .map_err(|e| LauncherError::Steam(format!("Failed to execute pkill: {}", e)))?;

        // Wait up to 10 seconds for Steam to close
        for i in 0..10 {
            std::thread::sleep(Duration::from_secs(1));
            if !Self::is_steam_running() {
                tracing::info!("Steam closed successfully after {} seconds", i + 1);
                return Ok(true);
            }
        }

        tracing::warn!("Steam did not close within 10 seconds");
        Ok(false)
    }

    /// Restart Steam
    #[cfg(windows)]
    pub fn restart_steam(&self) -> Result<()> {
        if let Some(steam_path) = &self.steam_path {
            let steam_exe = steam_path.join("steam.exe");
            
            tracing::info!("Restarting Steam from: {}", steam_exe.display());
            
            Command::new(steam_exe)
                .arg("-silent")
                .spawn()
                .map_err(|e| LauncherError::Steam(format!("Failed to restart Steam: {}", e)))?;
            
            tracing::info!("Steam restart initiated");
            Ok(())
        } else {
            Err(LauncherError::Steam("Steam path not found".to_string()))
        }
    }

    /// Restart Steam on non-Windows platforms
    #[cfg(not(windows))]
    pub fn restart_steam(&self) -> Result<()> {
        tracing::info!("Restarting Steam...");
        
        Command::new("steam")
            .arg("-silent")
            .spawn()
            .map_err(|e| LauncherError::Steam(format!("Failed to restart Steam: {}", e)))?;
        
        tracing::info!("Steam restart initiated");
        Ok(())
    }

    /// Get all shortcuts.vdf file paths
    fn get_shortcuts_paths(&self) -> Result<Vec<PathBuf>> {
        let steam_path = self.steam_path.as_ref()
            .ok_or_else(|| LauncherError::Steam("Steam not installed".to_string()))?;

        let userdata_path = steam_path.join("userdata");
        
        if !userdata_path.exists() {
            tracing::warn!("Steam userdata directory not found: {}", userdata_path.display());
            return Ok(vec![]);
        }

        let mut shortcuts_paths = Vec::new();

        for entry in fs::read_dir(&userdata_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let shortcuts_file = path.join("config").join("shortcuts.vdf");
                shortcuts_paths.push(shortcuts_file);
            }
        }

        tracing::info!("Found {} potential shortcuts.vdf locations", shortcuts_paths.len());
        Ok(shortcuts_paths)
    }

    /// Calculate Steam shortcut app ID using CRC32
    fn calculate_app_id(exe_path: &str, app_name: &str) -> u32 {
        let mut hasher = crc32fast::Hasher::new();
        
        let input = format!("{}{}", exe_path, app_name);
        hasher.update(input.as_bytes());
        
        let crc = hasher.finalize();
        // Steam uses: (crc | 0x80000000) as the app ID
        crc | 0x80000000
    }

    /// Add game to Steam shortcuts
    pub fn add_to_steam(
        &self,
        game_name: &str,
        exe_path: &PathBuf,
        start_dir: &PathBuf,
        icon_path: Option<&PathBuf>,
    ) -> Result<SteamIntegrationResult> {
        tracing::info!("=== ADDING TO STEAM ===");
        tracing::info!("Game: {}", game_name);
        tracing::info!("Exe: {}", exe_path.display());
        tracing::info!("Start dir: {}", start_dir.display());

        if self.steam_path.is_none() {
            return Err(LauncherError::Steam("Steam not installed".to_string()));
        }

        let shortcuts_paths = self.get_shortcuts_paths()?;
        
        if shortcuts_paths.is_empty() {
            return Err(LauncherError::Steam("No Steam user accounts found".to_string()));
        }

        let mut shortcuts_added = 0;
        let exe_str = exe_path.to_string_lossy().to_string();
        let start_dir_str = start_dir.to_string_lossy().to_string();
        let icon_str = icon_path.map(|p| p.to_string_lossy().to_string()).unwrap_or_default();

        for shortcuts_path in shortcuts_paths {
            match self.add_shortcut_to_file(
                &shortcuts_path,
                game_name,
                &exe_str,
                &start_dir_str,
                &icon_str,
            ) {
                Ok(()) => {
                    shortcuts_added += 1;
                    tracing::info!("✓ Added shortcut to: {}", shortcuts_path.display());
                }
                Err(e) => {
                    tracing::warn!("Failed to add shortcut to {}: {}", shortcuts_path.display(), e);
                }
            }
        }

        if shortcuts_added == 0 {
            return Err(LauncherError::Steam("Failed to add shortcuts to any Steam account".to_string()));
        }

        Ok(SteamIntegrationResult {
            success: true,
            message: format!("Added to {} Steam account(s)", shortcuts_added),
            steam_was_running: false,
            shortcuts_added,
        })
    }

    /// Add shortcut to a specific shortcuts.vdf file
    fn add_shortcut_to_file(
        &self,
        shortcuts_path: &PathBuf,
        game_name: &str,
        exe_path: &str,
        start_dir: &str,
        icon_path: &str,
    ) -> Result<()> {
        // Backup existing file
        if shortcuts_path.exists() {
            let backup_path = shortcuts_path.with_extension("vdf.bak");
            fs::copy(shortcuts_path, &backup_path)?;
            tracing::info!("Created backup: {}", backup_path.display());
        } else {
            // Create parent directories if they don't exist
            if let Some(parent) = shortcuts_path.parent() {
                fs::create_dir_all(parent)?;
            }
        }

        // Load existing shortcuts or create new
        let data = if shortcuts_path.exists() {
            fs::read(shortcuts_path)?
        } else {
            Vec::new()
        };
        
        let mut shortcuts = if !data.is_empty() {
            parse_shortcuts(&data)
                .map_err(|e| LauncherError::Steam(format!("Failed to parse shortcuts.vdf: {}", e)))?
        } else {
            Vec::new()
        };

        // Check if shortcut already exists (by app name and exe)
        let existing_index = shortcuts.iter().position(|s| {
            s.app_name == game_name && s.exe == exe_path
        });

        if let Some(index) = existing_index {
            // Update existing shortcut
            tracing::info!("Updating existing shortcut at index {}", index);
            let shortcut = &mut shortcuts[index];
            shortcut.start_dir = start_dir;
            shortcut.icon = icon_path;
            shortcut.allow_overlay = true;
            shortcut.is_hidden = false;
            shortcut.open_vr = 0;
            shortcut.tags = vec!["DexLauncher"];
        } else {
            // Add new shortcut
            tracing::info!("Adding new shortcut");
            let shortcut = Shortcut {
                app_id: 0,
                app_name: game_name,
                exe: exe_path,
                start_dir,
                icon: icon_path,
                shortcut_path: "",
                launch_options: "",
                is_hidden: false,
                allow_desktop_config: true,
                allow_overlay: true,
                open_vr: 0,
                dev_kit: 0,
                dev_kit_game_id: "",
                dev_kit_overrite_app_id: 0,
                last_play_time: 0,
                tags: vec!["DexLauncher"],
                order: "",
            };
            shortcuts.push(shortcut);
        }

        // Serialize and write
        let data = shortcuts_to_bytes(&shortcuts);
        
        fs::write(shortcuts_path, data)?;
        tracing::info!("✓ Wrote shortcuts.vdf");

        // Verify by reloading
        let verify_data = fs::read(shortcuts_path)?;
        let verify_shortcuts = parse_shortcuts(&verify_data)
            .map_err(|e| LauncherError::Steam(format!("Verification failed: {}", e)))?;
        
        let found = verify_shortcuts.iter().any(|s| {
            s.app_name == game_name && s.exe == exe_path
        });

        if !found {
            return Err(LauncherError::Steam("Verification failed: shortcut not found after write".to_string()));
        }

        tracing::info!("✓ Verification passed");
        Ok(())
    }

    /// Check if game is already in Steam shortcuts
    pub fn is_in_steam(&self, game_name: &str, exe_path: &PathBuf) -> Result<bool> {
        tracing::info!("Checking if game is in Steam shortcuts...");
        
        if self.steam_path.is_none() {
            return Ok(false);
        }

        let shortcuts_paths = self.get_shortcuts_paths()?;
        let exe_str = exe_path.to_string_lossy().to_string();

        for shortcuts_path in shortcuts_paths {
            if !shortcuts_path.exists() {
                continue;
            }

            let data = fs::read(&shortcuts_path)?;
            if data.is_empty() {
                continue;
            }

            match parse_shortcuts(&data) {
                Ok(shortcuts) => {
                    let found = shortcuts.iter().any(|s| {
                        s.app_name == game_name && s.exe == exe_str
                    });
                    
                    if found {
                        tracing::info!("Game found in Steam shortcuts");
                        return Ok(true);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to parse shortcuts.vdf: {}", e);
                    continue;
                }
            }
        }

        tracing::info!("Game not found in Steam shortcuts");
        Ok(false)
    }

    /// Remove game from Steam shortcuts
    pub fn remove_from_steam(
        &self,
        game_name: &str,
        exe_path: &PathBuf,
    ) -> Result<SteamIntegrationResult> {
        tracing::info!("=== REMOVING FROM STEAM ===");
        tracing::info!("Game: {}", game_name);
        tracing::info!("Exe: {}", exe_path.display());

        if self.steam_path.is_none() {
            return Err(LauncherError::Steam("Steam not installed".to_string()));
        }

        let shortcuts_paths = self.get_shortcuts_paths()?;
        
        if shortcuts_paths.is_empty() {
            return Err(LauncherError::Steam("No Steam user accounts found".to_string()));
        }

        let mut shortcuts_removed = 0;
        let exe_str = exe_path.to_string_lossy().to_string();

        for shortcuts_path in shortcuts_paths {
            match self.remove_shortcut_from_file(&shortcuts_path, game_name, &exe_str) {
                Ok(removed) => {
                    if removed {
                        shortcuts_removed += 1;
                        tracing::info!("✓ Removed shortcut from: {}", shortcuts_path.display());
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to remove shortcut from {}: {}", shortcuts_path.display(), e);
                }
            }
        }

        if shortcuts_removed == 0 {
            return Ok(SteamIntegrationResult {
                success: false,
                message: "Game not found in Steam shortcuts".to_string(),
                steam_was_running: false,
                shortcuts_added: 0,
            });
        }

        Ok(SteamIntegrationResult {
            success: true,
            message: format!("Removed from {} Steam account(s)", shortcuts_removed),
            steam_was_running: false,
            shortcuts_added: 0,
        })
    }

    /// Remove shortcut from a specific shortcuts.vdf file
    fn remove_shortcut_from_file(
        &self,
        shortcuts_path: &PathBuf,
        game_name: &str,
        exe_path: &str,
    ) -> Result<bool> {
        if !shortcuts_path.exists() {
            return Ok(false);
        }

        // Backup existing file
        let backup_path = shortcuts_path.with_extension("vdf.bak");
        fs::copy(shortcuts_path, &backup_path)?;
        tracing::info!("Created backup: {}", backup_path.display());

        // Load existing shortcuts
        let data = fs::read(shortcuts_path)?;
        if data.is_empty() {
            return Ok(false);
        }
        
        let mut shortcuts = parse_shortcuts(&data)
            .map_err(|e| LauncherError::Steam(format!("Failed to parse shortcuts.vdf: {}", e)))?;

        // Find and remove the shortcut
        let initial_len = shortcuts.len();
        shortcuts.retain(|s| {
            !(s.app_name == game_name && s.exe == exe_path)
        });

        let removed = shortcuts.len() < initial_len;
        
        if !removed {
            tracing::info!("Shortcut not found in this file");
            return Ok(false);
        }

        // Serialize and write
        let data = shortcuts_to_bytes(&shortcuts);
        fs::write(shortcuts_path, data)?;
        tracing::info!("✓ Wrote updated shortcuts.vdf");

        // Verify by reloading
        let verify_data = fs::read(shortcuts_path)?;
        let verify_shortcuts = parse_shortcuts(&verify_data)
            .map_err(|e| LauncherError::Steam(format!("Verification failed: {}", e)))?;
        
        let still_exists = verify_shortcuts.iter().any(|s| {
            s.app_name == game_name && s.exe == exe_path
        });

        if still_exists {
            return Err(LauncherError::Steam("Verification failed: shortcut still exists after removal".to_string()));
        }

        tracing::info!("✓ Verification passed");
        Ok(true)
    }

    /// Install Steam grid art (optional)
    pub fn install_grid_art(
        &self,
        game_name: &str,
        exe_path: &PathBuf,
        grid_image: Option<&[u8]>,
        hero_image: Option<&[u8]>,
        logo_image: Option<&[u8]>,
    ) -> Result<()> {
        tracing::info!("Installing Steam grid art...");

        let steam_path = self.steam_path.as_ref()
            .ok_or_else(|| LauncherError::Steam("Steam not installed".to_string()))?;

        let exe_str = exe_path.to_string_lossy().to_string();
        let app_id = Self::calculate_app_id(&exe_str, game_name);
        
        tracing::info!("Calculated Steam app ID: {}", app_id);

        let userdata_path = steam_path.join("userdata");
        
        for entry in fs::read_dir(&userdata_path)? {
            let entry = entry?;
            let grid_dir = entry.path().join("config").join("grid");
            
            if let Err(e) = fs::create_dir_all(&grid_dir) {
                tracing::warn!("Failed to create grid directory: {}", e);
                continue;
            }

            // Install grid tile (460x215)
            if let Some(data) = grid_image {
                let grid_path = grid_dir.join(format!("{}p.jpg", app_id));
                if let Err(e) = fs::write(&grid_path, data) {
                    tracing::warn!("Failed to write grid image: {}", e);
                } else {
                    tracing::info!("✓ Installed grid tile: {}", grid_path.display());
                }
            }

            // Install hero image (1920x620)
            if let Some(data) = hero_image {
                let hero_path = grid_dir.join(format!("{}_hero.jpg", app_id));
                if let Err(e) = fs::write(&hero_path, data) {
                    tracing::warn!("Failed to write hero image: {}", e);
                } else {
                    tracing::info!("✓ Installed hero image: {}", hero_path.display());
                }
            }

            // Install logo (transparent PNG)
            if let Some(data) = logo_image {
                let logo_path = grid_dir.join(format!("{}_logo.png", app_id));
                if let Err(e) = fs::write(&logo_path, data) {
                    tracing::warn!("Failed to write logo image: {}", e);
                } else {
                    tracing::info!("✓ Installed logo: {}", logo_path.display());
                }
            }
        }

        Ok(())
    }
}
