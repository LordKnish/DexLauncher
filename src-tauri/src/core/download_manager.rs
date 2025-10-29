use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::fs::File;
use std::io::{Write, Seek, SeekFrom};
use curl::easy::Easy;
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

use crate::error::{LauncherError, Result};
use crate::db::DbState;
use crate::core::installer::InstallProgress;

/// Enhanced download manager with resume capability
pub struct DownloadManager {
    db: Arc<DbState>,
    app_handle: AppHandle,
}

impl DownloadManager {
    /// Create a new download manager
    pub fn new(db: Arc<DbState>, app_handle: AppHandle) -> Self {
        Self { db, app_handle }
    }

    /// Check if server supports range requests
    fn supports_range_requests(url: &str) -> Result<bool> {
        let mut easy = Easy::new();
        easy.url(url).map_err(|e| LauncherError::Network(format!("Failed to set URL: {}", e)))?;
        easy.useragent("DexLauncher/0.1.0").map_err(|e| LauncherError::Network(format!("Failed to set user agent: {}", e)))?;
        easy.nobody(true).map_err(|e| LauncherError::Network(format!("Failed to set nobody: {}", e)))?;
        easy.follow_location(true).map_err(|e| LauncherError::Network(format!("Failed to set follow location: {}", e)))?;
        easy.timeout(std::time::Duration::from_secs(10)).map_err(|e| LauncherError::Network(format!("Failed to set timeout: {}", e)))?;

        let mut supports_ranges = false;
        {
            let mut transfer = easy.transfer();
            transfer.header_function(|header| {
                if let Ok(header_str) = std::str::from_utf8(header) {
                    if header_str.to_lowercase().starts_with("accept-ranges:") {
                        if header_str.to_lowercase().contains("bytes") {
                            supports_ranges = true;
                        }
                    }
                }
                true
            }).map_err(|e| LauncherError::Network(format!("Failed to set header function: {}", e)))?;
            transfer.perform().map_err(|e| LauncherError::Network(format!("Failed to perform request: {}", e)))?;
        }

        Ok(supports_ranges)
    }

    /// Get file size via HEAD request
    fn get_file_size(url: &str) -> Result<Option<u64>> {
        let mut easy = Easy::new();
        easy.url(url).map_err(|e| LauncherError::Network(format!("Failed to set URL: {}", e)))?;
        easy.useragent("DexLauncher/0.1.0").map_err(|e| LauncherError::Network(format!("Failed to set user agent: {}", e)))?;
        easy.nobody(true).map_err(|e| LauncherError::Network(format!("Failed to set nobody: {}", e)))?;
        easy.follow_location(true).map_err(|e| LauncherError::Network(format!("Failed to set follow location: {}", e)))?;
        easy.timeout(std::time::Duration::from_secs(30)).map_err(|e| LauncherError::Network(format!("Failed to set timeout: {}", e)))?;

        easy.perform().map_err(|e| LauncherError::Network(format!("Failed to perform request: {}", e)))?;

        let size = easy.download_size().map_err(|e| {
            LauncherError::Network(format!("Failed to get download size: {}", e))
        })?;
        
        if size > 0.0 {
            Ok(Some(size as u64))
        } else {
            Ok(None)
        }
    }

