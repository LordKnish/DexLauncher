use rusqlite::Connection;
use crate::error::Result;

/// Initialize the database schema
pub fn init_schema(conn: &Connection) -> Result<()> {
    // Create installations table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS installations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id TEXT NOT NULL,
            version TEXT NOT NULL,
            install_path TEXT NOT NULL,
            installed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_played DATETIME,
            size_bytes INTEGER,
            integrity_hash TEXT,
            is_valid BOOLEAN DEFAULT 1,
            UNIQUE(game_id, version, install_path)
        )",
        [],
    )?;

    // Create installation_files table for integrity verification
    conn.execute(
        "CREATE TABLE IF NOT EXISTS installation_files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            installation_id INTEGER NOT NULL,
            file_path TEXT NOT NULL,
            hash TEXT NOT NULL,
            size_bytes INTEGER NOT NULL,
            FOREIGN KEY(installation_id) REFERENCES installations(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Create index on installation_files for faster lookups
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_installation_files_installation_id 
         ON installation_files(installation_id)",
        [],
    )?;

    // Create settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Create download_cache table for resume capability
    conn.execute(
        "CREATE TABLE IF NOT EXISTS download_cache (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL UNIQUE,
            local_path TEXT NOT NULL,
            total_bytes INTEGER,
            downloaded_bytes INTEGER DEFAULT 0,
            hash TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            expires_at DATETIME
        )",
        [],
    )?;

    // Insert default settings if they don't exist
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('auto_update', 'false')",
        [],
    )?;
    
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('verify_integrity', 'true')",
        [],
    )?;

    Ok(())
}

/// Run database migrations
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Create migrations table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version INTEGER NOT NULL UNIQUE,
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Check current migration version
    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Apply migrations in order
    if current_version < 1 {
        // Migration 1: Initial schema (already applied by init_schema)
        conn.execute(
            "INSERT INTO migrations (version) VALUES (1)",
            [],
        )?;
    }

    // Future migrations can be added here
    // if current_version < 2 {
    //     // Migration 2: Add new column, etc.
    //     conn.execute("ALTER TABLE ...", [])?;
    //     conn.execute("INSERT INTO migrations (version) VALUES (2)", [])?;
    // }

    Ok(())
}