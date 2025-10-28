// Verifier module - for integrity verification
use crate::error::Result;
use std::path::PathBuf;

/// Installation verifier
pub struct Verifier;

impl Verifier {
    pub fn new() -> Self {
        Self
    }

    /// Verify installation exists and has required files
    pub fn verify_basic(install_path: &PathBuf) -> Result<bool> {
        // Check if directory exists
        if !install_path.exists() {
            return Ok(false);
        }

        // Check for InfiniteFusion.exe (main executable)
        let game_exe = install_path.join("InfiniteFusion.exe");
        let exists = game_exe.exists();
        
        if !exists {
            tracing::warn!("InfiniteFusion.exe not found at: {}", game_exe.display());
        }
        
        Ok(exists)
    }
}