// Source: ~/claudecode/openclaudecode/src/utils/hooks/hooksConfigSnapshot.ts
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
fn get_hooks_from_allowed_sources() -> HooksSettings {
    // In a real implementation, this would read from:
    // 1. Policy settings (managed settings)
    // 2. Merged settings (user, project, local, managed)
    //
    // The logic follows:
    // - If policySettings.disableAllHooks is true, return empty
    // - If policySettings.allowManagedHooksOnly is true, only return managed hooks
    // - If restrictedToPluginOnly('hooks') is set, only return managed hooks
    // - If mergedSettings.disableAllHooks is true (non-managed), only return managed hooks
    // - Otherwise return all merged hooks

    // For now, return empty (would need settings module integration)
    HooksSettings::default()
}

/// Check if only managed hooks should run
pub fn should_allow_managed_hooks_only() -> bool {
    // This would check policy settings and merged settings
    false
}

/// Check if all hooks (including managed) should be disabled
pub fn should_disable_all_hooks_including_managed() -> bool {
    // This would check if policySettings has disableAllHooks: true
    false
}

/// Capture a snapshot of the current hooks configuration.
/// This should be called once during application startup.
/// Respects the allowManagedHooksOnly setting.
pub fn capture_hooks_config_snapshot() {
    let hooks = get_hooks_from_allowed_sources();
    let mut snapshot = HOOKS_CONFIG_SNAPSHOT.lock().unwrap();
    *snapshot = Some(hooks);
}

/// Update the hooks configuration snapshot.
/// This should be called when hooks are modified through the settings.
/// Respects the allowManagedHooksOnly setting.
pub fn update_hooks_config_snapshot() {
    // Reset the session cache to ensure we read fresh settings from disk
    reset_settings_cache();

    let hooks = get_hooks_from_allowed_sources();
    let mut snapshot = HOOKS_CONFIG_SNAPSHOT.lock().unwrap();
    *snapshot = Some(hooks);
}

/// Get the current hooks configuration from snapshot.
/// Falls back to settings if no snapshot exists.
pub fn get_hooks_config_from_snapshot() -> Option<HooksSettings> {
    let mut snapshot = HOOKS_CONFIG_SNAPSHOT.lock().unwrap();
    if snapshot.is_none() {
        // Capture snapshot on first access
        let hooks = get_hooks_from_allowed_sources();
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
}
