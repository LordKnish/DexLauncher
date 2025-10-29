use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::Write;
use curl::easy::Easy;
use tauri::{AppHandle, Emitter};

use crate::error::{LauncherError, Result};
use crate::core::installer::InstallProgress;

/// HTTP downloader using curl with progress tracking
pub struct Downloader {
    app_handle: AppHandle,
}

impl Downloader {
    /// Create a new downloader
    pub fn new(app_handle: AppHandle) -> Result<Self> {
        Ok(Self { app_handle })
    }

    /// Get file size via HEAD request
    fn get_file_size(url: &str) -> Result<Option<u64>> {
        tracing::info!("=== FETCHING FILE SIZE ===");
        tracing::info!("URL: {}", url);
        
        let mut easy = Easy::new();
        
        // Set URL
        easy.url(url).map_err(|e| {
            LauncherError::Network(format!("Failed to set URL: {}", e))
        })?;

        // Set user agent
        easy.useragent("DexLauncher/0.1.0").map_err(|e| {
            LauncherError::Network(format!("Failed to set user agent: {}", e))
        })?;

        // Follow redirects
        easy.follow_location(true).map_err(|e| {
            LauncherError::Network(format!("Failed to set follow location: {}", e))
        })?;

        // Use HEAD request (nobody = true means no body download)
        easy.nobody(true).map_err(|e| {
            LauncherError::Network(format!("Failed to set nobody: {}", e))
        })?;

        // Set timeout
        easy.timeout(std::time::Duration::from_secs(30)).map_err(|e| {
            LauncherError::Network(format!("Failed to set timeout: {}", e))
        })?;

        // Perform HEAD request
        tracing::info!("Performing HEAD request...");
        easy.perform().map_err(|e| {
            tracing::warn!("HEAD request failed: {}", e);
            LauncherError::Network(format!("Failed to get file size: {}", e))
        })?;

        // Check response code
        let response_code = easy.response_code().map_err(|e| {
            LauncherError::Network(format!("Failed to get response code: {}", e))
        })?;
        tracing::info!("HEAD response code: {}", response_code);

        // Get content length
        let size = easy.download_size().map_err(|e| {
            LauncherError::Network(format!("Failed to get download size: {}", e))
        })?;

        tracing::info!("Content-Length from HEAD: {}", size);

        if size > 0.0 {
            let size_bytes = size as u64;
            tracing::info!("✓ File size: {:.2} MB ({} bytes)", size_bytes as f64 / 1_048_576.0, size_bytes);
            Ok(Some(size_bytes))
        } else {
            tracing::warn!("✗ File size unknown - GitHub may not provide Content-Length for archive downloads");
            tracing::info!("Note: GitHub archive endpoints often generate files on-demand and don't provide size upfront");
            Ok(None)
        }
    }

