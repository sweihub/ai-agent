// Source: ~/claudecode/openclaudecode/src/utils/permissions/PermissionResult.ts
#![allow(dead_code)]

//! Permission result types and helpers.

use crate::types::permissions::PermissionBehavior;

// Re-exports for backwards compatibility
pub use crate::types::permissions::{
    PermissionAllowDecision, PermissionAskDecision, PermissionDecision, PermissionDecisionReason,
    PermissionDenyDecision, PermissionMetadata,
};

/// Helper function to get the appropriate prose description for rule behavior.
pub fn get_rule_behavior_description(permission_result: &str) -> &'static str {
    match permission_result {
        "allow" => "allowed",
        "deny" => "denied",
        _ => "asked for confirmation for",
    }
}

/// Converts a PermissionBehavior to its string representation.
pub fn behavior_to_str(behavior: PermissionBehavior) -> &'static str {
    match behavior {
        PermissionBehavior::Allow => "allow",
        PermissionBehavior::Deny => "deny",
        PermissionBehavior::Ask => "ask",
    }
}
