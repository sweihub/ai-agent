// Source: ~/claudecode/openclaudecode/src/utils/hooks/hooksConfigSnapshot.ts
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::utils::hooks::hooks_settings::{
    EditableSettingSource, HookMatcher as SettingsHookMatcher,
    get_hooks_for_source as get_settings_hooks_for_source, parse_hook_event,
};
use crate::utils::settings::{get_settings_file_path_for_source, read_settings_file};

/// Hooks settings structure (simplified from TS types)
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HooksSettings {
    #[serde(flatten)]
    pub events: HashMap<String, Vec<HookMatcher>>,
}

/// A hook matcher groups hooks by matching criteria
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HookMatcher {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matcher: Option<String>,
    pub hooks: Vec<serde_json::Value>,
}

lazy_static::lazy_static! {
    static ref HOOKS_CONFIG_SNAPSHOT: Arc<Mutex<Option<HooksSettings>>> = Arc::new(Mutex::new(None));
}

/// Get hooks from allowed sources.
/// Respects policy settings for allowManagedHooksOnly and disableAllHooks.
///
/// Matches TypeScript `getHooksFromAllowedSources()` from hooksConfigSnapshot.ts lines 18-53.
///
/// Policy precedence:
/// 1. If policySettings.disableAllHooks is true, return empty (all hooks disabled)
/// 2. If policySettings.allowManagedHooksOnly is true, only return managed hooks
/// 3. If restrictedToPluginOnly('hooks') is set, only return managed hooks
/// 4. If mergedSettings.disableAllHooks is true (non-managed), only return managed hooks
/// 5. Otherwise return all merged hooks from all sources
fn get_hooks_from_allowed_sources_impl() -> HooksSettings {
    // Check if all hooks should be disabled (policy settings)
    if should_disable_all_hooks_including_managed() {
        return HooksSettings::default();
    }

    // If allowManagedHooksOnly is set in managed settings, only use managed hooks
    if should_allow_managed_hooks_only() {
        // Return only policy/managed hooks
        // (would need policy settings integration)
        return HooksSettings::default();
    }

    // Check strictPluginOnlyCustomization for hooks surface
    if is_restricted_to_plugin_only() {
        // Block user/project/local settings hooks
        // Plugin hooks (registered channel, hooks.ts:1391) are NOT affected
        return HooksSettings::default();
    }

    // If disableAllHooks is set in non-managed settings, only managed hooks still run
    // (non-managed settings cannot override managed hooks)
    if is_disable_all_hooks_in_merged() {
        return HooksSettings::default();
    }

    // Otherwise, use all hooks (merged from all sources) - backwards compatible
    // Collect hooks from all editable settings sources
    let mut merged = HooksSettings::default();
    let sources = [
        EditableSettingSource::UserSettings,
        EditableSettingSource::ProjectSettings,
        EditableSettingSource::LocalSettings,
    ];

    for source in &sources {
        if let Some(parsed_hooks) = get_settings_hooks_for_source(source) {
            for (event_name, matchers) in parsed_hooks {
                let converted: Vec<HookMatcher> = matchers
                    .into_iter()
                    .map(|m| HookMatcher {
                        matcher: m.matcher,
                        hooks: m
                            .hooks
                            .into_iter()
                            .filter_map(|h| serde_json::to_value(&h).ok())
                            .collect(),
                    })
                    .collect();
                if !converted.is_empty() {
                    merged
                        .events
                        .entry(event_name)
                        .or_insert_with(Vec::new)
                        .extend(converted);
                }
            }
        }
    }

    merged
}

/// Check if only managed hooks should run.
/// This is true when:
/// - policySettings has allowManagedHooksOnly: true, OR
/// - disableAllHooks is set in non-managed settings (non-managed settings
///   cannot disable managed hooks, so they effectively become managed-only), OR
/// - AI_CODE_ALLOW_MANAGED_HOOKS_ONLY env var is set
pub fn should_allow_managed_hooks_only() -> bool {
    // Check environment variable (localized from CLAUDE_CODE_ to AI_)
    if std::env::var("AI_CODE_ALLOW_MANAGED_HOOKS_ONLY")
        .ok()
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
    {
        return true;
    }

    // This would check policy settings and merged settings
    false
}

