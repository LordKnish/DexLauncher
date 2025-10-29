// Prevents additional console window on Windows in release
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod core;
mod db;
mod error;
mod utils;
mod platform;

use std::sync::Arc;
use commands::AppState;
use core::{OperationManager, LauncherUpdater};
use db::DbState;
use utils::get_db_path;

fn main() {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Initialize database
    let db_path = get_db_path().expect("Failed to get database path");
    let db = Arc::new(DbState::new(db_path).expect("Failed to initialize database"));

    // Initialize operation manager
    let operation_manager = OperationManager::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Start background update checker
            LauncherUpdater::start_background_checker(app.handle().clone());
            Ok(())
        })
        .manage(AppState { 
            db,
            operation_manager,
        })
        .invoke_handler(tauri::generate_handler![
            commands::check_system_ready,
            commands::get_installations,
            commands::get_installation,
            commands::select_install_directory,
            commands::check_disk_space,
            commands::install_game,
            commands::launch_game,
            commands::delete_installation,
            commands::get_setting,
            commands::set_setting,
            commands::get_all_settings,
            commands::cancel_operation,
            commands::get_repository_size,
            commands::check_for_updates,
            commands::update_game,
            commands::verify_installation,
            commands::repair_installation,
            commands::expand_path,
            commands::open_directory,
            commands::add_to_steam,
            commands::check_steam_shortcut,
            commands::remove_from_steam,
            commands::install_steam_artwork,
            commands::check_steam_status,
            commands::close_steam,
            core::check_launcher_update,
            core::install_launcher_update,
            core::restart_launcher,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
