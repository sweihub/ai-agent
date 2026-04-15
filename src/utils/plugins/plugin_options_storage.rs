// Source: ~/claudecode/openclaudecode/src/utils/plugins/pluginOptionsStorage.ts
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Mutex;

use once_cell::sync::Lazy;

use super::plugin_directories::get_plugin_data_dir;
use crate::plugin::types::LoadedPlugin;

pub type PluginOptionValues = HashMap<String, serde_json::Value>;
pub type PluginOptionSchema = HashMap<String, serde_json::Value>;

static PLUGIN_OPTIONS_CACHE: Lazy<Mutex<HashMap<String, PluginOptionValues>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Get the canonical storage key for a plugin's options.
pub fn get_plugin_storage_id(plugin: &LoadedPlugin) -> String {
    plugin.source.clone()
}

/// Load saved option values for a plugin.
pub fn load_plugin_options(plugin_id: &str) -> PluginOptionValues {
    {
        let cache = PLUGIN_OPTIONS_CACHE.lock().unwrap();
        if let Some(values) = cache.get(plugin_id) {
            return values.clone();
        }
    }

    let merged = PluginOptionValues::new();

    {
        let mut cache = PLUGIN_OPTIONS_CACHE.lock().unwrap();
        cache.insert(plugin_id.to_string(), merged.clone());
    }

    merged
}

/// Clear the plugin options cache.
pub fn clear_plugin_options_cache() {
    let mut cache = PLUGIN_OPTIONS_CACHE.lock().unwrap();
    cache.clear();
}

/// Save option values.
pub fn save_plugin_options(
    _plugin_id: &str,
    _values: &PluginOptionValues,
    _schema: &PluginOptionSchema,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    clear_plugin_options_cache();
    Ok(())
}

/// Delete all stored option values for a plugin.
pub fn delete_plugin_options(_plugin_id: &str) {
    clear_plugin_options_cache();
}

/// Find option keys whose saved values don't satisfy the schema.
pub fn get_unconfigured_options(_plugin: &LoadedPlugin) -> PluginOptionSchema {
    // Stub: simplified implementation
    HashMap::new()
}

/// Substitute ${CLAUDE_PLUGIN_ROOT} and ${CLAUDE_PLUGIN_DATA} with their paths.
pub fn substitute_plugin_variables(value: &str, plugin_path: &str, source: &str) -> String {
    let normalize = |p: &str| -> String {
        if cfg!(windows) {
            p.replace('\\', "/")
        } else {
            p.to_string()
        }
    };

    let mut out = value.replace("${CLAUDE_PLUGIN_ROOT}", &normalize(plugin_path));

    out = out.replace(
        "${CLAUDE_PLUGIN_DATA}",
        &normalize(&get_plugin_data_dir(source)),
    );

    out
}

/// Substitute ${user_config.KEY} with saved option values.
pub fn substitute_user_config_variables(
    value: &str,
    user_config: &PluginOptionValues,
) -> String {
    let mut result = value.to_string();
    let re = regex::Regex::new(r"\$\{user_config\.([^}]+)\}").unwrap();

    for cap in re.captures_iter(value) {
        let key = &cap[1];
        let full_match = &cap[0];

        if let Some(config_value) = user_config.get(key) {
            result = result.replace(full_match, &config_value.to_string());
        } else {
            log::debug!("Missing user config value for key: {}", key);
        }
    }

    result
}

/// Content-safe variant for skill/agent prose.
pub fn substitute_user_config_in_content(
    content: &str,
    options: &PluginOptionValues,
    schema: &PluginOptionSchema,
) -> String {
    let re = regex::Regex::new(r"\$\{user_config\.([^}]+)\}").unwrap();

    re.replace_all(content, |caps: &regex::Captures| {
        let key = &caps[1];

        if let Some(field_schema) = schema.get(key) {
            let is_sensitive = field_schema
                .get("sensitive")
                .and_then(|s| s.as_bool())
                .unwrap_or(false);

            if is_sensitive {
                return format!("[sensitive option '{}' not available in skill content]", key);
            }
        }

        match options.get(key) {
            Some(value) => value.to_string(),
            None => caps[0].to_string(),
        }
    })
    .to_string()
}
