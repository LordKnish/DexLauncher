use std::path::PathBuf;
use std::sync::Arc;
use futures_util::StreamExt;
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::error::{LauncherError, Result};
use crate::utils::ProgressTracker;

/// HTTP downloader with progress tracking
pub struct Downloader {
    client: Client,
}

impl Downloader {
    /// Create a new downloader
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Pokemon-Fusion-Launcher/0.1.0")
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .map_err(|e| LauncherError::Network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client })
    }

    /// Download a file with progress tracking
    pub async fn download_file(
        &self,
        url: &str,
        dest_path: &PathBuf,
        progress: Option<Arc<ProgressTracker>>,
    ) -> Result<()> {
        tracing::info!("=== STARTING DOWNLOAD ===");
        tracing::info!("URL: {}", url);
        tracing::info!("Destination: {}", dest_path.display());

        // Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Start the download
        tracing::info!("Connecting to GitHub...");
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to connect: {}", e);
                LauncherError::Network(format!("Failed to start download: {}", e))
            })?;

        tracing::info!("Connected! Status: {}", response.status());

        if !response.status().is_success() {
            return Err(LauncherError::Network(format!(
                "Download failed with status: {}",
                response.status()
            )));
        }

        // Get total size
        let total_size = response.content_length().unwrap_or(0);
        tracing::info!("=== DOWNLOAD SIZE: {:.2} MB ({} bytes) ===",
            total_size as f64 / 1_048_576.0, total_size);

        // Update progress tracker with total size
        if let Some(ref p) = progress {
            p.set_total(total_size);
        }

        // Create temporary file
        let temp_path = dest_path.with_extension("part");
        let mut file = File::create(&temp_path)
            .await
            .map_err(|e| LauncherError::FileSystem(format!("Failed to create file: {}", e)))?;

        // Download with progress tracking
        tracing::info!("Starting download stream...");
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();
        let mut last_log = std::time::Instant::now();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| LauncherError::Network(format!("Download error: {}", e)))?;
            
            file.write_all(&chunk)
                .await
                .map_err(|e| LauncherError::FileSystem(format!("Failed to write file: {}", e)))?;

            downloaded += chunk.len() as u64;

            // Update progress
            if let Some(ref p) = progress {
                p.update(downloaded);
                
                // Check if cancelled
                if p.is_cancelled() {
                    tracing::warn!("Download cancelled by user");
                    let _ = tokio::fs::remove_file(&temp_path).await;
                    return Err(LauncherError::General("Download cancelled".to_string()));
                }
            }

            // Log progress every 2 seconds
            if last_log.elapsed().as_secs() >= 2 {
                tracing::info!("Downloaded: {:.1} MB / {:.1} MB ({:.1}%)",
                    downloaded as f64 / 1_048_576.0,
                    total_size as f64 / 1_048_576.0,
                    (downloaded as f64 / total_size as f64) * 100.0
                );
                last_log = std::time::Instant::now();
            }
        }

        // Flush and close file
        file.flush().await?;
        drop(file);

        // Rename temp file to final name
        tokio::fs::rename(&temp_path, dest_path)
            .await
            .map_err(|e| LauncherError::FileSystem(format!("Failed to finalize download: {}", e)))?;

        tracing::info!("=== DOWNLOAD COMPLETE ===");
        tracing::info!("File saved to: {}", dest_path.display());
        Ok(())
    }

    /// Download with resume capability
    pub async fn download_with_resume(
        &self,
        url: &str,
        dest_path: &PathBuf,
        progress: Option<Arc<ProgressTracker>>,
    ) -> Result<()> {
        tracing::info!("=== STARTING DOWNLOAD WITH RESUME ===");
        tracing::info!("URL: {}", url);
        tracing::info!("Destination: {}", dest_path.display());
        
        let temp_path = dest_path.with_extension("part");
        
        // Check if partial download exists
        let start_byte = if temp_path.exists() {
            let metadata = tokio::fs::metadata(&temp_path).await?;
            let size = metadata.len();
            tracing::info!("Found partial download, resuming from byte {} ({:.2} MB)",
                size, size as f64 / 1_048_576.0);
            size
        } else {
            tracing::info!("Starting fresh download");
            0
        };

        // Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Make request with Range header if resuming
        tracing::info!("Connecting to GitHub...");
        let mut request = self.client.get(url);
        if start_byte > 0 {
            request = request.header("Range", format!("bytes={}-", start_byte));
        }

        let response = request
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to connect: {}", e);
                LauncherError::Network(format!("Failed to start download: {}", e))
            })?;

        tracing::info!("Connected! Status: {}", response.status());

        if !response.status().is_success() && response.status().as_u16() != 206 {
            return Err(LauncherError::Network(format!(
                "Download failed with status: {}",
                response.status()
            )));
        }

        // Get total size
        let content_length = response.content_length().unwrap_or(0);
        let total_size = if start_byte > 0 {
            start_byte + content_length
        } else {
            content_length
        };

        tracing::info!("=== TOTAL DOWNLOAD SIZE: {:.2} MB ({} bytes) ===",
            total_size as f64 / 1_048_576.0, total_size);
        
        if start_byte > 0 {
            tracing::info!("Remaining to download: {:.2} MB",
                (total_size - start_byte) as f64 / 1_048_576.0);
        }

        // Update progress tracker
        if let Some(ref p) = progress {
            p.set_total(total_size);
            p.update(start_byte);
        }

        // Open file for appending
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&temp_path)
            .await
            .map_err(|e| LauncherError::FileSystem(format!("Failed to open file: {}", e)))?;

        // Download with progress
        tracing::info!("Starting download stream...");
        let mut downloaded = start_byte;
        let mut stream = response.bytes_stream();
        let mut last_log = std::time::Instant::now();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| LauncherError::Network(format!("Download error: {}", e)))?;
            
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            // Update progress
            if let Some(ref p) = progress {
                p.update(downloaded);
                
                if p.is_cancelled() {
                    tracing::warn!("Download cancelled by user");
                    return Err(LauncherError::General("Download cancelled".to_string()));
                }
            }

            // Log progress every 2 seconds
            if last_log.elapsed().as_secs() >= 2 {
                tracing::info!("Downloaded: {:.1} MB / {:.1} MB ({:.1}%)",
                    downloaded as f64 / 1_048_576.0,
                    total_size as f64 / 1_048_576.0,
                    (downloaded as f64 / total_size as f64) * 100.0
                );
                last_log = std::time::Instant::now();
            }
        }

        file.flush().await?;
        drop(file);

        // Rename to final name
        tokio::fs::rename(&temp_path, dest_path).await?;

        tracing::info!("=== DOWNLOAD COMPLETE ===");
        tracing::info!("File saved to: {}", dest_path.display());
        tracing::info!("Total downloaded: {:.2} MB", downloaded as f64 / 1_048_576.0);
        Ok(())
    }
}