// Source: ~/claudecode/openclaudecode/src/utils/plugins/loadPluginHooks.ts
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use serde_json::Value;

use super::loader::load_all_plugins_cache_only;

static PLUGIN_HOOK_CACHE: Lazy<Mutex<Option<HashMap<String, Vec<PluginHookMatcher>>>>> =
    Lazy::new(|| Mutex::new(None));

/// Hook event types.
pub const HOOK_EVENTS: &[&str] = &[
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "PermissionDenied",
    "Notification",
    "UserPromptSubmit",
    "SessionStart",
    "SessionEnd",
    "Stop",
    "StopFailure",
    "SubagentStart",
    "SubagentStop",
    "PreCompact",
    "PostCompact",
    "PermissionRequest",
    "Setup",
    "TeammateIdle",
    "TaskCreated",
    "TaskCompleted",
    "Elicitation",
    "ElicitationResult",
    "ConfigChange",
    "WorktreeCreate",
    "WorktreeRemove",
    "InstructionsLoaded",
    "CwdChanged",
    "FileChanged",
];

/// Plugin hook matcher with plugin context.
#[derive(Clone, Debug)]
pub struct PluginHookMatcher {
    pub matcher: String,
    pub hooks: Vec<String>,
    pub plugin_root: String,
    pub plugin_name: String,
    pub plugin_id: String,
}

/// Convert plugin hooks configuration to native matchers with plugin context.
fn convert_plugin_hooks_to_matchers(
    plugin: &crate::plugin::types::LoadedPlugin,
) -> HashMap<String, Vec<PluginHookMatcher>> {
    let mut plugin_matchers: HashMap<String, Vec<PluginHookMatcher>> = HashMap::new();

    for &event in HOOK_EVENTS {
        plugin_matchers.insert(event.to_string(), Vec::new());
    }

    let hooks_config = match &plugin.hooks_config {
        Some(config) => config,
        None => return plugin_matchers,
    };

    if let Some(obj) = hooks_config.as_object() {
        for (event, matchers) in obj {
            if let Some(matchers_array) = matchers.as_array() {
                for matcher_entry in matchers_array {
                    if let Some(hooks) = matcher_entry.get("hooks").and_then(|h| h.as_array()) {
                        if !hooks.is_empty() {
                            let matcher = matcher_entry
                                .get("matcher")
                                .and_then(|m| m.as_str())
                                .unwrap_or("*")
                                .to_string();

                            let hook_list: Vec<String> = hooks
                                .iter()
                                .filter_map(|h| h.as_str().map(|s| s.to_string()))
                                .collect();

                            let entry = plugin_matchers.entry(event.clone()).or_default();
                            entry.push(PluginHookMatcher {
                                matcher,
                                hooks: hook_list,
                                plugin_root: plugin.path.clone(),
                                plugin_name: plugin.name.clone(),
                                plugin_id: plugin.source.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    plugin_matchers
}

/// Load and register hooks from all enabled plugins.
pub async fn load_plugin_hooks() -> Result<HashMap<String, Vec<PluginHookMatcher>>, Box<dyn std::error::Error + Send + Sync>> {
    let plugin_result = load_all_plugins_cache_only().await?;
    let mut all_plugin_hooks: HashMap<String, Vec<PluginHookMatcher>> = HashMap::new();

    for &event in HOOK_EVENTS {
        all_plugin_hooks.insert(event.to_string(), Vec::new());
    }

    for plugin in &plugin_result.enabled {
        let plugin_matchers = convert_plugin_hooks_to_matchers(plugin);

        for (event, matchers) in plugin_matchers {
            all_plugin_hooks
                .entry(event)
                .or_default()
                .extend(matchers);
        }
    }

    let total_hooks: usize = all_plugin_hooks
        .values()
        .map(|matchers| matchers.iter().map(|m| m.hooks.len()).sum::<usize>())
        .sum();

    log::debug!(
        "Registered {} hooks from {} plugins",
        total_hooks,
        plugin_result.enabled.len()
    );

    {
        let mut cache = PLUGIN_HOOK_CACHE.lock().unwrap();
        *cache = Some(all_plugin_hooks.clone());
    }

    Ok(all_plugin_hooks)
}

/// Clear the plugin hook cache.
pub fn clear_plugin_hook_cache() {
    let mut cache = PLUGIN_HOOK_CACHE.lock().unwrap();
    *cache = None;
}

/// Remove hooks from plugins no longer in the enabled set.
pub async fn prune_removed_plugin_hooks() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let plugin_result = load_all_plugins_cache_only().await?;
    let enabled_roots: std::collections::HashSet<_> =
        plugin_result.enabled.iter().map(|p| p.path.clone()).collect();

    let cache = PLUGIN_HOOK_CACHE.lock().unwrap();
    let current = match cache.as_ref() {
        Some(c) => c.clone(),
        None => return Ok(()),
    };

    let mut survivors: HashMap<String, Vec<PluginHookMatcher>> = HashMap::new();
    for (event, matchers) in current {
        let kept: Vec<PluginHookMatcher> = matchers
            .into_iter()
            .filter(|m| enabled_roots.contains(&m.plugin_root))
            .collect();
        if !kept.is_empty() {
            survivors.insert(event, kept);
        }
    }

    drop(cache);
    {
        let mut cache = PLUGIN_HOOK_CACHE.lock().unwrap();
        *cache = Some(survivors);
    }

    Ok(())
}

/// Reset hot reload subscription state.
pub fn reset_hot_reload_state() {
    // Test-only function
}
