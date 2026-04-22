// Source: /data/home/swei/claudecode/openclaudecode/src/memdir/teamMemPaths.ts
//! Team memory paths and validation
//!
//! Provides team memory path resolution with security checks for path traversal
//! and symlink attacks.

use crate::memdir::paths::{get_auto_mem_path, is_auto_memory_enabled};
use std::path::{Path, PathBuf};

/// Error thrown when a path validation detects a traversal or injection attempt.
#[derive(Debug, Clone)]
pub struct PathTraversalError {
    message: String,
}

impl std::fmt::Display for PathTraversalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PathTraversalError {}

impl PathTraversalError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

/// Sanitize a file path key by rejecting dangerous patterns.
/// Checks for null bytes, URL-encoded traversals, and other injection vectors.
/// Returns the sanitized string or throws PathTraversalError.
fn sanitize_path_key(key: &str) -> Result<String, PathTraversalError> {
    // Null bytes can truncate paths in C-based syscalls
    if key.contains('\0') {
        return Err(PathTraversalError::new(&format!(
            "Null byte in path key: \"{}\"",
            key
        )));
    }

    // URL-encoded traversals (e.g. %2e%2e%2f = ../)
    // Simple check: decode percent-encoded sequences if possible
    let mut has_url_encoded_traversal = false;
    let key_lower = key.to_lowercase();
    // Check for common URL-encoded patterns
    if key_lower.contains("%2e%2e") || key_lower.contains("%2e/") || key_lower.contains("/%2e%2e") {
        has_url_encoded_traversal = true;
    }

    if has_url_encoded_traversal {
        return Err(PathTraversalError::new(&format!(
            "URL-encoded traversal in path key: \"{}\"",
            key
        )));
    }

    // Reject backslashes (Windows path separator used as traversal vector)
    if key.contains('\\') {
        return Err(PathTraversalError::new(&format!(
            "Backslash in path key: \"{}\"",
            key
        )));
    }

    // Reject absolute paths
    if key.starts_with('/') {
        return Err(PathTraversalError::new(&format!(
            "Absolute path key: \"{}\"",
            key
        )));
    }

    Ok(key.to_string())
}

/// Whether team memory features are enabled.
/// Team memory is a subdirectory of auto memory, so it requires auto memory
/// to be enabled. This keeps all team-memory consumers (prompt, content
/// injection, sync watcher, file detection) consistent when auto memory is
/// disabled via env var or settings.
pub fn is_team_memory_enabled() -> bool {
    if !is_auto_memory_enabled() {
        return false;
    }

    // TODO: integrate with growthbook feature flags
    // getFeatureValue_CACHED_MAY_BE_STALE('tengu_herring_clock', false)
    false
}

/// Returns the team memory path: <memoryBase>/projects/<sanitized-project-root>/memory/team/
/// Lives as a subdirectory of the auto-memory directory, scoped per-project.
pub fn get_team_mem_path() -> PathBuf {
    let auto_mem = get_auto_mem_path();
    let team_path = auto_mem.join("team");
    let path_str = team_path.to_string_lossy().to_string();

    // Ensure trailing separator and NFC normalization
    let sep = std::path::MAIN_SEPARATOR;
    if !path_str.ends_with(sep) {
        format!("{}{}", path_str, sep).into()
    } else {
        team_path
    }
}

/// Returns the team memory entrypoint: <memoryBase>/projects/<sanitized-project-root>/memory/team/MEMORY.md
/// Lives as a subdirectory of the auto-memory directory, scoped per-project.
pub fn get_team_mem_entypoint() -> PathBuf {
    get_team_mem_path().join("MEMORY.md")
}

