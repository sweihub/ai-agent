// Source: ~/claudecode/openclaudecode/src/utils/plugins/refresh.ts
#![allow(dead_code)]

use super::cache_utils::clear_all_caches;
use super::load_plugin_hooks::load_plugin_hooks;
use super::lsp_plugin_integration::load_plugin_lsp_servers;
use super::mcp_plugin_integration::load_plugin_mcp_servers;
use super::orphaned_plugin_filter::clear_plugin_cache_exclusions;
use super::loader::load_all_plugins;

/// Result of refreshing active plugins.
pub struct RefreshActivePluginsResult {
    pub enabled_count: usize,
    pub disabled_count: usize,
    pub command_count: usize,
    pub agent_count: usize,
    pub hook_count: usize,
    pub mcp_count: usize,
    pub lsp_count: usize,
    pub error_count: usize,
}

/// Refresh all active plugin components.
pub async fn refresh_active_plugins() -> Result<RefreshActivePluginsResult, Box<dyn std::error::Error + Send + Sync>> {
    log::debug!("refresh_active_plugins: clearing all plugin caches");
    clear_all_caches();
    clear_plugin_cache_exclusions();

    // Load all plugins first
    let plugin_result = load_all_plugins().await?;

    let enabled = plugin_result.enabled;
    let disabled = plugin_result.disabled;
    let errors = plugin_result.errors;

    // Populate mcp_servers/lsp_servers on each enabled plugin
    let mut mcp_count = 0;
    let mut lsp_count = 0;

    for plugin in &enabled {
        if plugin.mcp_servers.is_some() {
            mcp_count += plugin.mcp_servers.as_ref().unwrap().len();
        } else if let Ok(Some(servers)) = load_plugin_mcp_servers(plugin, &mut Vec::new()).await {
            mcp_count += servers.len();
        }

        if plugin.lsp_servers.is_some() {
            lsp_count += plugin.lsp_servers.as_ref().unwrap().len();
        } else if let Ok(Some(servers)) = load_plugin_lsp_servers(plugin, &mut Vec::new()).await {
            lsp_count += servers.len();
        }
    }

    // Load plugin hooks
    let mut hook_load_failed = false;
    if let Err(e) = load_plugin_hooks().await {
        hook_load_failed = true;
        log::error!("refresh_active_plugins: load_plugin_hooks failed: {}", e);
    }

    // Count hooks from enabled plugins
    let hook_count: usize = enabled.iter().map(|p| {
        p.hooks_config.as_ref().map_or(0, |config| {
            if let Some(obj) = config.as_object() {
                obj.values().map(|v| {
                    v.as_array().map_or(0, |arr| {
                        arr.iter().map(|m| {
                            m.get("hooks").and_then(|h| h.as_array()).map_or(0, |hooks| hooks.len())
                        }).sum::<usize>()
                    })
                }).sum::<usize>()
            } else {
                0
            }
        })
    }).sum();

    log::debug!(
        "refresh_active_plugins: {} enabled, 0 commands, 0 agents, {} hooks, {} MCP, {} LSP",
        enabled.len(),
        hook_count,
        mcp_count,
        lsp_count
    );

    Ok(RefreshActivePluginsResult {
        enabled_count: enabled.len(),
        disabled_count: disabled.len(),
        command_count: 0,
        agent_count: 0,
        hook_count,
        mcp_count,
        lsp_count,
        error_count: errors.len() + if hook_load_failed { 1 } else { 0 },
    })
}

/// Merge fresh plugin-load errors with existing errors.
fn _merge_plugin_errors(
    existing: &[crate::plugin::types::PluginError],
    fresh: &[crate::plugin::types::PluginError],
) -> Vec<crate::plugin::types::PluginError> {
    let mut result = existing.to_vec();
    result.extend(fresh.iter().cloned());
    result
}
