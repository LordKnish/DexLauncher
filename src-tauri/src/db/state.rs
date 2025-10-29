use chrono::{DateTime, Utc};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

use crate::error::{LauncherError, Result};
use super::schema::{init_schema, run_migrations};

/// Database connection pool type
pub type DbPool = Pool<SqliteConnectionManager>;

/// Installation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installation {
    pub id: i64,
    pub game_id: String,
    pub version: String,
    pub install_path: String,
    pub installed_at: String,
    pub last_played: Option<String>,
    pub size_bytes: Option<i64>,
    pub integrity_hash: Option<String>,
    pub is_valid: bool,
}

/// Installation file record for integrity verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationFile {
    pub id: i64,
    pub installation_id: i64,
    pub file_path: String,
    pub hash: String,
    pub size_bytes: i64,
}

/// Download cache record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadCache {
    pub id: i64,
    pub url: String,
    pub local_path: String,
    pub total_bytes: Option<i64>,
    pub downloaded_bytes: i64,
    pub hash: Option<String>,
    pub created_at: String,
    pub expires_at: Option<String>,
}

/// Database state manager
pub struct DbState {
    pool: Arc<DbPool>,
}

impl DbState {
    /// Create a new database state manager
    pub fn new(db_path: PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let manager = SqliteConnectionManager::file(db_path);
        let pool = Pool::new(manager)
            .map_err(|e| LauncherError::Database(format!("Failed to create connection pool: {}", e)))?;

        // Initialize schema and run migrations
        let conn = pool.get()?;
        init_schema(&conn)?;
        run_migrations(&conn)?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    /// Get a connection from the pool
    fn get_conn(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.pool.get().map_err(|e| e.into())
    }

    // ===== Installation Methods =====

    /// Create a new installation record
    pub fn create_installation(
        &self,
        game_id: &str,
        version: &str,
        install_path: &str,
        size_bytes: Option<i64>,
    ) -> Result<i64> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO installations (game_id, version, install_path, size_bytes, installed_at)
             VALUES (?1, ?2, ?3, ?4, datetime('now'))",
            params![game_id, version, install_path, size_bytes],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Get all installations
    pub fn get_installations(&self) -> Result<Vec<Installation>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, game_id, version, install_path, installed_at, last_played, 
                    size_bytes, integrity_hash, is_valid
             FROM installations
             ORDER BY installed_at DESC"
        )?;

        let installations = stmt
            .query_map([], |row| {
                Ok(Installation {
                    id: row.get(0)?,
                    game_id: row.get(1)?,
                    version: row.get(2)?,
                    install_path: row.get(3)?,
                    installed_at: row.get(4)?,
                    last_played: row.get(5)?,
                    size_bytes: row.get(6)?,
                    integrity_hash: row.get(7)?,
                    is_valid: row.get(8)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(installations)
    }

    /// Get installation by ID
    pub fn get_installation(&self, id: i64) -> Result<Option<Installation>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, game_id, version, install_path, installed_at, last_played,
                    size_bytes, integrity_hash, is_valid
             FROM installations
             WHERE id = ?1"
        )?;

        let result = stmt.query_row([id], |row| {
            Ok(Installation {
                id: row.get(0)?,
                game_id: row.get(1)?,
                version: row.get(2)?,
                install_path: row.get(3)?,
                installed_at: row.get(4)?,
                last_played: row.get(5)?,
                size_bytes: row.get(6)?,
                integrity_hash: row.get(7)?,
                is_valid: row.get(8)?,
            })
        });

        match result {
            Ok(installation) => Ok(Some(installation)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Update installation validity
    pub fn update_installation_validity(&self, id: i64, is_valid: bool) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "UPDATE installations SET is_valid = ?1 WHERE id = ?2",
            params![is_valid, id],
        )?;
        Ok(())
    }

    /// Update installation version
    pub fn update_installation_version(&self, id: i64, version: &str) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "UPDATE installations SET version = ?1, is_valid = 1 WHERE id = ?2",
            params![version, id],
        )?;
        Ok(())
    }

