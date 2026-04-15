// Source: /data/home/swei/claudecode/openclaudecode/src/keybindings/validate.ts
//! Keybinding validation

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeybindingWarningType {
    ParseError,
    Duplicate,
    Reserved,
    InvalidContext,
    InvalidAction,
}

#[derive(Debug, Clone)]
pub struct KeybindingWarning {
    pub warning_type: KeybindingWarningType,
    pub severity: String,
    pub message: String,
    pub key: Option<String>,
    pub context: Option<String>,
    pub action: Option<String>,
    pub suggestion: Option<String>,
}

const VALID_CONTEXTS: &[&str] = &[
    "Global",
    "Chat",
    "Autocomplete",
    "Confirmation",
    "Help",
    "Transcript",
    "HistorySearch",
    "Task",
    "ThemePicker",
    "Settings",
    "Tabs",
    "Attachments",
    "Footer",
    "MessageSelector",
    "DiffDialog",
    "ModelPicker",
    "Select",
    "Plugin",
];

pub fn is_valid_context(value: &str) -> bool {
    VALID_CONTEXTS.contains(&value)
}

pub fn validate_user_config(user_blocks: &[serde_json::Value]) -> Vec<KeybindingWarning> {
    let mut warnings = Vec::new();

    for (i, block) in user_blocks.iter().enumerate() {
        if let Some(obj) = block.as_object() {
            if let Some(context) = obj.get("context").and_then(|v| v.as_str()) {
                if !is_valid_context(context) {
                    warnings.push(KeybindingWarning {
                        warning_type: KeybindingWarningType::InvalidContext,
                        severity: "error".to_string(),
                        message: format!("Unknown context \"{}\"", context),
                        context: Some(context.to_string()),
                        key: None,
                        action: None,
                        suggestion: Some(format!("Valid contexts: {}", VALID_CONTEXTS.join(", "))),
                    });
                }
            }
        } else {
            warnings.push(KeybindingWarning {
                warning_type: KeybindingWarningType::ParseError,
                severity: "error".to_string(),
                message: format!("Keybinding block {} is not an object", i + 1),
                key: None,
                context: None,
                action: None,
                suggestion: None,
            });
        }
    }

    warnings
}

pub fn format_warning(warning: &KeybindingWarning) -> String {
    let icon = if warning.severity == "error" {
        "✗"
    } else {
        "⚠"
    };
    let mut msg = format!(
        "{} Keybinding {}: {}",
        icon, warning.severity, warning.message
    );

    if let Some(suggestion) = &warning.suggestion {
        msg.push_str(&format!("\n  {}", suggestion));
    }

    msg
}
