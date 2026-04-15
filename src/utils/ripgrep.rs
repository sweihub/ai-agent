// Source: /data/home/swei/claudecode/openclaudecode/src/utils/ripgrep.ts
//! Ripgrep utility functions for searching files.

use std::process::Command;

/// Run ripgrep to find files matching a pattern
pub fn ripgrep_files(pattern: &str, path: &str) -> Result<Vec<String>, String> {
    let output = Command::new("rg")
        .args(["--files", "--hidden", "--glob", "!*node_modules*", path])
        .arg(pattern)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Ok(Vec::new()); // No matches is not an error
    }

    let files = String::from_utf8_lossy(&output.stdout);
    Ok(files.lines().map(|s| s.to_string()).collect())
}

/// Check if ripgrep is available
pub fn is_ripgrep_available() -> bool {
    Command::new("rg")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the ripgrep version
pub fn get_ripgrep_version() -> Option<String> {
    Command::new("rg")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
}
