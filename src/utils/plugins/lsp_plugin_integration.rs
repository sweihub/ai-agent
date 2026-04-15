// Source: ~/claudecode/openclaudecode/src/utils/plugins/lspPluginIntegration.rs
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::plugin_directories::get_plugin_data_dir;
use super::plugin_options_storage::{
    get_plugin_storage_id, load_plugin_options, substitute_plugin_variables,
    substitute_user_config_variables,
};
use crate::plugin::types::{LoadedPlugin, PluginError};

/// LSP server configuration.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LspServerConfig {
    pub command: String,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub workspace_folder: Option<String>,
    #[serde(rename = "extensionToLanguage")]
    pub extension_to_language: Option<HashMap<String, String>>,
}

/// Scoped LSP server configuration.
#[derive(Debug, Clone)]
pub struct ScopedLspServerConfig {
    pub config: LspServerConfig,
    pub scope: String,
    pub source: String,
}

/// Load LSP server configurations from a plugin.
pub async fn load_plugin_lsp_servers(
    plugin: &LoadedPlugin,
    errors: &mut Vec<PluginError>,
) -> Result<Option<HashMap<String, LspServerConfig>>, String> {
    let mut servers: HashMap<String, LspServerConfig> = HashMap::new();

    // Check for .lsp.json file in plugin directory
    let lsp_json_path = Path::new(&plugin.path).join(".lsp.json");
    if lsp_json_path.exists() {
        match tokio::fs::read_to_string(&lsp_json_path).await {
            Ok(content) => {
                match serde_json::from_str::<HashMap<String, LspServerConfig>>(&content) {
                    Ok(parsed) => {
                        servers.extend(parsed);
                    }
                    Err(e) => {
                        log::error!(
                            "LSP config validation failed for .lsp.json in plugin {}: {}",
                            plugin.name,
                            e
                        );
                    }
                }
            }
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                log::error!("Failed to read .lsp.json in plugin {}: {}", plugin.name, e);
            }
            _ => {}
        }
    }

    if servers.is_empty() {
        Ok(None)
    } else {
        Ok(Some(servers))
    }
}

/// Resolve environment variables for plugin LSP servers.
pub fn resolve_plugin_lsp_environment(
    config: &LspServerConfig,
    plugin: &LoadedPlugin,
    user_config: Option<&HashMap<String, serde_json::Value>>,
) -> LspServerConfig {
    let resolve_value = |value: &str| -> String {
        let mut resolved = substitute_plugin_variables(value, &plugin.path, &plugin.source);

        if let Some(uc) = user_config {
            resolved = substitute_user_config_variables(&resolved, uc);
        }

        expand_env_vars_in_string(&resolved)
    };

    let mut resolved = config.clone();

    resolved.command = resolve_value(&config.command);

    if let Some(ref args) = config.args {
        resolved.args = Some(args.iter().map(|a| resolve_value(a)).collect());
    }

    let mut resolved_env: HashMap<String, String> = HashMap::new();
    resolved_env.insert("CLAUDE_PLUGIN_ROOT".to_string(), plugin.path.clone());
    resolved_env.insert("CLAUDE_PLUGIN_DATA".to_string(), get_plugin_data_dir(&plugin.source));

    if let Some(ref env) = config.env {
        for (key, value) in env {
            if key != "CLAUDE_PLUGIN_ROOT" && key != "CLAUDE_PLUGIN_DATA" {
                resolved_env.insert(key.clone(), resolve_value(value));
            }
        }
    }
    resolved.env = Some(resolved_env);

    if let Some(ref wf) = config.workspace_folder {
        resolved.workspace_folder = Some(resolve_value(wf));
    }

    resolved
}

fn expand_env_vars_in_string(value: &str) -> String {
    let mut result = value.to_string();
    let mut start = 0;
    while let Some(pos) = result[start..].find("${") {
        let abs_pos = start + pos;
        if let Some(end) = result[abs_pos + 2..].find('}') {
            let var_name = &result[abs_pos + 2..abs_pos + 2 + end];
            if let Ok(env_value) = std::env::var(var_name) {
                result.replace_range(abs_pos..abs_pos + 2 + end + 1, &env_value);
                start = abs_pos + env_value.len();
                continue;
            }
        }
        start = abs_pos + 2;
    }
    result
}

/// Add plugin scope to LSP server configs.
pub fn add_plugin_scope_to_lsp_servers(
    servers: HashMap<String, LspServerConfig>,
    plugin_name: &str,
) -> HashMap<String, ScopedLspServerConfig> {
    servers
        .into_iter()
        .map(|(name, config)| {
            let scoped_name = format!("plugin:{}:{}", plugin_name, name);
            (
                scoped_name,
                ScopedLspServerConfig {
                    config,
                    scope: "dynamic".to_string(),
                    source: plugin_name.to_string(),
                },
            )
        })
        .collect()
}

/// Extract all LSP servers from loaded plugins.
pub async fn extract_lsp_servers_from_plugins(
    plugins: &[LoadedPlugin],
    errors: &mut Vec<PluginError>,
) -> HashMap<String, ScopedLspServerConfig> {
    let mut all_servers = HashMap::new();

    for plugin in plugins {
        if !plugin.enabled.unwrap_or(true) {
            continue;
        }

        if let Ok(Some(servers)) = load_plugin_lsp_servers(plugin, errors).await {
            let scoped_servers = add_plugin_scope_to_lsp_servers(servers, &plugin.name);
            log::debug!(
                "Loaded {} LSP servers from plugin {}",
                scoped_servers.len(),
                plugin.name
            );
            all_servers.extend(scoped_servers);
        }
    }

    all_servers
}
