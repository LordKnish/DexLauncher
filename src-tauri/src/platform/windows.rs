use std::path::PathBuf;
use std::fs;
use crate::error::{LauncherError, Result};

/// Create a Windows shortcut (.lnk file)
pub fn create_shortcut(
    target_path: &PathBuf,
    shortcut_path: &PathBuf,
    _description: &str,
    working_dir: Option<&PathBuf>,
) -> Result<()> {
    tracing::info!("Creating shortcut: {} -> {}", shortcut_path.display(), target_path.display());

    // Ensure parent directory exists
    if let Some(parent) = shortcut_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let work_dir = working_dir
        .map(|p| p.as_path())
        .or_else(|| target_path.parent())
        .unwrap();

    // Create proper .lnk shortcut using mslnk
    let lnk_path = shortcut_path.with_extension("lnk");
    
    tracing::info!("Creating .lnk file at: {}", lnk_path.display());
    
    let mut shortcut = mslnk::ShellLink::new(target_path)
        .map_err(|e| {
            tracing::error!("Failed to create ShellLink: {}", e);
            LauncherError::FileSystem(format!("Failed to create shortcut: {}", e))
        })?;

    shortcut.set_working_dir(Some(work_dir.to_string_lossy().to_string()));
    shortcut.set_icon_location(Some(target_path.to_string_lossy().to_string()));

    shortcut.create_lnk(&lnk_path)
        .map_err(|e| {
            tracing::error!("Failed to save .lnk file: {}", e);
            LauncherError::FileSystem(format!("Failed to save shortcut: {}", e))
        })?;

    tracing::info!("✓ Shortcut created successfully: {}", lnk_path.display());
    Ok(())
}

/// Create desktop shortcut
pub fn create_desktop_shortcut(
    game_name: &str,
    target_path: &PathBuf,
    working_dir: Option<&PathBuf>,
) -> Result<()> {
    let desktop = dirs::desktop_dir()
        .ok_or_else(|| LauncherError::FileSystem("Could not find desktop directory".to_string()))?;

    tracing::info!("Desktop directory: {}", desktop.display());
    
    let shortcut_path = desktop.join(game_name);
    tracing::info!("Creating desktop shortcut for: {}", game_name);
    
    create_shortcut(target_path, &shortcut_path, game_name, working_dir)
}

/// Create Start Menu shortcut
pub fn create_start_menu_shortcut(
    game_name: &str,
    target_path: &PathBuf,
    working_dir: Option<&PathBuf>,
) -> Result<()> {
    // Get Start Menu Programs directory
    let start_menu = PathBuf::from(std::env::var("APPDATA").unwrap_or_default())
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("DexLauncher");

    fs::create_dir_all(&start_menu)?;

    tracing::info!("Start Menu directory: {}", start_menu.display());
    
    let shortcut_path = start_menu.join(game_name);
    tracing::info!("Creating Start Menu shortcut for: {}", game_name);
    
    create_shortcut(target_path, &shortcut_path, game_name, working_dir)
}

/// Remove desktop shortcut
pub fn remove_desktop_shortcut(game_name: &str) -> Result<()> {
    let desktop = dirs::desktop_dir()
        .ok_or_else(|| LauncherError::FileSystem("Could not find desktop directory".to_string()))?;

    let shortcut_path = desktop.join(format!("{}.lnk", game_name));
    if shortcut_path.exists() {
        fs::remove_file(&shortcut_path)?;
        tracing::info!("Removed desktop shortcut: {}", shortcut_path.display());
    }

    Ok(())
}

/// Remove Start Menu shortcut
pub fn remove_start_menu_shortcut(game_name: &str) -> Result<()> {
    let start_menu = PathBuf::from(std::env::var("APPDATA").unwrap_or_default())
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("DexLauncher");

    let shortcut_path = start_menu.join(format!("{}.lnk", game_name));
    if shortcut_path.exists() {
        fs::remove_file(&shortcut_path)?;
        tracing::info!("Removed Start Menu shortcut: {}", shortcut_path.display());
    }

    Ok(())
}
