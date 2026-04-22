//! Plugin loader - cache-only loading functions
//!
//! Ported from ~/claudecode/openclaudecode/src/utils/plugins/marketplaceManager.ts
//!
//! This module provides cache-only loading functions for marketplaces and plugins.
//! These functions are used for startup paths that should never block on network.

use std::fs;
use std::path::PathBuf;

use super::types::{KnownMarketplacesFile, PluginMarketplace, PluginMarketplaceEntry};
use crate::plugin::types::{LoadedPlugin, PluginError, PluginLoadResult};
use crate::utils::config::get_global_config_path;

/// Get the path to the known marketplaces config file
fn get_known_marketplaces_file() -> PathBuf {
    get_global_config_path().join("known_marketplaces.json")
}

/// Read a cached marketplace from disk
async fn read_cached_marketplace(install_location: &str) -> Option<PluginMarketplace> {
    let marketplace_path = PathBuf::from(install_location)
        .join(".ai-plugin")
        .join("marketplace.json");

    if !marketplace_path.exists() {
        return None;
    }

    match fs::read_to_string(&marketplace_path) {
        Ok(content) => match serde_json::from_str::<PluginMarketplace>(&content) {
            Ok(marketplace) => Some(marketplace),
            Err(e) => {
                eprintln!(
                    "Failed to parse marketplace at {}: {}",
                    marketplace_path.display(),
                    e
                );
                None
            }
        },
        Err(e) => {
            eprintln!(
                "Failed to read marketplace at {}: {}",
                marketplace_path.display(),
                e
            );
            None
        }
    }
}

