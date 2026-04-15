// Source: /data/home/swei/claudecode/openclaudecode/src/services/autoDream/config.ts
//! Leaf config module for auto-dream feature.
//! Intentionally minimal imports so components can read the auto-dream
//! enabled state without dragging in the agent/task registry chain.

use crate::utils::settings::settings::get_initial_settings;

/// Whether background memory consolidation should run.
/// User setting (auto_dream_enabled in settings.json) overrides the
/// GrowthBook default when explicitly set; otherwise falls through to
/// tengu_onyx_plover.
pub fn is_auto_dream_enabled() -> bool {
    // Check user setting first
    if let Some(setting) = get_initial_settings().auto_dream_enabled {
        return setting;
    }

    // TODO: Integrate with GrowthBook
    // For now, return false as default
    // let gb = get_feature_value_cached_may_be_stale::<Value>("tengu_onyx_plover", None);
    // gb?.enabled == true
    false
}
