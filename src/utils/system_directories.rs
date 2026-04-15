// Source: ~/claudecode/openclaudecode/src/utils/systemDirectories.ts

use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// System directories for cross-platform access.
#[derive(Debug, Clone, Serialize)]
pub struct SystemDirectories {
    pub home: PathBuf,
    pub desktop: PathBuf,
    pub documents: PathBuf,
    pub downloads: PathBuf,
}

/// Platform type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Linux,
    Wsl,
    MacOs,
    Unknown,
}

impl Platform {
    /// Detect the current platform.
    pub fn detect() -> Self {
        if cfg!(target_os = "windows") {
            return Platform::Windows;
        }
        if cfg!(target_os = "linux") {
            // Check for WSL
            if std::fs::read_to_string("/proc/version")
                .map(|v| v.to_lowercase().contains("microsoft"))
                .unwrap_or(false)
            {
                return Platform::Wsl;
            }
            return Platform::Linux;
        }
        if cfg!(target_os = "macos") {
            return Platform::MacOs;
        }
        Platform::Unknown
    }
}

/// Options for getting system directories.
pub struct SystemDirectoriesOptions {
    pub env: Option<HashMap<String, String>>,
    pub home_dir: Option<PathBuf>,
    pub platform: Option<Platform>,
}

impl Default for SystemDirectoriesOptions {
    fn default() -> Self {
        Self {
            env: None,
            home_dir: None,
            platform: None,
        }
    }
}

/// Get cross-platform system directories.
/// Handles differences between Windows, macOS, Linux, and WSL.
pub fn get_system_directories(options: Option<SystemDirectoriesOptions>) -> SystemDirectories {
    let options = options.unwrap_or_default();
    let platform = options.platform.unwrap_or_else(Platform::detect);
    let home_dir = options.home_dir.unwrap_or_else(|| {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"))
    });

    let env = options.env.unwrap_or_else(|| {
        std::env::vars().collect()
    });

    // Default paths
    let defaults = SystemDirectories {
        home: home_dir.clone(),
        desktop: home_dir.join("Desktop"),
        documents: home_dir.join("Documents"),
        downloads: home_dir.join("Downloads"),
    };

    match platform {
        Platform::Windows => {
            // Windows: Use USERPROFILE if available (handles localized folder names)
            let user_profile = env
                .get("USERPROFILE")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir.clone());
            SystemDirectories {
                home: home_dir,
                desktop: user_profile.join("Desktop"),
                documents: user_profile.join("Documents"),
                downloads: user_profile.join("Downloads"),
            }
        }
        Platform::Linux | Platform::Wsl => {
            // Linux/WSL: Check XDG Base Directory specification first
            SystemDirectories {
                home: home_dir.clone(),
                desktop: env
                    .get("XDG_DESKTOP_DIR")
                    .map(PathBuf::from)
                    .unwrap_or(defaults.desktop),
                documents: env
                    .get("XDG_DOCUMENTS_DIR")
                    .map(PathBuf::from)
                    .unwrap_or(defaults.documents),
                downloads: env
                    .get("XDG_DOWNLOAD_DIR")
                    .map(PathBuf::from)
                    .unwrap_or(defaults.downloads),
            }
        }
        Platform::MacOs => defaults,
        Platform::Unknown => {
            eprintln!("Unknown platform detected, using default paths");
            defaults
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_platform() {
        let platform = Platform::detect();
        // Just verify it doesn't panic
        match platform {
            Platform::Windows | Platform::Linux | Platform::Wsl | Platform::MacOs | Platform::Unknown => {}
        }
    }

    #[test]
    fn test_get_system_directories_linux() {
        let dirs = get_system_directories(Some(SystemDirectoriesOptions {
            platform: Some(Platform::Linux),
            home_dir: Some(PathBuf::from("/home/test")),
            env: Some(HashMap::from([
                ("XDG_DESKTOP_DIR".to_string(), "/home/test/Desktop".to_string()),
            ])),
        }));
        assert_eq!(dirs.home, PathBuf::from("/home/test"));
        assert_eq!(dirs.desktop, PathBuf::from("/home/test/Desktop"));
    }
}
