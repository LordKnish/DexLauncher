// Steam artwork generation from embedded assets
use image::{DynamicImage, ImageFormat, ImageReader};
use std::io::Cursor;
use crate::error::{LauncherError, Result};

// Embed artwork assets at compile time
const BANNER_BYTES: &[u8] = include_bytes!("../../assets/banner.png");
const LOGO_BYTES: &[u8] = include_bytes!("../../assets/logo.png");

/// Steam artwork dimensions
const GRID_WIDTH: u32 = 460;
const GRID_HEIGHT: u32 = 215;
const HERO_WIDTH: u32 = 1920;
const HERO_HEIGHT: u32 = 620;

/// Steam artwork bundle
#[derive(Debug)]
pub struct SteamArtwork {
    pub grid_jpeg: Vec<u8>,
    pub hero_jpeg: Vec<u8>,
    pub logo_png: Vec<u8>,
}

impl SteamArtwork {
    /// Generate Steam artwork from embedded assets
    pub fn generate() -> Result<Self> {
        tracing::info!("Generating Steam artwork from embedded assets...");

        // Load banner image
        let banner = Self::load_image(BANNER_BYTES, "banner.png")?;
        tracing::info!("Loaded banner: {}x{}", banner.width(), banner.height());

        // Generate grid image (460x215)
        let grid_jpeg = Self::resize_and_encode(&banner, GRID_WIDTH, GRID_HEIGHT)?;
        tracing::info!("Generated grid image: {} bytes", grid_jpeg.len());

        // Generate hero image (1920x620)
        let hero_jpeg = Self::resize_and_encode(&banner, HERO_WIDTH, HERO_HEIGHT)?;
        tracing::info!("Generated hero image: {} bytes", hero_jpeg.len());

        // Logo is already in correct format (transparent PNG)
        let logo_png = LOGO_BYTES.to_vec();
        tracing::info!("Using logo image: {} bytes", logo_png.len());

        Ok(Self {
            grid_jpeg,
            hero_jpeg,
            logo_png,
        })
    }

    /// Load image from bytes
    fn load_image(bytes: &[u8], name: &str) -> Result<DynamicImage> {
        let reader = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|e| LauncherError::Steam(format!("Failed to read {}: {}", name, e)))?;

        reader
            .decode()
            .map_err(|e| LauncherError::Steam(format!("Failed to decode {}: {}", name, e)))
    }

    /// Resize image and encode as JPEG
    fn resize_and_encode(image: &DynamicImage, width: u32, height: u32) -> Result<Vec<u8>> {
        // Calculate aspect ratios
        let src_ratio = image.width() as f32 / image.height() as f32;
        let dst_ratio = width as f32 / height as f32;

        // Determine crop dimensions to maintain aspect ratio
        let (crop_width, crop_height) = if src_ratio > dst_ratio {
            // Source is wider - crop width
            let crop_width = (image.height() as f32 * dst_ratio) as u32;
            (crop_width, image.height())
        } else {
            // Source is taller - crop height
            let crop_height = (image.width() as f32 / dst_ratio) as u32;
            (image.width(), crop_height)
        };

        // Calculate crop position (center crop)
        let x = (image.width() - crop_width) / 2;
        let y = (image.height() - crop_height) / 2;

        // Crop and resize
        let cropped = image.crop_imm(x, y, crop_width, crop_height);
        let resized = cropped.resize_exact(
            width,
            height,
            image::imageops::FilterType::Lanczos3,
        );

        // Encode as JPEG with 85% quality
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        
        resized
            .write_to(&mut cursor, ImageFormat::Jpeg)
            .map_err(|e| LauncherError::Steam(format!("Failed to encode JPEG: {}", e)))?;

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_artwork() {
        let artwork = SteamArtwork::generate().expect("Failed to generate artwork");
        
        // Verify sizes are reasonable
        assert!(artwork.grid_jpeg.len() > 1000, "Grid image too small");
        assert!(artwork.hero_jpeg.len() > 1000, "Hero image too small");
        assert!(artwork.logo_png.len() > 1000, "Logo image too small");
        
        // Verify JPEG magic bytes
        assert_eq!(&artwork.grid_jpeg[0..2], &[0xFF, 0xD8], "Grid not a JPEG");
        assert_eq!(&artwork.hero_jpeg[0..2], &[0xFF, 0xD8], "Hero not a JPEG");
        
        // Verify PNG magic bytes
        assert_eq!(&artwork.logo_png[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A], "Logo not a PNG");
    }
}
