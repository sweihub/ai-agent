//! Sandbox utilities for isolated execution.

use crate::constants::env::ai_code;
use std::path::Path;

/// Check if sandbox mode is enabled
pub fn is_sandbox_enabled() -> bool {
    std::env::var(ai_code::SANDBOX_ENABLED)
        .map(|v| v == "true")
        .unwrap_or(false)
}

/// Get the sandbox directory
pub fn get_sandbox_dir() -> Option<std::path::PathBuf> {
    std::env::var(ai_code::SANDBOX_DIR)
        .ok()
        .map(std::path::PathBuf::from)
}

/// Check if a path is within the sandbox
pub fn is_path_in_sandbox(path: &Path) -> bool {
    if let Some(sandbox_dir) = get_sandbox_dir() {
        return path.starts_with(&sandbox_dir);
    }

    // If no sandbox directory configured, allow access
    true
}
