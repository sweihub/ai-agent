// Source: ~/claudecode/openclaudecode/src/utils/plugins/pluginPolicy.ts
#![allow(dead_code)]

use std::collections::HashMap;

/// Check if a plugin is force-disabled by org policy.
pub fn is_plugin_blocked_by_policy(plugin_id: &str) -> bool {
    get_settings_for_source("policySettings")
        .and_then(|settings| settings.enabled_plugins)
        .and_then(|enabled| enabled.get(plugin_id).cloned())
        == Some(false)
}

/// Settings from a specific source.
#[derive(serde::Deserialize)]
struct SettingsSource {
    enabled_plugins: Option<HashMap<String, bool>>,
}

/// Get settings for a specific source.
fn get_settings_for_source(source: &str) -> Option<SettingsSource> {
    match source {
        "policySettings" => {
            if let Ok(policy_json) = std::env::var("AI_CODE_PLUGIN_POLICY") {
                if let Ok(settings) = serde_json::from_str(&policy_json) {
                    return Some(settings);
                }
            }
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_blocked_without_policy() {
        #[allow(unused_unsafe)]
        unsafe {
            std::env::remove_var("AI_CODE_PLUGIN_POLICY");
        }
        assert!(!is_plugin_blocked_by_policy("some-plugin"));
    }
}
