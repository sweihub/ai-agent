// Source: ~/claudecode/openclaudecode/src/utils/plugins/pluginDirectories.ts
#![allow(dead_code)]

use std::path::PathBuf;

use once_cell::sync::OnceCell;

const PLUGINS_DIR: &str = "plugins";
const COWORK_PLUGINS_DIR: &str = "cowork_plugins";

static PLUGINS_DIR_NAME: OnceCell<String> = OnceCell::new();

/// Get the plugins directory name based on current mode.
fn get_plugins_dir_name() -> &'static str {
    PLUGINS_DIR_NAME.get_or_init(|| {
        // Stub: bootstrap::state module not available
        // Check env var for cowork plugins
        if std::env::var("CLAUDE_CODE_USE_COWORK_PLUGINS")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
        {
            return COWORK_PLUGINS_DIR.to_string();
        }
        PLUGINS_DIR.to_string()
    })
}

/// Get the full path to the plugins directory.
pub fn get_plugins_directory() -> String {
    // Check for env override
    if let Ok(env_override) = std::env::var("CLAUDE_CODE_PLUGIN_CACHE_DIR") {
        return expand_tilde(&env_override);
    }

    let config_home = crate::utils::env_utils::get_claude_config_home_dir();
    format!("{}/{}", config_home, get_plugins_dir_name())
}

/// Get the read-only plugin seed directories.
pub fn get_plugin_seed_dirs() -> Vec<PathBuf> {
    let raw = match std::env::var("CLAUDE_CODE_PLUGIN_SEED_DIR") {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let delimiter = if cfg!(windows) { ";" } else { ":" };

    raw.split(delimiter)
        .filter(|s| !s.is_empty())
        .map(|s| PathBuf::from(expand_tilde(s)))
        .collect()
}

/// Sanitize a plugin ID for use in file paths.
fn sanitize_plugin_id(plugin_id: &str) -> String {
    plugin_id
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// Get the path for a plugin's data directory (without creating it).
pub fn plugin_data_dir_path(plugin_id: &str) -> PathBuf {
    PathBuf::from(get_plugins_directory())
        .join("data")
        .join(sanitize_plugin_id(plugin_id))
}

/// Get or create the persistent per-plugin data directory.
pub fn get_plugin_data_dir(plugin_id: &str) -> String {
    let dir = plugin_data_dir_path(plugin_id);
    std::fs::create_dir_all(&dir).ok();
    dir.to_string_lossy().to_string()
}

/// Delete a plugin's data directory (best-effort).
pub async fn delete_plugin_data_dir(plugin_id: &str) {
    let dir = plugin_data_dir_path(plugin_id);
    if let Err(e) = tokio::fs::remove_dir_all(&dir).await {
        log::debug!("Failed to delete plugin data dir {:?}: {}", dir, e);
    }
}

/// Expand ~ in a path to the home directory.
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") || path == "~" {
        if let Some(home) = dirs::home_dir() {
            return format!("{}{}", home.display(), &path[1..]);
        }
    }
    path.to_string()
}
