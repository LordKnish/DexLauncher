use directories::ProjectDirs;
use std::path::PathBuf;
use crate::error::{LauncherError, Result};

/// Get the application data directory
pub fn get_app_data_dir() -> Result<PathBuf> {
    ProjectDirs::from("com", "pokemon", "fusion-launcher")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .ok_or_else(|| LauncherError::Config("Failed to determine app data directory".to_string()))
}

/// Get the database file path
pub fn get_db_path() -> Result<PathBuf> {
    let app_dir = get_app_data_dir()?;
    Ok(app_dir.join("launcher.db"))
}

/// Get the cache directory
pub fn get_cache_dir() -> Result<PathBuf> {
    let app_dir = get_app_data_dir()?;
    Ok(app_dir.join("cache"))
}

/// Get the downloads directory
pub fn get_downloads_dir() -> Result<PathBuf> {
    let app_dir = get_app_data_dir()?;
    Ok(app_dir.join("downloads"))
}

/// Validate installation path
pub fn validate_install_path(path: &PathBuf) -> Result<()> {
    // Check if path exists
    if !path.exists() {
        return Err(LauncherError::FileSystem(
            "Installation path does not exist".to_string()
        ));
    }

    // Check if it's a directory
    if !path.is_dir() {
        return Err(LauncherError::FileSystem(
            "Installation path is not a directory".to_string()
        ));
    }

    // Check write permissions by trying to create a temp file
    let test_file = path.join(".write_test");
    match std::fs::write(&test_file, b"test") {
        Ok(_) => {
            let _ = std::fs::remove_file(&test_file);
            Ok(())
        }
        Err(e) => Err(LauncherError::FileSystem(
            format!("No write permission in installation directory: {}", e)
        )),
    }
}

/// Check if path is a system directory (to prevent accidental installations)
pub fn is_system_directory(path: &PathBuf) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    
    #[cfg(target_os = "windows")]
    {
        path_str.contains("windows") ||
        path_str.contains("program files") ||
        path_str.contains("system32") ||
        path_str == "c:\\"
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        path_str == "/" ||
        path_str.starts_with("/bin") ||
        path_str.starts_with("/sbin") ||
        path_str.starts_with("/usr/bin") ||
        path_str.starts_with("/usr/sbin") ||
        path_str.starts_with("/etc") ||
        path_str.starts_with("/sys") ||
        path_str.starts_with("/proc")
    }
}

/// Sanitize filename to remove invalid characters
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => c,
        })
        .collect()
}

/// Get installation directory name for a game version
pub fn get_install_dir_name(game_id: &str, version: &str) -> String {
    format!("{}-{}", sanitize_filename(game_id), sanitize_filename(version))
}

/// Ensure directory exists, create if it doesn't
pub fn ensure_dir_exists(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Get file size
pub fn get_file_size(path: &PathBuf) -> Result<u64> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len())
}

/// Calculate directory size recursively
pub fn get_dir_size(path: &PathBuf) -> Result<u64> {
    let mut total_size = 0u64;
    
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                total_size += get_dir_size(&path)?;
            } else {
                total_size += get_file_size(&path)?;
            }
        }
    }
    
    Ok(total_size)
}