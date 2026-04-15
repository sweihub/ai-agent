// Source: ~/claudecode/openclaudecode/src/utils/plugins/mcpPluginIntegration.rs
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::Path;

use super::mcpb_handler::{is_mcpb_source, load_mcpb_file, McpServerConfig};
use super::plugin_directories::get_plugin_data_dir;
use super::plugin_options_storage::{get_plugin_storage_id, load_plugin_options, substitute_plugin_variables, substitute_user_config_variables};
use crate::plugin::types::{LoadedPlugin, PluginError};

/// Scoped MCP server configuration.
#[derive(Clone, Debug)]
pub struct ScopedMcpServerConfig {
    pub config: McpServerConfig,
    pub scope: String,
    pub plugin_source: String,
}

/// Load MCP servers from a plugin's manifest.
pub async fn load_plugin_mcp_servers(
    plugin: &LoadedPlugin,
    errors: &mut Vec<PluginError>,
) -> Result<Option<HashMap<String, McpServerConfig>>, String> {
    let mut servers: HashMap<String, McpServerConfig> = HashMap::new();

    // Check for .mcp.json in plugin directory first
    let default_mcp_path = Path::new(&plugin.path).join(".mcp.json");
    if default_mcp_path.exists() {
        if let Ok(Some(default_servers)) = load_mcp_servers_from_file(&plugin.path, ".mcp.json").await {
            servers.extend(default_servers);
        }
    }

    // Handle manifest mcp_servers if present
    if let Some(ref mcp_servers_spec) = plugin.manifest.mcp_servers {
        if let Some(s) = mcp_servers_spec.as_str() {
            if is_mcpb_source(s) {
                if let Ok(Some(mcpb_servers)) =
                    load_mcp_servers_from_mcpb(plugin, s, errors).await
                {
                    servers.extend(mcpb_servers);
                }
            } else {
                if let Ok(Some(json_servers)) = load_mcp_servers_from_file(&plugin.path, s).await {
                    servers.extend(json_servers);
                }
            }
        } else if let Some(arr) = mcp_servers_spec.as_array() {
            for spec in arr {
                if let Some(s) = spec.as_str() {
                    if is_mcpb_source(s) {
                        if let Ok(Some(mcpb_servers)) =
                            load_mcp_servers_from_mcpb(plugin, s, errors).await
                        {
                            servers.extend(mcpb_servers);
                        }
                    } else if let Ok(Some(json_servers)) =
                        load_mcp_servers_from_file(&plugin.path, s).await
                    {
                        servers.extend(json_servers);
                    }
                } else if let Some(_obj) = spec.as_object() {
                    // Inline MCP server configs - stub
                }
            }
        } else if let Some(_obj) = mcp_servers_spec.as_object() {
            // Direct MCP server configs - stub
        }
    }

    if servers.is_empty() {
        Ok(None)
    } else {
        Ok(Some(servers))
    }
}

/// Load MCP servers from an MCPB file.
async fn load_mcp_servers_from_mcpb(
    plugin: &LoadedPlugin,
    mcpb_path: &str,
    errors: &mut Vec<PluginError>,
) -> Result<Option<HashMap<String, McpServerConfig>>, String> {
    log::debug!("Loading MCP servers from MCPB: {}", mcpb_path);

    let plugin_id = &plugin.source;

    match load_mcpb_file(mcpb_path, Path::new(&plugin.path), plugin_id).await {
        Ok(Ok(result)) => {
            let server_name = result.manifest.name;
            log::debug!(
                "Loaded MCP server \"{}\" from MCPB",
                server_name
            );
            Ok(Some(result.mcp_config))
        }
        Ok(Err(_needs_config)) => {
            log::debug!(
                "MCPB {} requires user configuration",
                mcpb_path
            );
            Ok(None)
        }
        Err(e) => {
            log::debug!("Failed to load MCPB {}: {}", mcpb_path, e);
            errors.push(PluginError::McpbExtractFailed {
                source: plugin_id.clone(),
                plugin: plugin.name.clone(),
                mcpb_path: mcpb_path.to_string(),
                reason: e.to_string(),
            });
            Ok(None)
        }
    }
}

