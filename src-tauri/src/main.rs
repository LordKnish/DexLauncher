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
use db::DbState;
use utils::get_db_path;

fn main() {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Initialize database
    let db_path = get_db_path().expect("Failed to get database path");
    let db = Arc::new(DbState::new(db_path).expect("Failed to initialize database"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState { db })
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
