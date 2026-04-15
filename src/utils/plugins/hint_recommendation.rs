// Source: ~/claudecode/openclaudecode/src/utils/plugins/hintRecommendation.ts
#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::Mutex;

use once_cell::sync::Lazy;

static TRIED_THIS_SESSION: Lazy<Mutex<HashSet<String>>> =
    Lazy::new(|| Mutex::new(HashSet::new()));

/// Plugin-hint recommendation utilities.
pub struct PluginHintRecommendation {
    pub plugin_id: String,
    pub plugin_name: String,
    pub marketplace_name: String,
    pub plugin_description: Option<String>,
    pub source_command: String,
}

/// Claude code hint structure.
pub struct ClaudeCodeHint {
    pub value: String,
    pub source_command: String,
}

/// Pre-store gate called when a plugin hint is detected.
pub fn _maybe_record_plugin_hint(_hint: &ClaudeCodeHint) {
    // Stub
}

/// Evaluate whether to show a plugin hint based on plugin and hint state.
pub fn _should_show_hint(_plugin_id: &str) -> bool {
    false
}

/// Resolve the pending hint to a renderable recommendation.
pub async fn resolve_plugin_hint(
    hint: &ClaudeCodeHint,
) -> Result<Option<PluginHintRecommendation>, Box<dyn std::error::Error + Send + Sync>> {
    let plugin_id = &hint.value;
    let parsed = super::plugin_identifier::parse_plugin_identifier(plugin_id);

    let plugin_data = super::marketplace_manager::get_plugin_by_id(plugin_id).await;

    let marketplace_name = parsed.marketplace.clone().unwrap_or_default();

    match plugin_data {
        Some(data) => Ok(Some(PluginHintRecommendation {
            plugin_id: plugin_id.clone(),
            plugin_name: data.entry.name.clone(),
            marketplace_name,
            plugin_description: data.entry.description.clone(),
            source_command: hint.source_command.clone(),
        })),
        None => {
            log::debug!("[hint_recommendation] {} not found in marketplace cache", plugin_id);
            Ok(None)
        }
    }
}

/// Record that a prompt for this plugin was surfaced.
pub fn _mark_hint_plugin_shown(_plugin_id: &str) {
    // Stub
}

/// Called when the user picks "don't show plugin installation hints again".
pub fn _disable_hint_recommendations() {
    // Stub
}

/// Test-only reset.
pub fn _reset_hint_recommendation_for_testing() {
    TRIED_THIS_SESSION.lock().unwrap().clear();
}