    /// Update last played time
    pub fn update_last_played(&self, id: i64) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "UPDATE installations SET last_played = datetime('now') WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// Delete installation
    pub fn delete_installation(&self, id: i64) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM installations WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ===== Installation Files Methods =====

    /// Add installation file
    pub fn add_installation_file(
        &self,
        installation_id: i64,
        file_path: &str,
        hash: &str,
        size_bytes: i64,
    ) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO installation_files (installation_id, file_path, hash, size_bytes)
             VALUES (?1, ?2, ?3, ?4)",
            params![installation_id, file_path, hash, size_bytes],
        )?;
        Ok(())
    }

    /// Get installation files
    pub fn get_installation_files(&self, installation_id: i64) -> Result<Vec<InstallationFile>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, installation_id, file_path, hash, size_bytes
             FROM installation_files
             WHERE installation_id = ?1"
        )?;

        let files = stmt
            .query_map([installation_id], |row| {
                Ok(InstallationFile {
                    id: row.get(0)?,
                    installation_id: row.get(1)?,
                    file_path: row.get(2)?,
                    hash: row.get(3)?,
                    size_bytes: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(files)
    }

    /// Clear installation files
    pub fn clear_installation_files(&self, installation_id: i64) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "DELETE FROM installation_files WHERE installation_id = ?1",
            params![installation_id],
        )?;
        Ok(())
    }

    /// Update installation file hash
    pub fn update_installation_file_hash(
        &self,
        installation_id: i64,
        file_path: &str,
        hash: &str,
        size_bytes: u64,
    ) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "UPDATE installation_files 
             SET hash = ?1, size_bytes = ?2
             WHERE installation_id = ?3 AND file_path = ?4",
            params![hash, size_bytes as i64, installation_id, file_path],
        )?;
        Ok(())
    }

    // ===== Settings Methods =====

    /// Get setting value
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.get_conn()?;
        let result = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Set setting value
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at)
             VALUES (?1, ?2, datetime('now'))",
            params![key, value],
        )?;
        Ok(())
    }

    /// Get all settings
    pub fn get_all_settings(&self) -> Result<std::collections::HashMap<String, String>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        
        let settings = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<std::result::Result<std::collections::HashMap<_, _>, _>>()?;

        Ok(settings)
    }

    // ===== Download Cache Methods =====

    /// Create or update download cache
    pub fn upsert_download_cache(
        &self,
        url: &str,
        local_path: &str,
        total_bytes: Option<i64>,
        downloaded_bytes: i64,
    ) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO download_cache (url, local_path, total_bytes, downloaded_bytes, created_at)
             VALUES (?1, ?2, ?3, ?4, datetime('now'))
             ON CONFLICT(url) DO UPDATE SET
                downloaded_bytes = ?4,
                total_bytes = ?3",
            params![url, local_path, total_bytes, downloaded_bytes],
        )?;
        Ok(())
    }

    /// Get download cache
    pub fn get_download_cache(&self, url: &str) -> Result<Option<DownloadCache>> {
        let conn = self.get_conn()?;
        let result = conn.query_row(
            "SELECT id, url, local_path, total_bytes, downloaded_bytes, hash, created_at, expires_at
             FROM download_cache
             WHERE url = ?1",
            params![url],
            |row| {
                Ok(DownloadCache {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    local_path: row.get(2)?,
                    total_bytes: row.get(3)?,
                    downloaded_bytes: row.get(4)?,
                    hash: row.get(5)?,
                    created_at: row.get(6)?,
                    expires_at: row.get(7)?,
                })
            },
        );

        match result {
            Ok(cache) => Ok(Some(cache)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Delete download cache
    pub fn delete_download_cache(&self, url: &str) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM download_cache WHERE url = ?1", params![url])?;
        Ok(())
    }

    /// Clean expired download cache
    pub fn clean_expired_cache(&self) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "DELETE FROM download_cache WHERE expires_at < datetime('now')",
            [],
        )?;
        Ok(())
    }
}
