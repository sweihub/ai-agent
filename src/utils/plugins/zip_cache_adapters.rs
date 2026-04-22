// Source: ~/claudecode/openclaudecode/src/utils/plugins/zipCacheAdapters.ts
#![allow(dead_code)]

use std::collections::HashMap;

use super::schemas::{KnownMarketplace, KnownMarketplacesFile};
use super::types::PluginMarketplace;
use super::zip_cache::{
    atomic_write_to_zip_cache, get_marketplace_json_relative_path, get_plugin_zip_cache_path,
    get_zip_cache_known_marketplaces_path,
};

/// Read known_marketplaces.json from the zip cache.
pub async fn read_zip_cache_known_marketplaces() -> KnownMarketplacesFile {
    let path = match get_zip_cache_known_marketplaces_path() {
        Ok(p) => p,
        Err(_) => return HashMap::new(),
    };

    match tokio::fs::read_to_string(&path).await {
        Ok(content) => match serde_json::from_str::<KnownMarketplacesFile>(&content) {
            Ok(parsed) => parsed,
            Err(e) => {
                log::debug!("Invalid known_marketplaces.json in zip cache: {}", e);
                HashMap::new()
            }
        },
        Err(_) => HashMap::new(),
    }
}

/// Write known_marketplaces.json to the zip cache atomically.
pub async fn write_zip_cache_known_marketplaces(
    data: &KnownMarketplacesFile,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path = get_zip_cache_known_marketplaces_path()?;
    let content = serde_json::to_string_pretty(data)?;
    atomic_write_to_zip_cache(&path, content.as_bytes()).await
}

/// Read a marketplace JSON file from the zip cache.
pub async fn read_marketplace_json(marketplace_name: &str) -> Option<PluginMarketplace> {
    let zip_cache_path = get_plugin_zip_cache_path()?;
    let rel_path = get_marketplace_json_relative_path(marketplace_name);
    let full_path = format!("{}/{}", zip_cache_path, rel_path);

    match tokio::fs::read_to_string(&full_path).await {
        Ok(content) => match serde_json::from_str::<PluginMarketplace>(&content) {
            Ok(parsed) => Some(parsed),
            Err(e) => {
                log::debug!("Invalid marketplace JSON for {}: {}", marketplace_name, e);
                None
            }
        },
        Err(_) => None,
    }
}

/// Save a marketplace JSON to the zip cache from its install location.
pub async fn save_marketplace_json_to_zip_cache(
    marketplace_name: &str,
    install_location: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let zip_cache_path = match get_plugin_zip_cache_path() {
        Some(p) => p,
        None => return Ok(()),
    };

    if let Some(content) = read_marketplace_json_content(install_location).await {
        let rel_path = get_marketplace_json_relative_path(marketplace_name);
        let full_path = format!("{}/{}", zip_cache_path, rel_path);
        atomic_write_to_zip_cache(&full_path, content.as_bytes()).await?;
    }

    Ok(())
}

/// Read marketplace.json content from a cloned marketplace directory or file.
async fn read_marketplace_json_content(dir: &str) -> Option<String> {
    let candidates = [
        format!("{}/.claude-plugin/marketplace.json", dir),
        format!("{}/marketplace.json", dir),
        dir.to_string(), // For URL sources, installLocation IS the marketplace JSON file
    ];

    for candidate in candidates {
        if let Ok(content) = tokio::fs::read_to_string(&candidate).await {
            return Some(content);
        }
    }

    None
}

/// Sync marketplace data to zip cache for offline access.
pub async fn sync_marketplaces_to_zip_cache() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let known_marketplaces =
        match super::marketplace_manager::load_known_marketplaces_config_safe().await {
            Ok(m) => m,
            Err(e) => {
                log::debug!("Failed to load known marketplaces config: {}", e);
                return Ok(());
            }
        };

    // Save marketplace JSONs to zip cache
    for (name, entry) in &known_marketplaces {
        let install_location = &entry.install_location;
        if !install_location.is_empty() {
            if let Err(e) = save_marketplace_json_to_zip_cache(name, install_location).await {
                log::debug!("Failed to save marketplace JSON for {}: {}", name, e);
            }
        }
    }

    // Merge with previously cached data
    let zip_cache_known_marketplaces = read_zip_cache_known_marketplaces().await;
    let mut merged = zip_cache_known_marketplaces;
    merged.extend(known_marketplaces);

    write_zip_cache_known_marketplaces(&merged).await?;

    Ok(())
}
