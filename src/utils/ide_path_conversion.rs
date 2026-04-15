//! IDE path conversion utilities
//!
//! Handles conversions between Claude's environment and the IDE's environment.

use std::process::Command;

/// Path converter trait for IDE communication
pub trait IdePathConverter {
    /// Convert path from IDE format to local format
    fn to_local_path(&self, ide_path: &str) -> String;
    /// Convert path from local format to IDE format
    fn to_ide_path(&self, local_path: &str) -> String;
}

/// Converter for Windows IDE + WSL Claude scenario
pub struct WindowsToWSLConverter {
    wsl_distro_name: Option<String>,
}

impl WindowsToWSLConverter {
    /// Create a new converter
    pub fn new(wsl_distro_name: Option<String>) -> Self {
        Self { wsl_distro_name }
    }
}

impl IdePathConverter for WindowsToWSLConverter {
    fn to_local_path(&self, windows_path: &str) -> String {
        if windows_path.is_empty() {
            return windows_path.to_string();
        }

        // Check if this is a path from a different WSL distro
        if let Some(ref distro) = self.wsl_distro_name {
            if let Some(caps) = regex::Regex::new(r"^\\\\wsl(?:\.localhost|\$)\\([^\\]+)(.*)$")
                .ok()
                .and_then(|r| r.captures(windows_path))
            {
                if caps.get(1).map(|m| m.as_str()) != Some(distro.as_str()) {
                    // Different distro - wslpath will fail, so return original path
                    return windows_path.to_string();
                }
            }
        }

        // Try wslpath first
        if let Ok(result) = Command::new("wslpath").args(["-u", windows_path]).output() {
            if result.status.success() {
                return String::from_utf8_lossy(&result.stdout).trim().to_string();
            }
        }

        // Fall back to manual conversion
        // Convert backslashes to forward slashes
        let result = windows_path.replace('\\', "/");

        // Convert drive letter (e.g., C: -> /mnt/c)
        if result.len() >= 2 && result.chars().nth(1) == Some(':') {
            let letter = result.chars().next().unwrap().to_ascii_lowercase();
            return format!("/mnt/{}{}", letter, &result[2..]);
        }

        result
    }

    fn to_ide_path(&self, wsl_path: &str) -> String {
        if wsl_path.is_empty() {
            return wsl_path.to_string();
        }

        // Try wslpath first
        if let Ok(result) = Command::new("wslpath").args(["-w", wsl_path]).output() {
            if result.status.success() {
                return String::from_utf8_lossy(&result.stdout).trim().to_string();
            }
        }

        // If wslpath fails, return the original path
        wsl_path.to_string()
    }
}

/// Check if distro names match for WSL UNC paths
///
/// # Arguments
/// * `windows_path` - The Windows path to check
/// * `wsl_distro_name` - The WSL distro name to match against
///
/// # Returns
/// True if the path matches the distro or is not a WSL UNC path
pub fn check_wsl_distro_match(windows_path: &str, wsl_distro_name: &str) -> bool {
    if let Some(caps) = regex::Regex::new(r"^\\\\wsl(?:\.localhost|\$)\\([^\\]+)(.*)$")
        .ok()
        .and_then(|r| r.captures(windows_path))
    {
        caps.get(1).map(|m| m.as_str()) == Some(wsl_distro_name)
    } else {
        true // Not a WSL UNC path, so no distro mismatch
    }
}
