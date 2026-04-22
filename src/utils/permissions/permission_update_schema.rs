// Source: ~/claudecode/openclaudecode/src/utils/permissions/PermissionUpdateSchema.ts
#![allow(dead_code)]

//! Zod schemas for permission updates, ported to Rust validation types.

use crate::types::permissions::{
    ExternalPermissionMode, PermissionBehavior, PermissionRuleValue, PermissionUpdateDestination,
};
use serde::{Deserialize, Serialize};

/// Validates a permission update destination string.
pub fn validate_permission_update_destination(s: &str) -> bool {
    matches!(
        s,
        "userSettings" | "projectSettings" | "localSettings" | "session" | "cliArg"
    )
}

/// Parses a permission update destination from string.
pub fn permission_update_destination_from_string(s: &str) -> Option<PermissionUpdateDestination> {
    match s {
        "userSettings" => Some(PermissionUpdateDestination::UserSettings),
        "projectSettings" => Some(PermissionUpdateDestination::ProjectSettings),
        "localSettings" => Some(PermissionUpdateDestination::LocalSettings),
        "session" => Some(PermissionUpdateDestination::Session),
        "cliArg" => Some(PermissionUpdateDestination::CliArg),
        _ => None,
    }
}

/// Validates a permission behavior string.
pub fn validate_permission_behavior(s: &str) -> bool {
    matches!(s, "allow" | "deny" | "ask")
}

/// Parses a permission behavior from string.
pub fn permission_behavior_from_string(s: &str) -> Option<PermissionBehavior> {
    match s {
        "allow" => Some(PermissionBehavior::Allow),
        "deny" => Some(PermissionBehavior::Deny),
        "ask" => Some(PermissionBehavior::Ask),
        _ => None,
    }
}

/// Validates an external permission mode string.
pub fn validate_external_permission_mode(s: &str) -> bool {
    matches!(
        s,
        "acceptEdits" | "bypassPermissions" | "default" | "dontAsk" | "plan"
    )
}
