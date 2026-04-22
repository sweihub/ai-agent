// Source: ~/claudecode/openclaudecode/src/utils/plugins/zipCache.ts
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use super::schemas::MarketplaceSource;

/// Check if the plugin zip cache mode is enabled.
pub fn is_plugin_zip_cache_enabled() -> bool {
    std::env::var("CLAUDE_CODE_PLUGIN_USE_ZIP_CACHE")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Get the path to the zip cache directory.
pub fn get_plugin_zip_cache_path() -> Option<String> {
    if !is_plugin_zip_cache_enabled() {
        return None;
    }
    std::env::var("CLAUDE_CODE_PLUGIN_CACHE_DIR")
        .ok()
        .map(|dir| {
            if dir.starts_with("~/") {
                dirs::home_dir()
                    .map(|h| format!("{}{}", h.display(), &dir[1..]))
                    .unwrap_or(dir)
            } else {
                dir
            }
        })
}

/// Get the path to known_marketplaces.json in the zip cache.
pub fn get_zip_cache_known_marketplaces_path()
-> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let cache_path =
        get_plugin_zip_cache_path().ok_or_else(|| "Plugin zip cache is not enabled".to_string())?;
    Ok(PathBuf::from(cache_path)
        .join("known_marketplaces.json")
        .to_string_lossy()
        .to_string())
}

/// Get the path to installed_plugins.json in the zip cache.
pub fn get_zip_cache_installed_plugins_path()
-> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let cache_path =
        get_plugin_zip_cache_path().ok_or_else(|| "Plugin zip cache is not enabled".to_string())?;
    Ok(PathBuf::from(cache_path)
        .join("installed_plugins.json")
        .to_string_lossy()
        .to_string())
}

/// Get the marketplaces directory within the zip cache.
pub fn get_zip_cache_marketplaces_dir() -> Result<String, Box<dyn std::error::Error + Send + Sync>>
{
    let cache_path =
        get_plugin_zip_cache_path().ok_or_else(|| "Plugin zip cache is not enabled".to_string())?;
    Ok(PathBuf::from(cache_path)
        .join("marketplaces")
        .to_string_lossy()
        .to_string())
}

/// Get the plugins directory within the zip cache.
pub fn get_zip_cache_plugins_dir() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let cache_path =
        get_plugin_zip_cache_path().ok_or_else(|| "Plugin zip cache is not enabled".to_string())?;
    Ok(PathBuf::from(cache_path)
        .join("plugins")
        .to_string_lossy()
        .to_string())
}

/// Session plugin cache: a temp directory on local disk.
static SESSION_PLUGIN_CACHE_PATH: once_cell::sync::Lazy<std::sync::Mutex<Option<String>>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(None));

/// Get or create the session plugin cache directory.
pub async fn get_session_plugin_cache_path()
-> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    {
        let guard = SESSION_PLUGIN_CACHE_PATH.lock().unwrap();
        if let Some(ref path) = *guard {
            return Ok(path.clone());
        }
    }

    let suffix = hex::encode(rand::random::<[u8; 8]>());
    let dir = PathBuf::from(std::env::temp_dir()).join(format!("claude-plugin-session-{}", suffix));

    tokio::fs::create_dir_all(&dir).await?;

    let path_str = dir.to_string_lossy().to_string();
    {
        let mut guard = SESSION_PLUGIN_CACHE_PATH.lock().unwrap();
        *guard = Some(path_str.clone());
    }

    log::debug!("Created session plugin cache at {}", path_str);
    Ok(path_str)
}

/// Clean up the session plugin cache directory.
pub async fn cleanup_session_plugin_cache() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let path = {
        let mut guard = SESSION_PLUGIN_CACHE_PATH.lock().unwrap();
        guard.take()
    };
    if let Some(path) = path {
        if let Err(e) = tokio::fs::remove_dir_all(&path).await {
            log::debug!("Failed to clean up session plugin cache at {}: {}", path, e);
        } else {
            log::debug!("Cleaned up session plugin cache at {}", path);
        }
    }
    Ok(())
}

/// Write data to a file in the zip cache atomically.
pub async fn atomic_write_to_zip_cache(
    target_path: &str,
    data: &[u8],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let dir = Path::new(target_path)
        .parent()
        .ok_or_else(|| "Invalid target path".to_string())?;
    tokio::fs::create_dir_all(dir).await?;

    let file_name = Path::new(target_path)
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    let tmp_name = format!(
        ".{}.tmp.{}",
        file_name,
        hex::encode(rand::random::<[u8; 4]>())
    );
    let tmp_path = dir.join(&tmp_name);

    tokio::fs::write(&tmp_path, data).await?;
    tokio::fs::rename(&tmp_path, target_path).await?;

    Ok(())
}

/// Create a ZIP archive from a directory.
pub async fn create_zip_from_directory(_source_dir: &Path) -> Result<Vec<u8>, String> {
    Err("create_zip_from_directory not implemented - add `zip` crate".to_string())
}

/// Extract a ZIP file to a target directory.
pub async fn extract_zip_to_directory(
    _zip_path: &str,
    _target_dir: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err("extract_zip_to_directory not implemented - add `zip` crate".into())
}

/// Convert a plugin directory to a ZIP in-place.
pub async fn convert_directory_to_zip_in_place(
    dir_path: &str,
    zip_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let zip_data = create_zip_from_directory(Path::new(dir_path)).await?;
    atomic_write_to_zip_cache(zip_path, &zip_data).await?;
    let _ = tokio::fs::remove_dir_all(dir_path).await;
    Ok(())
}

/// Get the relative path for a marketplace JSON file within the zip cache.
pub fn get_marketplace_json_relative_path(marketplace_name: &str) -> String {
    let sanitized = marketplace_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>();
    format!("marketplaces/{}.json", sanitized)
}

/// Check if a marketplace source type is supported by zip cache mode.
pub fn is_marketplace_source_supported_by_zip_cache(source: &MarketplaceSource) -> bool {
    matches!(
        source,
        MarketplaceSource::Github { .. }
            | MarketplaceSource::Git { .. }
            | MarketplaceSource::Url { .. }
            | MarketplaceSource::Settings { .. }
    )
}
