//! System directories utility for cross-platform directory paths.
//!
//! Handles differences between Windows, macOS, Linux, and WSL.

use crate::constants::env::system;
use std::collections::HashMap;
use std::path::PathBuf;

/// Environment-like record for testing
type EnvLike = HashMap<String, Option<String>>;

/// Options for getting system directories
#[derive(Default)]
pub struct SystemDirectoriesOptions {
    pub env: Option<EnvLike>,
    pub homedir: Option<String>,
    pub platform: Option<String>,
}

/// Get a value from environment or default
fn get_env_var(env: &EnvLike, key: &str) -> Option<String> {
    env.get(key)
        .and_then(|v| v.clone())
        .or_else(|| std::env::var(key).ok())
}

/// Get the platform (windows, linux, macos, wsl, unknown)
fn get_platform(options: &SystemDirectoriesOptions) -> String {
    if let Some(platform) = &options.platform {
        return platform.clone();
    }

    #[cfg(target_os = "windows")]
    return "windows".to_string();

    #[cfg(target_os = "macos")]
    return "macos".to_string();

    #[cfg(target_os = "linux")]
    {
        // Check if running under WSL
        if std::env::var(system::WSL_DISTRO_NAME).is_ok()
            || std::env::var("microsoft")
                .map(|v| v.to_lowercase())
                .unwrap_or_default()
                .contains("microsoft")
        {
            return "wsl".to_string();
        }
        return "linux".to_string();
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return "unknown".to_string();
}

/// Get the home directory
fn get_home_dir(options: &SystemDirectoriesOptions) -> String {
    if let Some(home) = &options.homedir {
        return home.clone();
    }

    if let Some(env) = &options.env {
        if let Some(home) = env.get("HOME").and_then(|v| v.clone()) {
            return home;
        }
        #[cfg(target_os = "windows")]
        if let Some(home) = env.get("USERPROFILE").and_then(|v| v.clone()) {
            return home;
        }
    }

    dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "".to_string())
}

/// Get cross-platform system directories
/// Handles differences between Windows, macOS, Linux, and WSL
///
/// # Arguments
/// * `options` - Optional overrides for testing (env, homedir, platform)
///
/// # Returns
/// A HashMap containing HOME, DESKTOP, DOCUMENTS, DOWNLOADS paths
pub fn get_system_directories(
    options: Option<SystemDirectoriesOptions>,
) -> HashMap<String, String> {
    let options = options.unwrap_or_default();
    let platform = get_platform(&options);
    let home_dir = get_home_dir(&options);

    let env = options.env.as_ref().map(|e| e).unwrap_or(&HashMap::new());

    // Default paths used by most platforms
    let mut defaults = HashMap::new();
    defaults.insert("HOME".to_string(), home_dir.clone());
    defaults.insert(
        "DESKTOP".to_string(),
        PathBuf::from(&home_dir)
            .join("Desktop")
            .to_string_lossy()
            .to_string(),
    );
    defaults.insert(
        "DOCUMENTS".to_string(),
        PathBuf::from(&home_dir)
            .join("Documents")
            .to_string_lossy()
            .to_string(),
    );
    defaults.insert(
        "DOWNLOADS".to_string(),
        PathBuf::from(&home_dir)
            .join("Downloads")
            .to_string_lossy()
            .to_string(),
    );

    match platform.as_str() {
        "windows" => {
            // Windows: Use USERPROFILE if available (handles localized folder names)
            let user_profile = get_env_var(env, "USERPROFILE").unwrap_or(home_dir.clone());
            let mut result = HashMap::new();
            result.insert("HOME".to_string(), home_dir);
            result.insert(
                "DESKTOP".to_string(),
                PathBuf::from(&user_profile)
                    .join("Desktop")
                    .to_string_lossy()
                    .to_string(),
            );
            result.insert(
                "DOCUMENTS".to_string(),
                PathBuf::from(&user_profile)
                    .join("Documents")
                    .to_string_lossy()
                    .to_string(),
            );
            result.insert(
                "DOWNLOADS".to_string(),
                PathBuf::from(&user_profile)
                    .join("Downloads")
                    .to_string_lossy()
                    .to_string(),
            );
            result
        }
        "linux" | "wsl" => {
            // Linux/WSL: Check XDG Base Directory specification first
            let mut result = HashMap::new();
            result.insert("HOME".to_string(), home_dir);
            result.insert(
                "DESKTOP".to_string(),
                get_env_var(env, "XDG_DESKTOP_DIR").unwrap_or_else(|| defaults["DESKTOP"].clone()),
            );
            result.insert(
                "DOCUMENTS".to_string(),
                get_env_var(env, "XDG_DOCUMENTS_DIR")
                    .unwrap_or_else(|| defaults["DOCUMENTS"].clone()),
            );
            result.insert(
                "DOWNLOADS".to_string(),
                get_env_var(env, "XDG_DOWNLOAD_DIR")
                    .unwrap_or_else(|| defaults["DOWNLOADS"].clone()),
            );
            result
        }
        "macos" | _ => {
            // macOS and unknown platforms use standard paths
            defaults
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_directories() {
        let dirs = get_system_directories(None);
        assert!(dirs.contains_key("HOME"));
        assert!(dirs.contains_key("DESKTOP"));
        assert!(dirs.contains_key("DOCUMENTS"));
        assert!(dirs.contains_key("DOWNLOADS"));
    }

    #[test]
    fn test_custom_home() {
        let mut options = SystemDirectoriesOptions::default();
        options.homedir = Some("/custom/home".to_string());
        let dirs = get_system_directories(Some(options));
        assert_eq!(dirs.get("HOME"), Some(&"/custom/home".to_string()));
    }
}
