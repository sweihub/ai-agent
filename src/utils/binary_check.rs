// Source: ~/claudecode/openclaudecode/src/utils/binaryCheck.ts
//! Check if a binary/command is installed and available on the system.
//! Uses 'which' on Unix systems (macOS, Linux, WSL) and 'where' on Windows.

#![allow(dead_code)]

use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;

static BINARY_CACHE: Mutex<HashMap<String, bool>> = Mutex::new(HashMap::new());

/// Check if a binary/command is installed and available on the system.
/// Uses 'which' on Unix systems (macOS, Linux, WSL) and 'where' on Windows.
///
/// # Arguments
/// * `command` - The command name to check (e.g. 'gopls', 'rust-analyzer')
///
/// # Returns
/// true if the command exists, false otherwise
pub async fn is_binary_installed(command: &str) -> bool {
    // Edge case: empty or whitespace-only command
    let trimmed = command.trim();
    if trimmed.is_empty() {
        log_for_debugging("[binaryCheck] Empty command provided, returning false");
        return false;
    }

    // Check cache first
    {
        let cache = BINARY_CACHE.lock().unwrap();
        if let Some(&cached) = cache.get(trimmed) {
            log_for_debugging(&format!(
                "[binaryCheck] Cache hit for '{trimmed}': {cached}"
            ));
            return cached;
        }
    }

    let exists = which_sync(trimmed);

    // Cache the result
    {
        let mut cache = BINARY_CACHE.lock().unwrap();
        cache.insert(trimmed.to_string(), exists);
    }

    log_for_debugging(&format!(
        "[binaryCheck] Binary '{trimmed}' {}",
        if exists { "found" } else { "not found" }
    ));

    exists
}

/// Synchronous check if a command exists on PATH.
fn which_sync(command: &str) -> bool {
    #[cfg(unix)]
    {
        Command::new("which")
            .arg(command)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        Command::new("where")
            .arg(command)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// Clear the binary check cache (useful for testing)
pub fn clear_binary_cache() {
    BINARY_CACHE.lock().unwrap().clear();
}

/// Log for debugging (stub implementation).
fn log_for_debugging(_msg: &str) {
    // In a full implementation, this would write to a debug log.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_binary_installed_known_command() {
        // "sh" should exist on most Unix systems
        #[cfg(unix)]
        {
            let rt = tokio::runtime::Runtime::new().unwrap();
            assert!(rt.block_on(is_binary_installed("sh")));
        }
    }

    #[test]
    fn test_is_binary_installed_unknown_command() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(!rt.block_on(is_binary_installed(
            "this-command-definitely-does-not-exist-xyz123"
        )));
    }

    #[test]
    fn test_empty_command() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(!rt.block_on(is_binary_installed("")));
        assert!(!rt.block_on(is_binary_installed("   ")));
    }

    #[test]
    fn test_clear_binary_cache() {
        clear_binary_cache();
    }
}
