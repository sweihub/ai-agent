//! Image validation utilities
//!
//! Validates that all images in messages are within the API size limit.

use serde::{Deserialize, Serialize};

/// Information about an oversized image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OversizedImage {
    /// Index of the image (1-based)
    pub index: usize,
    /// Size in bytes
    pub size: usize,
}

/// Error thrown when one or more images exceed the API size limit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSizeError {
    /// List of oversized images
    pub oversized_images: Vec<OversizedImage>,
    /// Maximum allowed size
    pub max_size: usize,
}

impl std::fmt::Display for ImageSizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let first_image = self.oversized_images.first();
        if self.oversized_images.len() == 1 && first_image.is_some() {
            let img = first_image.unwrap();
            write!(
                f,
                "Image base64 size ({} bytes) exceeds API limit ({} bytes). Please resize the image before sending.",
                Self::format_size(img.size),
                Self::format_size(self.max_size)
            )
        } else {
            let img_list = self
                .oversized_images
                .iter()
                .map(|img| format!("Image {}: {} bytes", img.index, Self::format_size(img.size)))
                .collect::<Vec<_>>()
                .join(", ");
            write!(
                f,
                "{} images exceed the API limit ({} bytes): {}. Please resize these images before sending.",
                self.oversized_images.len(),
                Self::format_size(self.max_size),
                img_list
            )
        }
    }
}

impl std::error::Error for ImageSizeError {}

impl ImageSizeError {
    fn format_size(bytes: usize) -> String {
        if bytes >= 1_000_000 {
            format!("{:.1}MB", bytes as f64 / 1_000_000.0)
        } else if bytes >= 1_000 {
            format!("{:.1}KB", bytes as f64 / 1_000.0)
        } else {
            format!("{}B", bytes)
        }
    }
}

/// API image maximum base64 size (5MB)
pub const API_IMAGE_MAX_BASE64_SIZE: usize = 5 * 1024 * 1024;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_message_single() {
        let error = ImageSizeError {
            oversized_images: vec![OversizedImage {
                index: 1,
                size: 6_000_000,
            }],
            max_size: API_IMAGE_MAX_BASE64_SIZE,
        };
        let msg = error.to_string();
        assert!(msg.contains("Image base64 size"));
    }

    #[test]
    fn test_error_message_multiple() {
        let error = ImageSizeError {
            oversized_images: vec![
                OversizedImage {
                    index: 1,
                    size: 6_000_000,
                },
                OversizedImage {
                    index: 2,
                    size: 7_000_000,
                },
            ],
            max_size: API_IMAGE_MAX_BASE64_SIZE,
        };
        let msg = error.to_string();
        assert!(msg.contains("2 images"));
    }
}
