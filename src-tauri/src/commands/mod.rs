use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::core::Installer;
use crate::db::{DbState, Installation};
use crate::utils::{get_disk_space, DiskSpaceInfo, is_system_directory, validate_install_path};

/// Application state
pub struct AppState {
    pub db: Arc<DbState>,
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
) -> std::result::Result<i64, String> {
    let installer = Installer::new(Arc::clone(&state.db), app);
    let path = PathBuf::from(install_path);
    
    installer
        .install_game(operation_id, game_id, version, path, create_start_menu, create_desktop)
        .await
        .map_err(|e| e.to_string())
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
    operation_id: String,
) -> std::result::Result<(), String> {
    tracing::info!("Cancelling operation: {}", operation_id);
    // TODO: Implement actual cancellation logic
    // For now, just log it - the frontend will handle UI state
    Ok(())
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