/// Load known marketplaces config from cache
async fn load_known_marketplaces_config() -> Option<KnownMarketplacesFile> {
    let config_file = get_known_marketplaces_file();

    if !config_file.exists() {
        return None;
    }

    match fs::read_to_string(&config_file) {
        Ok(content) => match serde_json::from_str::<KnownMarketplacesFile>(&content) {
            Ok(config) => Some(config),
            Err(e) => {
                eprintln!("Failed to parse known marketplaces: {}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("Failed to read known marketplaces file: {}", e);
            None
        }
    }
}

/// Parse plugin identifier into name and marketplace
///
/// # Arguments
/// * `plugin_id` - Plugin ID in format "name@marketplace"
///
/// # Returns
/// Tuple of (name, marketplace) or (None, None) if invalid
pub fn parse_plugin_identifier(plugin_id: &str) -> (Option<String>, Option<String>) {
    if let Some(at_pos) = plugin_id.rfind('@') {
        let name = plugin_id[..at_pos].to_string();
        let marketplace = plugin_id[at_pos + 1..].to_string();
        if !name.is_empty() && !marketplace.is_empty() {
            return (Some(name), Some(marketplace));
        }
    }
    (None, None)
}

/// Get a marketplace by name from cache only (no network)
///
/// Use this for startup paths that should never block on network.
///
/// # Arguments
/// * `name` - Marketplace name
///
/// # Returns
/// The marketplace or null if not found/cache missing
pub async fn get_marketplace_cache_only(name: &str) -> Option<PluginMarketplace> {
    let config_file = get_known_marketplaces_file();

    if !config_file.exists() {
        return None;
    }

    match fs::read_to_string(&config_file) {
        Ok(content) => {
            match serde_json::from_str::<KnownMarketplacesFile>(&content) {
                Ok(config) => {
                    if let Some(entry) = config.get(name) {
                        // Try to read the marketplace from the install location
                        if let Some(marketplace) =
                            read_cached_marketplace(&entry.install_location).await
                        {
                            return Some(marketplace);
                        }
                    }
                    None
                }
                Err(e) => {
                    eprintln!("Failed to parse known marketplaces config: {}", e);
                    None
                }
            }
        }
        Err(_) => None,
    }
}

/// Get a plugin by ID from cache only (no network)
///
/// # Arguments
/// * `plugin_id` - Plugin ID in format "name@marketplace"
///
/// # Returns
/// The plugin entry and marketplace install location, or null if not found/cache missing
pub async fn get_plugin_by_id_cache_only(
    plugin_id: &str,
) -> Option<(PluginMarketplaceEntry, String)> {
    let (plugin_name, marketplace_name) = parse_plugin_identifier(plugin_id);
    let plugin_name = plugin_name?;
    let marketplace_name = marketplace_name?;

    let config_file = get_known_marketplaces_file();

    if !config_file.exists() {
        return None;
    }

    match fs::read_to_string(&config_file) {
        Ok(content) => {
            match serde_json::from_str::<KnownMarketplacesFile>(&content) {
                Ok(config) => {
                    // Get marketplace config
                    let marketplace_config = config.get(&marketplace_name)?;

                    // Get the marketplace itself
                    let marketplace = get_marketplace_cache_only(&marketplace_name).await?;

                    // Find the plugin in the marketplace
                    marketplace
                        .plugins
                        .into_iter()
                        .find(|p| p.name == plugin_name)
                        .map(|entry| (entry, marketplace_config.install_location.clone()))
                }
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

/// Get all known marketplace names
pub async fn get_known_marketplace_names() -> Vec<String> {
    match load_known_marketplaces_config().await {
        Some(config) => config.keys().cloned().collect(),
        None => vec![],
    }
}

/// Load all plugins from cache (no network).
///
/// This is the main entry point for loading plugins at startup.
/// Returns enabled/disabled plugins and any errors encountered.
pub async fn load_all_plugins() -> Result<PluginLoadResult, Box<dyn std::error::Error + Send + Sync>>
{
    // Stub: return empty result - full implementation would load
    // from known_marketplaces config and resolve each plugin
    Ok(PluginLoadResult {
        enabled: vec![],
        disabled: vec![],
        errors: vec![],
    })
}

/// Load all plugins from cache only (strictly no network).
///
/// Same as load_all_plugins but guaranteed never to hit the network.
pub async fn load_all_plugins_cache_only()
-> Result<PluginLoadResult, Box<dyn std::error::Error + Send + Sync>> {
    load_all_plugins().await
}

/// Get the plugin cache root directory.
pub fn get_plugin_cache_path() -> String {
    get_global_config_path()
        .join("plugins")
        .to_string_lossy()
        .to_string()
}

/// Get a versioned cache path for a specific plugin version.
pub fn get_versioned_cache_path(plugin_id: &str, version: &str) -> String {
    let (name, marketplace) = parse_plugin_identifier(plugin_id);
    let marketplace = marketplace.unwrap_or_else(|| "unknown".to_string());
    let name = name.unwrap_or_else(|| plugin_id.to_string());
    get_global_config_path()
        .join("plugins")
        .join(&marketplace)
        .join(&name)
        .join(version)
        .to_string_lossy()
        .to_string()
}

/// Get a versioned zip cache path for a specific plugin version.
pub fn get_versioned_zip_cache_path(plugin_id: &str, version: &str) -> String {
    format!("{}.zip", get_versioned_cache_path(plugin_id, version))
}

/// Cache a plugin and return the cached path.
pub async fn cache_plugin(
    _source: &super::schemas::PluginSource,
    _entry: &PluginMarketplaceEntry,
) -> Result<CachePluginResult, Box<dyn std::error::Error + Send + Sync>> {
    // Stub: in production would clone/fetch plugin source
    Err("cache_plugin not fully implemented - stub".into())
}

/// Result of caching a plugin.
pub struct CachePluginResult {
    pub path: String,
    pub manifest: serde_json::Value,
    pub git_commit_sha: Option<String>,
}

/// Clear the plugin cache for a specific marketplace, or all if None.
pub fn clear_plugin_cache(_marketplace: Option<&str>) {
    // Stub: would clear in-memory caches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plugin_identifier_basic() {
        let (name, marketplace) = parse_plugin_identifier("my-plugin@my-marketplace");
        assert_eq!(name, Some("my-plugin".to_string()));
        assert_eq!(marketplace, Some("my-marketplace".to_string()));
    }

    #[test]
    fn test_parse_plugin_identifier_invalid() {
        let (name, marketplace) = parse_plugin_identifier("invalid");
        assert_eq!(name, None);
        assert_eq!(marketplace, None);
    }

    #[test]
    fn test_parse_plugin_identifier_empty_marketplace() {
        let (name, marketplace) = parse_plugin_identifier("my-plugin@");
        assert_eq!(name, None);
        assert_eq!(marketplace, None);
    }
}