/// Load MCP servers from a JSON file within a plugin.
async fn load_mcp_servers_from_file(
    plugin_path: &str,
    relative_path: &str,
) -> Result<Option<HashMap<String, McpServerConfig>>, String> {
    let file_path = Path::new(plugin_path).join(relative_path);

    let content = match tokio::fs::read_to_string(&file_path).await {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            log::debug!("Failed to load MCP servers from {:?}: {}", file_path, e);
            return Ok(None);
        }
    };

    match serde_json::from_str::<HashMap<String, McpServerConfig>>(&content) {
        Ok(servers) => Ok(Some(servers)),
        Err(e) => {
            log::debug!("Invalid MCP server config in {:?}: {}", file_path, e);
            Ok(None)
        }
    }
}

/// Add plugin scope to MCP server configs.
pub fn add_plugin_scope_to_servers(
    servers: HashMap<String, McpServerConfig>,
    plugin_name: &str,
    plugin_source: &str,
) -> HashMap<String, ScopedMcpServerConfig> {
    servers
        .into_iter()
        .map(|(name, config)| {
            let scoped_name = format!("plugin:{}:{}", plugin_name, name);
            (
                scoped_name,
                ScopedMcpServerConfig {
                    config,
                    scope: "dynamic".to_string(),
                    plugin_source: plugin_source.to_string(),
                },
            )
        })
        .collect()
}

/// Resolve environment variables for plugin MCP servers.
pub fn resolve_plugin_mcp_environment(
    config: &McpServerConfig,
    plugin: &LoadedPlugin,
    user_config: Option<&HashMap<String, serde_json::Value>>,
) -> McpServerConfig {
    let resolve_value = |value: &str| -> String {
        let mut resolved = substitute_plugin_variables(value, &plugin.path, &plugin.source);

        if let Some(uc) = user_config {
            resolved = substitute_user_config_variables(&resolved, uc);
        }

        expand_env_vars_in_string(&resolved)
    };

    let mut resolved = config.clone();

    if let Some(ref cmd) = config.command {
        resolved.command = Some(resolve_value(cmd));
    }

    if let Some(ref args) = config.args {
        resolved.args = Some(args.iter().map(|a| resolve_value(a)).collect());
    }

    let mut resolved_env: HashMap<String, String> = HashMap::new();
    resolved_env.insert("CLAUDE_PLUGIN_ROOT".to_string(), plugin.path.clone());
    resolved_env.insert(
        "CLAUDE_PLUGIN_DATA".to_string(),
        get_plugin_data_dir(&plugin.source),
    );

    if let Some(ref env) = config.env {
        for (key, value) in env {
            if key != "CLAUDE_PLUGIN_ROOT" && key != "CLAUDE_PLUGIN_DATA" {
                resolved_env.insert(key.clone(), resolve_value(value));
            }
        }
    }
    resolved.env = Some(resolved_env);

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

/// Extract all MCP servers from loaded plugins.
pub async fn extract_mcp_servers_from_plugins(
    plugins: &[LoadedPlugin],
    errors: &mut Vec<PluginError>,
) -> HashMap<String, ScopedMcpServerConfig> {
    let mut all_servers = HashMap::new();

    for plugin in plugins {
        if !plugin.enabled.unwrap_or(true) {
            continue;
        }

        match load_plugin_mcp_servers(plugin, errors).await {
            Ok(Some(servers)) => {
                let scoped_servers =
                    add_plugin_scope_to_servers(servers, &plugin.name, &plugin.source);
                log::debug!(
                    "Loaded {} MCP servers from plugin {}",
                    scoped_servers.len(),
                    plugin.name
                );
                all_servers.extend(scoped_servers);
            }
            _ => {}
        }
    }

    all_servers
}
