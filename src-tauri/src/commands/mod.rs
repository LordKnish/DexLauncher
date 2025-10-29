use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::core::{Installer, OperationManager, Updater, Verifier, VerificationMode};
use crate::db::{DbState, Installation};
use crate::utils::{get_disk_space, DiskSpaceInfo, is_system_directory, validate_install_path};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Application state
pub struct AppState {
    pub db: Arc<DbState>,
    pub operation_manager: OperationManager,
}

/// Check system requirements (git is now required for submodule support)
#[tauri::command]
pub async fn check_system_ready() -> std::result::Result<bool, String> {
    use crate::core::github::GitInstaller;
    
    // Check if git is available
    let git_available = GitInstaller::is_git_available();
    
    if !git_available {
        return Err("Git is required but not found on this system. Please install Git and restart the launcher.".to_string());
    }
    
    tracing::info!("Git is available: {}", git_available);
    Ok(git_available)
}

/// Get all installations
#[tauri::command]
pub async fn get_installations(state: State<'_, AppState>) -> std::result::Result<Vec<Installation>, String> {
    state.db.get_installations().map_err(|e| e.to_string())
}

/// Get installation by ID
#[tauri::command]
pub async fn get_installation(
    state: State<'_, AppState>,
    id: i64,
) -> std::result::Result<Option<Installation>, String> {
    state.db.get_installation(id).map_err(|e| e.to_string())
}

/// Select installation directory using native dialog
#[tauri::command]
pub async fn select_install_directory(app: AppHandle) -> std::result::Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    
    let result = app.dialog()
        .file()
        .set_title("Select Installation Directory")
        .blocking_pick_folder();
    
    if let Some(path) = result {
        let path_buf = PathBuf::from(path.to_string());
        
        // Validate the path
        if let Err(e) = validate_install_path(&path_buf) {
            return Err(format!("Invalid installation directory: {}", e));
        }
        
        // Check if it's a system directory
        if is_system_directory(&path_buf) {
            return Err("Cannot install in system directory".to_string());
        }
        
        Ok(Some(path.to_string()))
    } else {
        Ok(None)
    }
}

/// Check disk space for a path
#[tauri::command]
pub async fn check_disk_space(path: String) -> std::result::Result<DiskSpaceInfo, String> {
    let path_buf = PathBuf::from(path);
    get_disk_space(&path_buf).map_err(|e| e.to_string())
}

/// Install a game
#[tauri::command]
pub async fn install_game(
    state: State<'_, AppState>,
    app: AppHandle,
    operation_id: String,
    game_id: String,
    version: String,
    install_path: String,
    create_start_menu: bool,
    create_desktop: bool,
    add_to_steam: bool,
) -> std::result::Result<i64, String> {
    let installer = Installer::new(Arc::clone(&state.db), app);
    let path = PathBuf::from(install_path);
    
    installer
        .install_game(operation_id, game_id, version, path, create_start_menu, create_desktop, add_to_steam)
        .await
        .map_err(|e| e.to_string())
}

/// Add game to Steam (manual retry)
#[tauri::command]
pub async fn add_to_steam(
    state: State<'_, AppState>,
    app: AppHandle,
    install_path: String,
) -> std::result::Result<String, String> {
    let installer = Installer::new(Arc::clone(&state.db), app);
    let path = PathBuf::from(install_path);
    
    installer
        .add_to_steam(path)
        .await
        .map_err(|e| e.to_string())
}

/// Check if game is in Steam shortcuts
#[tauri::command]
pub async fn check_steam_shortcut(
    install_path: String,
) -> std::result::Result<bool, String> {
    use crate::core::SteamIntegration;
    
    let steam = SteamIntegration::new().map_err(|e| e.to_string())?;
    let path = PathBuf::from(&install_path);
    let exe_path = path.join("InfiniteFusion.exe");
    
    steam
        .is_in_steam("Pokémon Infinite Fusion", &exe_path)
        .map_err(|e| e.to_string())
}

/// Remove game from Steam shortcuts
#[tauri::command]
pub async fn remove_from_steam(
    install_path: String,
) -> std::result::Result<String, String> {
    use crate::core::SteamIntegration;
    
    let steam = SteamIntegration::new().map_err(|e| e.to_string())?;
    let path = PathBuf::from(&install_path);
    let exe_path = path.join("InfiniteFusion.exe");
    
    let result = steam
        .remove_from_steam("Pokémon Infinite Fusion", &exe_path)
        .map_err(|e| e.to_string())?;
    
    Ok(result.message)
}

/// Check if Steam is installed and running
#[tauri::command]
pub async fn check_steam_status() -> std::result::Result<crate::core::steam::SteamRunningState, String> {
    use crate::core::SteamIntegration;
    
    let steam = SteamIntegration::new().map_err(|e| e.to_string())?;
    Ok(steam.get_steam_state())
}

/// Close Steam (for Steam integration)
#[tauri::command]
pub async fn close_steam() -> std::result::Result<bool, String> {
    use crate::core::SteamIntegration;
    
    SteamIntegration::close_steam().map_err(|e| e.to_string())
}

/// Launch a game
#[tauri::command]
pub async fn launch_game(
    state: State<'_, AppState>,
    app: AppHandle,
    installation_id: i64,
) -> std::result::Result<(), String> {
    let installer = Installer::new(Arc::clone(&state.db), app);
    
    installer
        .launch_game(installation_id)
        .await
        .map_err(|e| e.to_string())
}

