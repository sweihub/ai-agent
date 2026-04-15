// Source: ~/claudecode/openclaudecode/src/utils/plugins/lspRecommendation.rs
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::installed_plugins_manager::is_plugin_installed;
use super::marketplace_manager::{get_marketplace, load_known_marketplaces_config_safe};
use super::schemas::allowed_official_marketplace_names;

const MAX_IGNORED_COUNT: u32 = 5;

/// LSP plugin recommendation returned to the caller.
pub struct LspPluginRecommendation {
    pub plugin_id: String,
    pub plugin_name: String,
    pub marketplace_name: String,
    pub description: Option<String>,
    pub is_official: bool,
    pub extensions: Vec<String>,
    pub command: String,
}

/// Internal type for LSP info extracted from plugin manifest.
struct LspInfo {
    extensions: HashSet<String>,
    command: String,
}

/// Extract LSP info from inline lsp_servers config.
fn extract_lsp_info_from_manifest(lsp_servers: &serde_json::Value) -> Option<LspInfo> {
    if lsp_servers.is_string() {
        return None;
    }

    let mut extensions = HashSet::new();
    let mut command: Option<String> = None;

    if let Some(obj) = lsp_servers.as_object() {
        for (_server_name, config) in obj {
            if let Some(cmd) = config.get("command").and_then(|c| c.as_str()) {
                if command.is_none() {
                    command = Some(cmd.to_string());
                }
            }

            if let Some(ext_mapping) = config.get("extensionToLanguage").and_then(|e| e.as_object())
            {
                for ext in ext_mapping.keys() {
                    extensions.insert(ext.to_lowercase());
                }
            }
        }
    }

    if let Some(cmd) = command {
        if !extensions.is_empty() {
            return Some(LspInfo {
                extensions,
                command: cmd,
            });
        }
    }

    None
}

/// Get all LSP plugins from all installed marketplaces.
async fn get_lsp_plugins_from_marketplaces() -> HashMap<String, LspPluginInfo> {
    let mut result = HashMap::new();

    match load_known_marketplaces_config_safe().await {
        Ok(config) => {
            for marketplace_name in config.keys() {
                match get_marketplace(marketplace_name).await {
                    Ok(marketplace) => {
                        let is_official = is_official_marketplace(marketplace_name);

                        for entry in &marketplace.plugins {
                            if let Some(ref lsp_servers) = entry.lsp_servers {
                                if let Some(lsp_info) =
                                    extract_lsp_info_from_manifest(lsp_servers)
                                {
                                    let plugin_id = format!("{}@{}", entry.name, marketplace_name);
                                    result.insert(
                                        plugin_id,
                                        LspPluginInfo {
                                            entry: entry.clone(),
                                            marketplace_name: marketplace_name.clone(),
                                            extensions: lsp_info.extensions,
                                            command: lsp_info.command,
                                            is_official,
                                        },
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::debug!(
                            "[lsp_recommendation] Failed to load marketplace {}: {}",
                            marketplace_name,
                            e
                        );
                    }
                }
            }
        }
        Err(e) => {
            log::debug!(
                "[lsp_recommendation] Failed to load marketplaces config: {}",
                e
            );
        }
    }

    result
}

struct LspPluginInfo {
    entry: super::types::PluginMarketplaceEntry,
    marketplace_name: String,
    extensions: HashSet<String>,
    command: String,
    is_official: bool,
}

fn is_official_marketplace(name: &str) -> bool {
    allowed_official_marketplace_names().iter().any(|&s| s == name.to_lowercase())
}

/// Find matching LSP plugins for a file path.
pub async fn get_matching_lsp_plugins(file_path: &str) -> Vec<LspPluginRecommendation> {
    if is_lsp_recommendations_disabled() {
        return Vec::new();
    }

    let ext = Path::new(file_path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if ext.is_empty() {
        return Vec::new();
    }

    let all_lsp_plugins = get_lsp_plugins_from_marketplaces().await;

    let mut matching: Vec<LspPluginRecommendation> = Vec::new();

    for (plugin_id, info) in &all_lsp_plugins {
        if !info.extensions.contains(&ext) {
            continue;
        }

        if is_plugin_installed(plugin_id) {
            continue;
        }

        matching.push(LspPluginRecommendation {
            plugin_id: plugin_id.clone(),
            plugin_name: info.entry.name.clone(),
            marketplace_name: info.marketplace_name.clone(),
            description: info.entry.description.clone(),
            is_official: info.is_official,
            extensions: info.extensions.iter().cloned().collect(),
            command: info.command.clone(),
        });
    }

    matching.sort_by(|a, b| {
        match (a.is_official, b.is_official) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    });

    matching
}

/// Add a plugin to the "never suggest" list.
pub fn add_to_never_suggest(_plugin_id: &str) {
    // Stub
}

/// Increment the ignored recommendation count.
pub fn _increment_ignored_count() {
    // Stub
}

/// Check if LSP recommendations are disabled.
pub fn is_lsp_recommendations_disabled() -> bool {
    false
}

/// Reset the ignored count.
pub fn _reset_ignored_count() {
    // Stub
}