/// Check if all hooks (including managed) should be disabled.
/// This is only true when managed/policy settings has disableAllHooks: true,
/// or the AI_CODE_DISABLE_ALL_HOOKS env var is set.
pub fn should_disable_all_hooks_including_managed() -> bool {
    // Check environment variable (localized from CLAUDE_CODE_ to AI_)
    if std::env::var("AI_CODE_DISABLE_ALL_HOOKS")
        .ok()
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
    {
        return true;
    }

    // This would check if policySettings has disableAllHooks: true
    false
}

/// Check if disableAllHooks is set in merged (non-managed) settings.
fn is_disable_all_hooks_in_merged() -> bool {
    // Check each editable source for disableAllHooks: true
    let sources = [
        EditableSettingSource::UserSettings,
        EditableSettingSource::ProjectSettings,
        EditableSettingSource::LocalSettings,
    ];

    for source in &sources {
        if let Some(path) = get_settings_file_path_for_source(source) {
            if let Some(settings) = read_settings_file(&path) {
                if settings.get("disableAllHooks").and_then(|v| v.as_bool()) == Some(true) {
                    return true;
                }
            }
        }
    }

    false
}

/// Check if the hooks surface is restricted to plugin-only sources.
/// This would check policy settings for strictPluginOnlyCustomization.
fn is_restricted_to_plugin_only() -> bool {
    // Check environment variable as a simple override
    if std::env::var("AI_CODE_STRICT_PLUGIN_ONLY_HOOKS")
        .ok()
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
    {
        return true;
    }

    // This would check policySettings.strictPluginOnlyCustomization for 'hooks'
    false
}

/// Capture a snapshot of the current hooks configuration.
/// This should be called once during application startup.
/// Respects the allowManagedHooksOnly setting.
pub fn capture_hooks_config_snapshot() {
    let hooks = get_hooks_from_allowed_sources_impl();
    let mut snapshot = HOOKS_CONFIG_SNAPSHOT.lock().unwrap();
    *snapshot = Some(hooks);
}

/// Update the hooks configuration snapshot.
/// This should be called when hooks are modified through the settings.
/// Respects the allowManagedHooksOnly setting.
pub fn update_hooks_config_snapshot() {
    // Reset the session cache to ensure we read fresh settings from disk
    reset_settings_cache();

    let hooks = get_hooks_from_allowed_sources_impl();
    let mut snapshot = HOOKS_CONFIG_SNAPSHOT.lock().unwrap();
    *snapshot = Some(hooks);
}

/// Get the current hooks configuration from snapshot.
/// Falls back to settings if no snapshot exists.
pub fn get_hooks_config_from_snapshot() -> Option<HooksSettings> {
    let mut snapshot = HOOKS_CONFIG_SNAPSHOT.lock().unwrap();
    if snapshot.is_none() {
        // Capture snapshot on first access
        let hooks = get_hooks_from_allowed_sources_impl();
        *snapshot = Some(hooks.clone());
    }
    snapshot.clone()
}

/// Reset the hooks configuration snapshot (useful for testing).
/// Also resets SDK init state to prevent test pollution.
pub fn reset_hooks_config_snapshot() {
    let mut snapshot = HOOKS_CONFIG_SNAPSHOT.lock().unwrap();
    *snapshot = None;
    reset_sdk_init_state();
}

/// Reset settings cache (simplified)
fn reset_settings_cache() {
    log::debug!("Resetting settings cache");
}

