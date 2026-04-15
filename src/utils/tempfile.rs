// Source: /data/home/swei/claudecode/openclaudecode/src/utils/tempfile.ts
//! Temporary file path generation utilities.

use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// Generate a temporary file path.
///
/// # Arguments
/// * `prefix` - Optional prefix for the temp file name (defaults to "ai-prompt")
/// * `extension` - Optional file extension (defaults to ".md")
/// * `content_hash` - When provided, the identifier is derived from a SHA-256 hash
///   of this string (first 16 hex chars). This produces a path that is stable across
///   process boundaries - any process with the same content will get the same path.
///   Use this when the path ends up in content sent to the Anthropic API.
///
/// # Returns
/// Temp file path as a PathBuf
pub fn generate_temp_file_path(
    prefix: &str,
    extension: &str,
    content_hash: Option<&str>,
) -> PathBuf {
    let id = if let Some(hash_content) = content_hash {
        // Generate SHA-256 hash and take first 16 hex chars
        let mut hasher = Sha256::new();
        hasher.update(hash_content.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)[..16].to_string()
    } else {
        // Use UUID v4
        uuid::Uuid::new_v4().to_string()
    };

    let filename = format!("{}-{}-{}", prefix, id, extension);
    let tmpdir = std::env::temp_dir();
    tmpdir.join(filename)
}

/// Generate a temporary file path with default values.
/// Uses prefix "ai-prompt" and extension ".md"
pub fn generate_temp_file_path_default() -> PathBuf {
    generate_temp_file_path("ai-prompt", ".md", None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_temp_file_path_default() {
        let path = generate_temp_file_path_default();
        assert!(path.to_string_lossy().contains("ai-prompt"));
        assert!(path.to_string_lossy().ends_with(".md"));
    }

    #[test]
    fn test_generate_temp_file_path_custom() {
        let path = generate_temp_file_path("test-prefix", ".txt", None);
        assert!(path.to_string_lossy().contains("test-prefix"));
        assert!(path.to_string_lossy().ends_with(".txt"));
    }

    #[test]
    fn test_generate_temp_file_path_with_content_hash() {
        let content = "test content";
        let path1 = generate_temp_file_path("ai-prompt", ".md", Some(content));
        let path2 = generate_temp_file_path("ai-prompt", ".md", Some(content));

        // Same content should produce same path
        assert_eq!(path1, path2);

        // Path should be different when content is different
        let path3 = generate_temp_file_path("ai-prompt", ".md", Some("different"));
        assert_ne!(path1, path3);
    }

    #[test]
    fn test_generate_temp_file_path_stability() {
        // Verify the hash algorithm produces consistent results
        let content = "stable content";
        let path1 = generate_temp_file_path("ai-prompt", ".md", Some(content));
        let path2 = generate_temp_file_path("ai-prompt", ".md", Some(content));
        assert_eq!(path1, path2);
    }
}
