// Source: ~/claudecode/openclaudecode/src/utils/permissions/getNextPermissionMode.ts
#![allow(dead_code)]

//! Determines the next permission mode when cycling through modes with Shift+Tab.

use crate::types::permissions::{PermissionMode, ToolPermissionContext};

/// Determines the next permission mode when cycling through modes.
pub fn get_next_permission_mode(
    tool_permission_context: &ToolPermissionContext,
    _team_context: Option<&str>,
) -> String {
    match tool_permission_context.mode.as_str() {
        "default" => {
            // Ants skip acceptEdits and plan — auto mode replaces them
            if std::env::var("USER_TYPE").as_deref() == Ok("ant") {
                if tool_permission_context.is_bypass_permissions_mode_available {
                    return "bypassPermissions".to_string();
                }
                if can_cycle_to_auto(tool_permission_context) {
                    return "auto".to_string();
                }
                return "default".to_string();
            }
            "acceptEdits".to_string()
        }
        "acceptEdits" => "plan".to_string(),
        "plan" => {
            if tool_permission_context.is_bypass_permissions_mode_available {
                return "bypassPermissions".to_string();
            }
            if can_cycle_to_auto(tool_permission_context) {
                return "auto".to_string();
            }
            "default".to_string()
        }
        "bypassPermissions" => {
            if can_cycle_to_auto(tool_permission_context) {
                return "auto".to_string();
            }
            "default".to_string()
        }
        "dontAsk" => "default".to_string(),
        // Covers auto and any future modes
        _ => "default".to_string(),
    }
}

/// Checks if auto mode can be cycled to.
fn can_cycle_to_auto(ctx: &ToolPermissionContext) -> bool {
    // feature('TRANSCRIPT_CLASSIFIER') guard
    let gate_enabled = is_auto_mode_gate_enabled();
    // is_auto_mode_available is not on ToolPermissionContext in the Rust types
    // Using a placeholder for now
    let can = gate_enabled;
    if !can {
        log::debug!(
            "[auto-mode] canCycleToAuto=false: isAutoModeGateEnabled={}",
            gate_enabled,
        );
    }
    can
}

/// Checks if auto mode gate is enabled (synchronous check).
fn is_auto_mode_gate_enabled() -> bool {
    // In a full implementation, this would check GrowthBook/settings
    // For now, return a placeholder
    true
}

/// Computes the next permission mode and prepares the context for it.
pub fn cycle_permission_mode(
    tool_permission_context: &ToolPermissionContext,
    team_context: Option<&str>,
) -> (String, ToolPermissionContext) {
    let next_mode = get_next_permission_mode(tool_permission_context, team_context);
    let context = transition_permission_mode(
        &tool_permission_context.mode,
        &next_mode,
        tool_permission_context,
    );
    (next_mode, context)
}

/// Handles state transitions when switching permission modes.
pub fn transition_permission_mode(
    from_mode: &str,
    to_mode: &str,
    context: &ToolPermissionContext,
) -> ToolPermissionContext {
    if from_mode == to_mode {
        return context.clone();
    }

    // In a full implementation, this would:
    // - Handle plan mode enter/exit attachments
    // - Handle auto mode activation (strip/restore dangerous permissions)
    // - Set autoModeActive state

    // Only spread if there's something to clear
    if from_mode == "plan" && to_mode != "plan" && context.pre_plan_mode.is_some() {
        let mut new_ctx = context.clone();
        new_ctx.pre_plan_mode = None;
        return new_ctx;
    }

    context.clone()
}
