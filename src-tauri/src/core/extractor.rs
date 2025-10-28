use std::path::PathBuf;
use std::sync::Arc;
use std::io::Read;
use zip::ZipArchive;

use crate::error::{LauncherError, Result};
use crate::utils::ProgressTracker;

/// ZIP file extractor with progress tracking
pub struct Extractor;

impl Extractor {
    /// Extract a ZIP archive with progress tracking
    pub async fn extract_zip(
        archive_path: &PathBuf,
        dest_dir: &PathBuf,
        progress: Option<Arc<ProgressTracker>>,
    ) -> Result<()> {
        tracing::info!("Extracting: {} to {}", archive_path.display(), dest_dir.display());

        // Open the ZIP file
        let file = std::fs::File::open(archive_path)
            .map_err(|e| LauncherError::FileSystem(format!("Failed to open archive: {}", e)))?;

        let mut archive = ZipArchive::new(file)
            .map_err(|e| LauncherError::FileSystem(format!("Failed to read ZIP archive: {}", e)))?;

        let total_files = archive.len();
        tracing::info!("Archive contains {} files", total_files);

        // Update progress tracker
        if let Some(ref p) = progress {
            p.set_total(total_files as u64);
        }

        // Ensure destination directory exists
        tokio::fs::create_dir_all(dest_dir)
            .await
            .map_err(|e| LauncherError::FileSystem(format!("Failed to create destination directory: {}", e)))?;

        // Extract each file
        for i in 0..total_files {
            // Check if cancelled
            if let Some(ref p) = progress {
                if p.is_cancelled() {
                    tracing::warn!("Extraction cancelled");
                    return Err(LauncherError::General("Extraction cancelled".to_string()));
                }
            }

            let mut file = archive.by_index(i)
                .map_err(|e| LauncherError::FileSystem(format!("Failed to read file from archive: {}", e)))?;

            let file_path = file.name().to_string();
            
            // Skip if it's a directory
            if file.is_dir() {
                continue;
            }

            // Remove the first path component (the archive root folder)
            let relative_path = PathBuf::from(&file_path);
            let components: Vec<_> = relative_path.components().skip(1).collect();
            if components.is_empty() {
                continue;
            }
            let dest_file_path: PathBuf = components.iter().collect();
            let full_dest_path = dest_dir.join(&dest_file_path);

            // Create parent directories
            if let Some(parent) = full_dest_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| LauncherError::FileSystem(format!("Failed to create directory: {}", e)))?;
            }

            // Extract file
            let mut dest_file = std::fs::File::create(&full_dest_path)
                .map_err(|e| LauncherError::FileSystem(format!("Failed to create file {}: {}", full_dest_path.display(), e)))?;

            std::io::copy(&mut file, &mut dest_file)
                .map_err(|e| LauncherError::FileSystem(format!("Failed to extract file: {}", e)))?;

            // Update progress
            if let Some(ref p) = progress {
                p.update((i + 1) as u64);
            }

            // Log progress every 50 files for better visibility
            if (i + 1) % 50 == 0 {
                tracing::info!("Extracted {} / {} files ({:.1}%)",
                    i + 1, total_files,
                    ((i + 1) as f64 / total_files as f64) * 100.0
                );
            }
        }

        tracing::info!("Extraction complete: {} files extracted", total_files);
        Ok(())
    }

    /// Get the number of files in a ZIP archive
    pub fn count_files(archive_path: &PathBuf) -> Result<usize> {
        let file = std::fs::File::open(archive_path)
            .map_err(|e| LauncherError::FileSystem(format!("Failed to open archive: {}", e)))?;

        let archive = ZipArchive::new(file)
            .map_err(|e| LauncherError::FileSystem(format!("Failed to read ZIP archive: {}", e)))?;

        Ok(archive.len())
    }
}