/// Delete an installation
#[tauri::command]
pub async fn delete_installation(
    state: State<'_, AppState>,
    app: AppHandle,
    installation_id: i64,
) -> std::result::Result<(), String> {
    let installer = Installer::new(Arc::clone(&state.db), app);
    
    installer
        .delete_installation(installation_id)
        .await
        .map_err(|e| e.to_string())
}

/// Get a setting value
#[tauri::command]
pub async fn get_setting(
    state: State<'_, AppState>,
    key: String,
) -> std::result::Result<Option<String>, String> {
    state.db.get_setting(&key).map_err(|e| e.to_string())
}

/// Set a setting value
#[tauri::command]
pub async fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> std::result::Result<(), String> {
    state.db.set_setting(&key, &value).map_err(|e| e.to_string())
}

/// Get all settings
#[tauri::command]
pub async fn get_all_settings(
    state: State<'_, AppState>,
) -> std::result::Result<std::collections::HashMap<String, String>, String> {
    state.db.get_all_settings().map_err(|e| e.to_string())
}

/// Cancel an ongoing operation
#[tauri::command]
pub async fn cancel_operation(
    state: State<'_, AppState>,
    operation_id: String,
) -> std::result::Result<(), String> {
    tracing::info!("Cancelling operation: {}", operation_id);
    let cancelled = state.operation_manager.cancel_operation(&operation_id).await;
    if cancelled {
        Ok(())
    } else {
        Err(format!("Operation {} not found or already completed", operation_id))
    }
}

/// Get repository size from GitHub API
#[tauri::command]
pub async fn get_repository_size() -> std::result::Result<u64, String> {
    use crate::core::GitHubApi;
    
    let github = GitHubApi::new("infinitefusion", "infinitefusion-e18")
        .map_err(|e| e.to_string())?;
    
    github.get_repository_info()
        .await
        .map_err(|e| e.to_string())
}

/// Check for updates for an installation
#[tauri::command]
pub async fn check_for_updates(
    state: State<'_, AppState>,
    app: AppHandle,
    installation_id: i64,
) -> std::result::Result<crate::core::updater::UpdateInfo, String> {
    let updater = Updater::new(Arc::clone(&state.db), app);
    updater
        .check_for_updates(installation_id)
        .await
        .map_err(|e| e.to_string())
}

/// Update a game installation
#[tauri::command]
pub async fn update_game(
    state: State<'_, AppState>,
    app: AppHandle,
    operation_id: String,
    installation_id: i64,
) -> std::result::Result<(), String> {
    let updater = Updater::new(Arc::clone(&state.db), app);
    let cancel_token = state.operation_manager.register_operation(operation_id.clone()).await;
    
    let result = updater
        .update_game(operation_id.clone(), installation_id, cancel_token)
        .await;
    
    state.operation_manager.unregister_operation(&operation_id).await;
    result.map_err(|e| e.to_string())
}

/// Verify installation integrity
#[tauri::command]
pub async fn verify_installation(
    state: State<'_, AppState>,
    installation_id: i64,
    full_check: bool,
) -> std::result::Result<crate::core::verifier::VerificationReport, String> {
    let verifier = Verifier::new(Arc::clone(&state.db));
    let mode = if full_check {
        VerificationMode::Full
    } else {
        VerificationMode::Quick
    };
    
    verifier
        .verify_installation(installation_id, mode)
        .await
        .map_err(|e| e.to_string())
}

/// Repair corrupted installation files
#[tauri::command]
pub async fn repair_installation(
    state: State<'_, AppState>,
    installation_id: i64,
    corrupted_files: Vec<String>,
) -> std::result::Result<usize, String> {
    let verifier = Verifier::new(Arc::clone(&state.db));
    verifier
        .repair_installation(installation_id, corrupted_files)
        .await
        .map_err(|e| e.to_string())
}

/// Expand environment variables in a path string
#[tauri::command]
pub fn expand_path(path: String) -> std::result::Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        // Expand Windows environment variables like %APPDATA%
        let expanded = shellexpand::env(&path)
            .map_err(|e| format!("Failed to expand path: {}", e))?;
        Ok(expanded.to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // On Unix-like systems, expand ~ and environment variables
        let expanded = shellexpand::full(&path)
            .map_err(|e| format!("Failed to expand path: {}", e))?;
        Ok(expanded.to_string())
    }
}

/// Open a directory in the system's file explorer
#[tauri::command]
pub async fn open_directory(path: String) -> std::result::Result<(), String> {
    // First expand any environment variables in the path
    let expanded_path = {
        #[cfg(target_os = "windows")]
        {
            shellexpand::env(&path)
                .map_err(|e| format!("Failed to expand path: {}", e))?
                .to_string()
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            shellexpand::full(&path)
                .map_err(|e| format!("Failed to expand path: {}", e))?
                .to_string()
        }
    };
    
    let path_buf = PathBuf::from(&expanded_path);
    
    // Check if directory exists, if not create it
    if !path_buf.exists() {
        std::fs::create_dir_all(&path_buf)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    if !path_buf.is_dir() {
        return Err(format!("Path is not a directory: {}", expanded_path));
    }
    
    #[cfg(target_os = "windows")]
    {
        // Use explorer.exe on Windows
        // CREATE_NO_WINDOW flag to prevent console window from appearing
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        
        std::process::Command::new("explorer.exe")
            .arg(&expanded_path)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&expanded_path)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&expanded_path)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }
    
    Ok(())
}