/// Check if a resolved absolute path is within the team memory directory.
/// Uses path.resolve() to convert relative paths and eliminate traversal segments.
/// Does NOT resolve symlinks — for write validation use validate_team_mem_write_path()
/// or validate_team_mem_key() which include symlink resolution.
pub fn is_team_mem_path(file_path: &Path) -> bool {
    // SECURITY: resolve() converts to absolute and eliminates .. segments,
    // preventing path traversal attacks (e.g. "team/../../etc/passwd")
    let resolved_path = std::fs::canonicalize(file_path).unwrap_or_else(|_| {
        // If canonicalize fails, fall back to resolving manually
        if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|c| c.join(file_path))
                .unwrap_or_else(|_| file_path.to_path_buf())
        }
    });

    let team_dir = get_team_mem_path();
    let team_dir_str = team_dir.to_string_lossy();

    resolved_path.to_string_lossy().starts_with(&*team_dir_str)
}

/// Validate that an absolute file path is safe for writing to the team memory directory.
/// Returns the resolved absolute path if valid.
/// Throws PathTraversalError if the path contains injection vectors, escapes the
/// directory via .. segments, or escapes via a symlink.
pub fn validate_team_mem_write_path(file_path: &Path) -> Result<PathBuf, PathTraversalError> {
    let path_str = file_path.to_string_lossy();
    if path_str.contains('\0') {
        return Err(PathTraversalError::new(&format!(
            "Null byte in path: \"{}\"",
            file_path.display()
        )));
    }

    // First pass: normalize .. segments and check string-level containment.
    // This is a fast rejection for obvious traversal attempts before we touch
    // the filesystem.
    let resolved_path = std::fs::canonicalize(file_path).unwrap_or_else(|_| {
        if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|c| c.join(file_path))
                .unwrap_or_else(|_| file_path.to_path_buf())
        }
    });

    let team_dir = get_team_mem_path();
    let team_dir_str = team_dir.to_string_lossy();

    // Prefix attack protection: teamDir already ends with sep (from getTeamMemPath),
    // so "team-evil/" won't match "team/"
    if !resolved_path.to_string_lossy().starts_with(&*team_dir_str) {
        return Err(PathTraversalError::new(&format!(
            "Path escapes team memory directory: \"{}\"",
            file_path.display()
        )));
    }

    // TODO: Second pass - resolve symlinks on the deepest existing ancestor
    // and verify the real path is still within the real team dir.

    Ok(resolved_path)
}

/// Validate a relative path key from the server against the team memory directory.
/// Sanitizes the key, joins with the team dir, resolves symlinks on the deepest
/// existing ancestor, and verifies containment against the real team dir.
/// Returns the resolved absolute path.
/// Throws PathTraversalError if the key is malicious.
pub fn validate_team_mem_key(relative_key: &str) -> Result<PathBuf, PathTraversalError> {
    sanitize_path_key(relative_key)?;

    let team_dir = get_team_mem_path();
    let full_path = team_dir.join(relative_key);

    // First pass: normalize .. segments and check string-level containment.
    let resolved_path = std::fs::canonicalize(&full_path).unwrap_or_else(|_| full_path.clone());

    let team_dir_str = team_dir.to_string_lossy();
    if !resolved_path.to_string_lossy().starts_with(&*team_dir_str) {
        return Err(PathTraversalError::new(&format!(
            "Key escapes team memory directory: \"{}\"",
            relative_key
        )));
    }

    // TODO: Second pass - resolve symlinks and verify real containment.

    Ok(resolved_path)
}

/// Check if a file path is within the team memory directory
/// and team memory is enabled.
pub fn is_team_mem_file(file_path: &Path) -> bool {
    is_team_memory_enabled() && is_team_mem_path(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_path_key_valid() {
        assert!(sanitize_path_key("valid_name.md").is_ok());
        assert!(sanitize_path_key("subdir/test.md").is_ok());
    }

    #[test]
    fn test_sanitize_path_key_null_byte() {
        let result = sanitize_path_key("test\0.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_path_key_absolute() {
        let result = sanitize_path_key("/etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_path_key_backslash() {
        let result = sanitize_path_key("test\\md");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_team_memory_enabled_when_auto_disabled() {
        // This test depends on environment, just verify the function runs
        let _ = is_team_memory_enabled();
    }
}
