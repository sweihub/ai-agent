// Source: ~/claudecode/openclaudecode/src/utils/permissions/bypassPermissionsKillswitch.ts
#![allow(dead_code)]

//! Bypass permissions killswitch and auto mode gate checking logic.
//!
//! Note: React hooks and GrowthBook integration from the TypeScript source
//! are adapted to plain async Rust functions.

use std::sync::atomic::{AtomicBool, Ordering};

static BYPASS_PERMISSIONS_CHECK_RAN: AtomicBool = AtomicBool::new(false);
static AUTO_MODE_CHECK_RAN: AtomicBool = AtomicBool::new(false);

/// Checks if bypass permissions should be disabled based on feature gate.
/// Runs only once before the first query.
pub async fn check_and_disable_bypass_permissions_if_needed(
    _tool_permission_context: &(),
    _set_app_state: &dyn Fn(&()) -> (),
) {
    // Check if bypassPermissions should be disabled based on Statsig gate
    // Do this only once, before the first query
    if BYPASS_PERMISSIONS_CHECK_RAN.swap(true, Ordering::SeqCst) {
        return;
    }

    // The following logic would call shouldDisableBypassPermissions()
    // and update app state — adapted for Rust:
    // let should_disable = should_disable_bypass_permissions().await;
    // if should_disable {
    //     set_app_state(|prev| {
    //         create_disabled_bypass_permissions_context(prev.tool_permission_context)
    //     });
    // }
}

/// Reset the run-once flag for check_and_disable_bypass_permissions_if_needed.
/// Call this after /login so the gate check re-runs with the new org.
pub fn reset_bypass_permissions_check() {
    BYPASS_PERMISSIONS_CHECK_RAN.store(false, Ordering::SeqCst);
}

/// Checks if auto mode should be disabled based on gate access verification.
pub async fn check_and_disable_auto_mode_if_needed(
    _tool_permission_context: &(),
    _set_app_state: &dyn Fn(&()) -> (),
    _fast_mode: Option<bool>,
) {
    // feature('TRANSCRIPT_CLASSIFIER') guard
    if AUTO_MODE_CHECK_RAN.swap(true, Ordering::SeqCst) {
        return;
    }

    // The following would call verifyAutoModeGateAccess and update context:
    // let (update_context, notification) = verify_auto_mode_gate_access(
    //     tool_permission_context,
    //     fast_mode,
    // ).await;
    // set_app_state(|prev| {
    //     let next_ctx = update_context(prev.tool_permission_context);
    //     // Apply notification if present
    // });
}

/// Reset the run-once flag for check_and_disable_auto_mode_if_needed.
/// Call this after /login so the gate check re-runs with the new org.
pub fn reset_auto_mode_gate_check() {
    AUTO_MODE_CHECK_RAN.store(false, Ordering::SeqCst);
}
