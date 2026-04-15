// Source: ~/claudecode/openclaudecode/src/utils/permissions/PermissionMode.ts
#![allow(dead_code)]

//! Permission mode utilities — title, symbol, color helpers.

use crate::types::permissions::{
    EXTERNAL_PERMISSION_MODES, INTERNAL_PERMISSION_MODES,
    ExternalPermissionMode, PermissionMode,
};

// Re-exports for backwards compatibility

/// Color key for permission mode display.
pub enum ModeColorKey {
    Text,
    PlanMode,
    Permission,
    AutoAccept,
    Error,
    Warning,
}

impl ModeColorKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModeColorKey::Text => "text",
            ModeColorKey::PlanMode => "planMode",
            ModeColorKey::Permission => "permission",
            ModeColorKey::AutoAccept => "autoAccept",
            ModeColorKey::Error => "error",
            ModeColorKey::Warning => "warning",
        }
    }
}

/// Configuration for a permission mode.
struct PermissionModeConfig {
    title: &'static str,
    short_title: &'static str,
    symbol: &'static str,
    color: ModeColorKey,
    external: &'static str,
}

const PAUSE_ICON: &str = "\u{23F8}";

const DEFAULT_CONFIG: PermissionModeConfig = PermissionModeConfig {
    title: "Default",
    short_title: "Default",
    symbol: "",
    color: ModeColorKey::Text,
    external: "default",
};

fn get_mode_config(mode: &str) -> PermissionModeConfig {
    match mode {
        "default" => DEFAULT_CONFIG,
        "plan" => PermissionModeConfig {
            title: "Plan Mode",
            short_title: "Plan",
            symbol: PAUSE_ICON,
            color: ModeColorKey::PlanMode,
            external: "plan",
        },
        "acceptEdits" => PermissionModeConfig {
            title: "Accept edits",
            short_title: "Accept",
            symbol: "\u{23F5}\u{23F5}",
            color: ModeColorKey::AutoAccept,
            external: "acceptEdits",
        },
        "bypassPermissions" => PermissionModeConfig {
            title: "Bypass Permissions",
            short_title: "Bypass",
            symbol: "\u{23F5}\u{23F5}",
            color: ModeColorKey::Error,
            external: "bypassPermissions",
        },
        "dontAsk" => PermissionModeConfig {
            title: "Don't Ask",
            short_title: "DontAsk",
            symbol: "\u{23F5}\u{23F5}",
            color: ModeColorKey::Error,
            external: "dontAsk",
        },
        "auto" => PermissionModeConfig {
            title: "Auto mode",
            short_title: "Auto",
            symbol: "\u{23F5}\u{23F5}",
            color: ModeColorKey::Warning,
            external: "default",
        },
        _ => DEFAULT_CONFIG,
    }
}

/// Type guard to check if a PermissionMode is an ExternalPermissionMode.
pub fn is_external_permission_mode(mode: &str) -> bool {
    // External users can't have auto, so always true for them
    if std::env::var("USER_TYPE").as_deref() != Ok("ant") {
        return true;
    }
    mode != "auto" && mode != "bubble"
}

/// Converts a PermissionMode to an ExternalPermissionMode.
pub fn to_external_permission_mode(mode: &str) -> String {
    get_mode_config(mode).external.to_string()
}

/// Parses a string into a PermissionMode.
pub fn permission_mode_from_string(s: &str) -> String {
    if INTERNAL_PERMISSION_MODES.contains(&s) {
        s.to_string()
    } else {
        "default".to_string()
    }
}

/// Gets the display title for a permission mode.
pub fn permission_mode_title(mode: &str) -> &'static str {
    get_mode_config(mode).title
}

/// Checks if a mode is the default mode.
pub fn is_default_mode(mode: Option<&str>) -> bool {
    mode.map_or(true, |m| m == "default")
}

/// Gets the short title for a permission mode.
pub fn permission_mode_short_title(mode: &str) -> &'static str {
    get_mode_config(mode).short_title
}

/// Gets the symbol for a permission mode.
pub fn permission_mode_symbol(mode: &str) -> &'static str {
    get_mode_config(mode).symbol
}

/// Gets the color key for a permission mode.
pub fn get_mode_color(mode: &str) -> ModeColorKey {
    get_mode_config(mode).color
}
