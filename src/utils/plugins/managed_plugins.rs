// Source: ~/claudecode/openclaudecode/src/utils/plugins/managedPlugins.ts
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

/// Plugin names locked by org policy.
pub fn get_managed_plugin_names() -> Option<HashSet<String>> {
    let enabled_plugins = get_settings_for_source("policySettings")
        .and_then(|settings| settings.enabled_plugins)?;

    if enabled_plugins.is_empty() {
        return None;
    }

    let mut names = HashSet::new();
    for (plugin_id, value) in &enabled_plugins {
        if !plugin_id.contains('@') {
            continue;
        }

        if let Some(name) = plugin_id.split('@').next() {
            if !name.is_empty() {
                names.insert(name.to_string());
            }
        }
    }

    if names.is_empty() {
        None
    } else {
        Some(names)
    }
}

/// Settings from a specific source.
#[derive(serde::Deserialize)]
struct SettingsSource {
    enabled_plugins: Option<HashMap<String, serde_json::Value>>,
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
    fn test_no_policy_returns_none() {
        #[allow(unused_unsafe)]
        unsafe {
            std::env::remove_var("AI_CODE_PLUGIN_POLICY");
        }
        assert!(get_managed_plugin_names().is_none());
    }
}