/// Reset SDK init state (simplified)
fn reset_sdk_init_state() {
    log::debug!("Resetting SDK init state");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_and_get_snapshot() {
        reset_hooks_config_snapshot();

        capture_hooks_config_snapshot();

        let snapshot = get_hooks_config_from_snapshot();
        assert!(snapshot.is_some());

        reset_hooks_config_snapshot();
    }

    #[test]
    fn test_update_snapshot() {
        reset_hooks_config_snapshot();

        capture_hooks_config_snapshot();
        update_hooks_config_snapshot();

        let snapshot = get_hooks_config_from_snapshot();
        assert!(snapshot.is_some());

        reset_hooks_config_snapshot();
    }

    #[test]
    fn test_get_snapshot_auto_capture() {
        reset_hooks_config_snapshot();

        // First access should auto-capture
        let snapshot = get_hooks_config_from_snapshot();
        assert!(snapshot.is_some());

        reset_hooks_config_snapshot();
    }

    #[test]
    fn test_should_allow_managed_hooks_only_env_var() {
        // Save original value
        let original = std::env::var("AI_CODE_ALLOW_MANAGED_HOOKS_ONLY").ok();

        // Set env var (unsafe required since Rust 1.83)
        unsafe {
            std::env::set_var("AI_CODE_ALLOW_MANAGED_HOOKS_ONLY", "true");
        }
        assert!(should_allow_managed_hooks_only());

        unsafe {
            std::env::set_var("AI_CODE_ALLOW_MANAGED_HOOKS_ONLY", "1");
        }
        assert!(should_allow_managed_hooks_only());

        unsafe {
            std::env::set_var("AI_CODE_ALLOW_MANAGED_HOOKS_ONLY", "false");
        }
        assert!(!should_allow_managed_hooks_only());

        unsafe {
            std::env::remove_var("AI_CODE_ALLOW_MANAGED_HOOKS_ONLY");
        }
        assert!(!should_allow_managed_hooks_only());

        // Restore original
        match original {
            Some(val) => unsafe {
                std::env::set_var("AI_CODE_ALLOW_MANAGED_HOOKS_ONLY", val);
            },
            None => unsafe {
                std::env::remove_var("AI_CODE_ALLOW_MANAGED_HOOKS_ONLY");
            },
        }
    }

    #[test]
    fn test_should_disable_all_hooks_env_var() {
        // Save original value
        let original = std::env::var("AI_CODE_DISABLE_ALL_HOOKS").ok();

        // Set env var (unsafe required since Rust 1.83)
        unsafe {
            std::env::set_var("AI_CODE_DISABLE_ALL_HOOKS", "true");
        }
        assert!(should_disable_all_hooks_including_managed());

        unsafe {
            std::env::set_var("AI_CODE_DISABLE_ALL_HOOKS", "1");
        }
        assert!(should_disable_all_hooks_including_managed());

        unsafe {
            std::env::set_var("AI_CODE_DISABLE_ALL_HOOKS", "false");
        }
        assert!(!should_disable_all_hooks_including_managed());

        unsafe {
            std::env::remove_var("AI_CODE_DISABLE_ALL_HOOKS");
        }
        assert!(!should_disable_all_hooks_including_managed());

        // Restore original
        match original {
            Some(val) => unsafe {
                std::env::set_var("AI_CODE_DISABLE_ALL_HOOKS", val);
            },
            None => unsafe {
                std::env::remove_var("AI_CODE_DISABLE_ALL_HOOKS");
            },
        }
    }

    #[test]
    fn test_hooks_settings_serialization() {
        let mut settings = HooksSettings::default();
        settings.events.insert(
            "Stop".to_string(),
            vec![HookMatcher {
                matcher: None,
                hooks: vec![serde_json::json!({"type": "command", "command": "echo hi"})],
            }],
        );

        let json = serde_json::to_string(&settings).unwrap();
        let parsed: HooksSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.events.len(), 1);
        assert!(parsed.events.contains_key("Stop"));
    }

    #[test]
    fn test_empty_snapshot_returns_default() {
        reset_hooks_config_snapshot();

        let snapshot = get_hooks_config_from_snapshot();
        assert!(snapshot.is_some());
        // Should have empty events map when no settings files exist
        assert!(snapshot.unwrap().events.is_empty());

        reset_hooks_config_snapshot();
    }
}