    /// Download file with resume support
    pub async fn download_with_resume(
        &self,
        url: &str,
        dest_path: &Path,
        operation_id: String,
        expected_size: Option<u64>,
        cancel_token: CancellationToken,
    ) -> Result<()> {
        tracing::info!("=== STARTING RESUMABLE DOWNLOAD ===");
        tracing::info!("URL: {}", url);
        tracing::info!("Destination: {}", dest_path.display());

        // Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let temp_path = dest_path.with_extension("part");

        // Check if server supports range requests
        let url_clone = url.to_string();
        let supports_resume = tokio::task::spawn_blocking(move || {
            Self::supports_range_requests(&url_clone)
        }).await
            .map_err(|e| LauncherError::General(format!("Failed to check range support: {}", e)))??;

        tracing::info!("Server supports resume: {}", supports_resume);

        // Get file size
        let total_size = if let Some(size) = expected_size {
            size
        } else {
            let url_clone = url.to_string();
            tokio::task::spawn_blocking(move || {
                Self::get_file_size(&url_clone)
            }).await
                .map_err(|e| LauncherError::General(format!("Failed to get file size: {}", e)))??
                .unwrap_or(0)
        };

        // Check download cache for resume
        let mut start_byte = 0u64;
        let cache = self.db.get_download_cache(url)?;

        if let Some(cache_entry) = cache {
            if temp_path.exists() {
                let metadata = temp_path.metadata()?;
                let actual_size = metadata.len();

                // Validate cached download
                if actual_size == cache_entry.downloaded_bytes as u64 && supports_resume {
                    start_byte = actual_size;
                    tracing::info!("Resuming download from byte {}", start_byte);
                } else {
                    tracing::warn!("Cache mismatch or no resume support, starting fresh");
                    std::fs::remove_file(&temp_path)?;
                    self.db.delete_download_cache(url)?;
                }
            } else {
                // Cache exists but file doesn't, clean up cache
                self.db.delete_download_cache(url)?;
            }
        }

        // Open file for writing (append if resuming)
        let file = if start_byte > 0 {
            std::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(&temp_path)?
        } else {
            File::create(&temp_path)?
        };

        let file = Arc::new(std::sync::Mutex::new(file));
        let downloaded = Arc::new(std::sync::Mutex::new(start_byte));
        let last_emit = Arc::new(std::sync::Mutex::new(std::time::Instant::now()));
        let last_cache_update = Arc::new(std::sync::Mutex::new(std::time::Instant::now()));

        // Clone for closures
        let app_handle = self.app_handle.clone();
        let op_id = operation_id.clone();
        let downloaded_clone = Arc::clone(&downloaded);
        let downloaded_clone2 = Arc::clone(&downloaded);
        let last_emit_clone = Arc::clone(&last_emit);
        let last_cache_update_clone = Arc::clone(&last_cache_update);
        let db = Arc::clone(&self.db);
        let url_owned = url.to_string();
        let temp_path_str = temp_path.to_string_lossy().to_string();

        // Perform download
        let url_owned2 = url.to_string();
        let temp_path_clone = temp_path.clone();
        let dest_path_clone = dest_path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let mut easy = Easy::new();
            easy.url(&url_owned2)?;
            easy.useragent("DexLauncher/0.1.0")?;
            easy.follow_location(true)?;
            easy.timeout(std::time::Duration::from_secs(300))?;

            // Set resume range if applicable
            if start_byte > 0 && supports_resume {
                easy.resume_from(start_byte)?;
                tracing::info!("Set resume from byte {}", start_byte);
            }

            // Write callback
            let file_clone = Arc::clone(&file);
            let downloaded_write = Arc::clone(&downloaded_clone2);
            let cancel_token_write = cancel_token.clone();
            easy.write_function(move |data| {
                // Check cancellation
                if cancel_token_write.is_cancelled() {
                    return Err(curl::easy::WriteError::Pause);
                }

                let mut file = file_clone.lock().unwrap();
                match file.write_all(data) {
                    Ok(_) => {
                        let mut dl = downloaded_write.lock().unwrap();
                        *dl += data.len() as u64;
                        Ok(data.len())
                    }
                    Err(e) => {
                        tracing::error!("Write error: {}", e);
                        Err(curl::easy::WriteError::Pause)
                    }
                }
            })?;

            // Progress callback
            easy.progress(true)?;
            easy.progress_function(move |_total_download, _downloaded_now, _, _| {
                // Check cancellation
                if cancel_token.is_cancelled() {
                    return false;
                }

                let current = {
                    let dl = downloaded_clone.lock().unwrap();
                    *dl
                };

                // Update cache every 1MB
                let mut last_cache = last_cache_update_clone.lock().unwrap();
                if current > 0 && (current % (1024 * 1024) == 0 || last_cache.elapsed().as_secs() >= 5) {
                    if let Err(e) = db.upsert_download_cache(
                        &url_owned,
                        &temp_path_str,
                        Some(total_size as i64),
                        current as i64,
                    ) {
                        tracing::warn!("Failed to update download cache: {}", e);
                    }
                    *last_cache = std::time::Instant::now();
                }

                // Emit progress
                let mut last = last_emit_clone.lock().unwrap();
                if last.elapsed().as_millis() >= 200 {
                    let percentage = if total_size > 0 {
                        ((current as f64 / total_size as f64) * 70.0).round()
                    } else {
                        5.0
                    };

                    let message = if total_size > 0 {
                        format!("Downloading: {:.1} MB / {:.1} MB",
                            current as f64 / 1_048_576.0,
                            total_size as f64 / 1_048_576.0)
                    } else {
                        format!("Downloading: {:.1} MB...",
                            current as f64 / 1_048_576.0)
                    };

                    let progress = InstallProgress {
                        operation_id: op_id.clone(),
                        phase: "downloading".to_string(),
                        percentage,
                        message,
                    };

                    if let Err(e) = app_handle.emit("install-progress", progress) {
                        tracing::warn!("Failed to emit progress: {}", e);
                    }

                    *last = std::time::Instant::now();
                }

                true
            })?;

            // Perform download
            tracing::info!("Starting curl transfer...");
            easy.perform()?;

            let response_code = easy.response_code()?;
            tracing::info!("Download complete! Response code: {}", response_code);

            if response_code != 200 && response_code != 206 {
                return Err(LauncherError::Network(format!(
                    "Download failed with HTTP {}", response_code
                )));
            }

            // Flush file
            {
                let mut file = file.lock().unwrap();
                file.flush()?;
            }

            // Rename to final name
            std::fs::rename(&temp_path_clone, &dest_path_clone)?;

            tracing::info!("=== DOWNLOAD COMPLETE ===");
            Ok::<(), LauncherError>(())
        }).await
            .map_err(|e| LauncherError::General(format!("Download task failed: {}", e)))??;

        // Clean up cache on success
        self.db.delete_download_cache(url)?;

        Ok(())
    }

    /// Clean expired cache entries
    pub fn clean_expired_cache(&self) -> Result<()> {
        self.db.clean_expired_cache()
    }
}
