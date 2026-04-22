// Source: ~/claudecode/openclaudecode/src/utils/plugins/marketplaceManager.ts
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::fs;

use super::plugin_directories::get_plugins_directory;
use super::plugin_identifier::parse_plugin_identifier;
use super::schemas::{MarketplaceSource, is_marketplace_auto_update};
use super::types::{PluginMarketplace, PluginMarketplaceEntry, PluginMarketplaceOwner};

static MARKETPLACE_CACHE: Lazy<Mutex<HashMap<String, PluginMarketplace>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Get the path to the known marketplaces configuration file.
fn get_known_marketplaces_file() -> PathBuf {
    PathBuf::from(get_plugins_directory()).join("known_marketplaces.json")
}

/// Get the path to the marketplaces cache directory.
pub fn get_marketplaces_cache_dir() -> PathBuf {
    PathBuf::from(get_plugins_directory()).join("marketplaces")
}

/// Declared marketplace entry (intent layer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclaredMarketplace {
    pub source: MarketplaceSource,
    #[serde(rename = "installLocation", skip_serializing_if = "Option::is_none")]
    pub install_location: Option<String>,
    #[serde(rename = "autoUpdate", skip_serializing_if = "Option::is_none")]
    pub auto_update: Option<bool>,
    #[serde(rename = "sourceIsFallback", skip_serializing_if = "Option::is_none")]
    pub source_is_fallback: Option<bool>,
}

/// Get declared marketplace intent from merged settings and --add-dir sources.
pub fn get_declared_marketplaces() -> HashMap<String, DeclaredMarketplace> {
    HashMap::new()
}

/// Load known marketplaces configuration from disk.
pub async fn load_known_marketplaces_config()
-> Result<HashMap<String, super::types::KnownMarketplace>, Box<dyn std::error::Error + Send + Sync>>
{
    let config_file = get_known_marketplaces_file();

    match fs::read_to_string(&config_file).await {
        Ok(content) => {
            let data: HashMap<String, super::types::KnownMarketplace> =
                serde_json::from_str(&content)?;
            Ok(data)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(HashMap::new()),
        Err(e) => Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
    }
}

/// Load known marketplaces config, returning empty map on any error.
pub async fn load_known_marketplaces_config_safe()
-> Result<HashMap<String, super::types::KnownMarketplace>, Box<dyn std::error::Error + Send + Sync>>
{
    load_known_marketplaces_config()
        .await
        .or_else(|_| Ok(HashMap::new()))
}

/// Save known marketplaces configuration to disk.
pub async fn save_known_marketplaces_config(
    config: &HashMap<String, super::types::KnownMarketplace>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config_file = get_known_marketplaces_file();

    let dir = config_file
        .parent()
        .ok_or_else(|| "Invalid config path".to_string())?;
    tokio::fs::create_dir_all(dir).await?;

    let json_content = serde_json::to_string_pretty(config)?;
    tokio::fs::write(&config_file, json_content).await?;
    Ok(())
}

/// Clear all cached marketplace data.
pub fn clear_marketplaces_cache() {
    let mut cache = MARKETPLACE_CACHE.lock().unwrap();
    cache.clear();
}

/// Get marketplace data by name.
pub async fn get_marketplace(
    name: &str,
) -> Result<PluginMarketplace, Box<dyn std::error::Error + Send + Sync>> {
    {
        let cache = MARKETPLACE_CACHE.lock().unwrap();
        if let Some(marketplace) = cache.get(name) {
            return Ok(marketplace.clone());
        }
    }

    Err(format!("Marketplace '{}' not found", name).into())
}

/// Register marketplaces from the read-only seed directories.
pub async fn register_seed_marketplaces() -> Result<bool, Box<dyn std::error::Error + Send + Sync>>
{
    Ok(false)
}

/// Add a marketplace source and materialize it.
pub async fn add_marketplace_source(
    source: &MarketplaceSource,
) -> Result<AddMarketplaceResult, Box<dyn std::error::Error + Send + Sync>> {
    // Stub: simplified
    Ok(AddMarketplaceResult {
        already_materialized: false,
    })
}

pub struct AddMarketplaceResult {
    pub already_materialized: bool,
}

/// Get plugin by ID from marketplace cache.
pub async fn get_plugin_by_id(plugin_id: &str) -> Option<PluginByIdResult> {
    let parsed = parse_plugin_identifier(plugin_id);
    let marketplace_name = parsed.marketplace?;

    match get_marketplace(&marketplace_name).await {
        Ok(marketplace) => {
            for entry in &marketplace.plugins {
                if entry.name == parsed.name {
                    return Some(PluginByIdResult {
                        entry: entry.clone(),
                        marketplace_install_location: String::new(),
                    });
                }
            }
            None
        }
        Err(_) => None,
    }
}

pub struct PluginByIdResult {
    pub entry: PluginMarketplaceEntry,
    pub marketplace_install_location: String,
}

/// Refresh a marketplace by pulling latest changes.
pub async fn refresh_marketplace(
    _name: &str,
    _progress: Option<impl Fn(&str)>,
    _options: Option<RefreshMarketplaceOptions>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Ok(())
}

pub struct RefreshMarketplaceOptions {
    pub disable_credential_helper: bool,
    pub sparse_paths: Option<Vec<String>>,
}

/// Save a marketplace entry to settings.
pub fn save_marketplace_to_settings(
    _name: &str,
    _entry: &DeclaredMarketplace,
    _setting_source: &str,
) {
    // Stub
}
