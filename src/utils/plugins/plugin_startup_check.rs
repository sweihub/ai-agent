// Source: ~/claudecode/openclaudecode/src/utils/plugins/pluginStartupCheck.ts
#![allow(dead_code)]

use super::installed_plugins_manager::get_in_memory_installed_plugins;
use super::marketplace_manager::get_plugin_by_id;
use super::plugin_identifier::parse_plugin_identifier;

/// Check for enabled plugins across all settings sources.
pub async fn check_enabled_plugins() -> Vec<String> {
    // Stub: settings module not available
    Vec::new()
}

/// Find plugins that are enabled but not installed.
pub async fn find_missing_plugins(enabled_plugins: &[String]) -> Vec<String> {
    let installed_plugins = match get_installed_plugin_ids() {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };

    let not_installed: Vec<_> = enabled_plugins
        .iter()
        .filter(|id| !installed_plugins.iter().any(|s| s.as_str() == id.as_str()))
        .cloned()
        .collect();

    let mut missing = Vec::new();
    for plugin_id in not_installed {
        match get_plugin_by_id(&plugin_id).await {
            Some(_) => missing.push(plugin_id),
            None => log::debug!("Plugin {} not found in any marketplace", plugin_id),
        }
    }

    missing
}

/// Get the list of currently installed plugins.
pub async fn get_installed_plugins() -> Result<Vec<String>, String> {
    let v2_data = get_in_memory_installed_plugins();
    let installed: Vec<_> = v2_data.plugins.keys().cloned().collect();
    log::debug!("Found {} installed plugins", installed.len());
    Ok(installed)
}

fn get_installed_plugin_ids() -> Result<Vec<String>, String> {
    let v2_data = get_in_memory_installed_plugins();
    Ok(v2_data.plugins.keys().cloned().collect())
}