    /// Download a file with progress tracking
    pub async fn download_with_resume(
        &self,
        url: &str,
        dest_path: &PathBuf,
        operation_id: String,
        expected_size: Option<u64>,
    ) -> Result<()> {
        tracing::info!("=== STARTING CURL DOWNLOAD ===");
        tracing::info!("URL: {}", url);
        tracing::info!("Destination: {}", dest_path.display());

        // Use expected size if provided, otherwise try HEAD request
        let prefetched_size = if let Some(size) = expected_size {
            tracing::info!("Using expected size from GitHub API: {:.2} MB", size as f64 / 1_048_576.0);
            Some(size)
        } else {
            let url_clone = url.to_string();
            tokio::task::spawn_blocking(move || {
                Self::get_file_size(&url_clone)
            }).await.map_err(|e| {
                LauncherError::General(format!("Failed to spawn size fetch task: {}", e))
            })??
        };

        // Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let temp_path = dest_path.with_extension("part");
        
        // GitHub archives don't support range requests, so always start fresh
        // Delete any partial download
        if temp_path.exists() {
            tracing::info!("Removing partial download (GitHub archives don't support resume)");
            std::fs::remove_file(&temp_path)?;
        }
        
        tracing::info!("Starting fresh download");
        let start_byte = 0u64;

        // Create new file for writing
        let file = File::create(&temp_path)?;
        let file = Arc::new(Mutex::new(file));

        // Progress tracking state
        let downloaded = Arc::new(Mutex::new(start_byte));
        let total_size = Arc::new(Mutex::new(prefetched_size.unwrap_or(0)));
        let last_emit = Arc::new(Mutex::new(std::time::Instant::now()));
        
        // Clone for closures
        let app_handle = self.app_handle.clone();
        let op_id = operation_id.clone();
        let downloaded_clone = Arc::clone(&downloaded);
        let downloaded_clone2 = Arc::clone(&downloaded);
        let downloaded_clone3 = Arc::clone(&downloaded);
        let total_clone = Arc::clone(&total_size);
        let last_emit_clone = Arc::clone(&last_emit);

        // Perform download in blocking task
        let url_owned = url.to_string();
        let temp_path_clone = temp_path.clone();
        let dest_path_clone = dest_path.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut easy = Easy::new();
            
            // Set URL
            easy.url(&url_owned).map_err(|e| {
                LauncherError::Network(format!("Failed to set URL: {}", e))
            })?;

            // Set user agent
            easy.useragent("DexLauncher/0.1.0").map_err(|e| {
                LauncherError::Network(format!("Failed to set user agent: {}", e))
            })?;

            // Follow redirects
            easy.follow_location(true).map_err(|e| {
                LauncherError::Network(format!("Failed to set follow location: {}", e))
            })?;

            // Set timeout
            easy.timeout(std::time::Duration::from_secs(300)).map_err(|e| {
                LauncherError::Network(format!("Failed to set timeout: {}", e))
            })?;

            // Don't set resume - GitHub archives don't support range requests

            // Write callback
            let file_clone = Arc::clone(&file);
            let downloaded_write = Arc::clone(&downloaded_clone2);
            easy.write_function(move |data| {
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
            }).map_err(|e| {
                LauncherError::Network(format!("Failed to set write function: {}", e))
            })?;

            // Progress callback
            easy.progress(true).map_err(|e| {
                LauncherError::Network(format!("Failed to enable progress: {}", e))
            })?;
            
            easy.progress_function(move |total_download, _downloaded_now, _, _| {
                // Update total size if we got it
                if total_download > 0.0 {
                    let mut total = total_clone.lock().unwrap();
                    let total_with_resume = start_byte + total_download as u64;
                    if *total != total_with_resume {
                        *total = total_with_resume;
                        tracing::info!("Total download size: {:.2} MB", *total as f64 / 1_048_576.0);
                    }
                }

                // Get current progress
                let current = {
                    let dl = downloaded_clone.lock().unwrap();
                    *dl
                };
                
                let total = {
                    let t = total_clone.lock().unwrap();
                    *t
                };

                // Emit progress update (throttled to every 200ms)
                let mut last = last_emit_clone.lock().unwrap();
                if last.elapsed().as_millis() >= 200 {
                    let percentage = if total > 0 {
                        ((current as f64 / total as f64) * 70.0).round()
                    } else {
                        5.0 // Show some progress even if size unknown
                    };

                    let message = if total > 0 {
                        let mb_current = current as f64 / 1_048_576.0;
                        let mb_total = total as f64 / 1_048_576.0;
                        format!("Downloading: {:.1} MB / {:.1} MB",
                            mb_current, mb_total)
                    } else {
                        format!("Downloading: {:.1} MB (size unknown)...",
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

                true // Continue download
            }).map_err(|e| {
                LauncherError::Network(format!("Failed to set progress function: {}", e))
            })?;

            // Perform the download
            tracing::info!("Starting curl transfer...");
            easy.perform().map_err(|e| {
                tracing::error!("Curl transfer failed: {}", e);
                LauncherError::Network(format!("Download failed: {}", e))
            })?;

            // Check response code
            let response_code = easy.response_code().map_err(|e| {
                LauncherError::Network(format!("Failed to get response code: {}", e))
            })?;

            tracing::info!("Download complete! Response code: {}", response_code);

            if response_code != 200 && response_code != 206 {
                return Err(LauncherError::Network(format!(
                    "Download failed with HTTP {}", response_code
                )));
            }

            // Flush and close file
            {
                let mut file = file.lock().unwrap();
                file.flush().map_err(|e| {
                    LauncherError::FileSystem(format!("Failed to flush file: {}", e))
                })?;
            }

            // Rename temp file to final name
            std::fs::rename(&temp_path_clone, &dest_path_clone).map_err(|e| {
                LauncherError::FileSystem(format!("Failed to finalize download: {}", e))
            })?;

            tracing::info!("=== DOWNLOAD COMPLETE ===");
            tracing::info!("File saved to: {}", dest_path_clone.display());
            
            let final_size = {
                let dl = downloaded_clone3.lock().unwrap();
                *dl
            };
            tracing::info!("Total downloaded: {:.2} MB", final_size as f64 / 1_048_576.0);

            Ok::<(), LauncherError>(())
        }).await.map_err(|e| {
            LauncherError::General(format!("Download task failed: {}", e))
        })??;

        Ok(())
    }
}
