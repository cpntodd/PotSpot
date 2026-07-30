// Photo service -- handles upload, EXIF stripping, thumbnail generation.

use anyhow::Context;
use image::GenericImageView;
use std::io::Cursor;

/// Supported MIME types for photo uploads.
const ALLOWED_MIME_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp"];

/// Maximum file sizes.
pub const MAX_PRIMARY_PHOTO_SIZE: usize = 20 * 1024 * 1024; // 20 MB
pub const MAX_REVIEW_PHOTO_SIZE: usize = 20 * 1024 * 1024;   // 20 MB

/// Validate that the uploaded file is a supported image type and within size limits.
pub fn validate_photo(data: &[u8], mime_type: &str, is_primary: bool) -> anyhow::Result<()> {
    let max_size = if is_primary {
        MAX_PRIMARY_PHOTO_SIZE
    } else {
        MAX_REVIEW_PHOTO_SIZE
    };

    if data.len() > max_size {
        anyhow::bail!(
            "File size exceeds maximum of {} bytes (got {} bytes)",
            max_size,
            data.len()
        );
    }

    if !ALLOWED_MIME_TYPES.contains(&mime_type) {
        anyhow::bail!(
            "Unsupported MIME type: {}. Allowed: {:?}",
            mime_type,
            ALLOWED_MIME_TYPES
        );
    }

    Ok(())
}

/// Strip EXIF metadata from an image.
/// Returns the cleaned image bytes.
pub fn strip_exif(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let img = image::load_from_memory(data)
        .context("Failed to decode image for EXIF stripping")?;

    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let _ = (width, height); // Dimensions preserved in re-encode; suppress unused warning

    // Re-encode as WebP (which doesn't carry EXIF from the source)
    let mut output = Cursor::new(Vec::new());
    let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);

    // Convert RGBA to the encoder's expected format
    let dynamic = image::DynamicImage::ImageRgba8(rgba);
    dynamic.write_with_encoder(encoder)
        .context("Failed to re-encode image without EXIF")?;

    Ok(output.into_inner())
}

/// Generate a thumbnail (300px wide, WebP, quality 80).
pub fn generate_thumbnail(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let img = image::load_from_memory(data)
        .context("Failed to decode image for thumbnail generation")?;

    let (orig_width, orig_height) = img.dimensions();
    let thumb_width: u32 = 300;
    let thumb_height = (orig_height as f64 * (thumb_width as f64 / orig_width as f64)) as u32;
    let thumb_height = thumb_height.max(1);

    let thumbnail = img.resize_exact(
        thumb_width,
        thumb_height,
        image::imageops::FilterType::Lanczos3,
    );

    let mut output = Cursor::new(Vec::new());
    let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
    thumbnail.write_with_encoder(encoder)
        .context("Failed to encode thumbnail")?;

    Ok(output.into_inner())
}

/// Get image dimensions from raw bytes.
pub fn get_dimensions(data: &[u8]) -> anyhow::Result<(u32, u32)> {
    let img = image::load_from_memory(data)
        .context("Failed to decode image for dimensions")?;
    Ok(img.dimensions())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_photo_rejects_large_files() {
        let data = vec![0u8; MAX_REVIEW_PHOTO_SIZE + 1];
        let result = validate_photo(&data, "image/jpeg", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_photo_rejects_bad_mime() {
        let data = vec![0u8; 100];
        let result = validate_photo(&data, "text/plain", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_photo_accepts_valid() {
        // Create a minimal valid WebP image (1x1 pixel)
        let data = vec![0u8; 100]; // Not actually valid, but within size
        let result = validate_photo(&data, "image/webp", false);
        assert!(result.is_ok());
    }
}
