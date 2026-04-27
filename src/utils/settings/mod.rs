// Source: ~/claudecode/openclaudecode/src/utils/settings/settings.ts
//! Settings file I/O for reading/writing Claude Code configuration.
//!
//! Translated from TypeScript settings.ts
//! Provides get_settings_for_source() and update_settings_for_source()
//! for reading and persisting settings to JSON files.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

pub mod permission_validation;
pub mod settings_cache;
pub mod tool_validation_config;
pub mod validation;

// Re-export from MCP types for use in settings validation
pub use crate::services::mcp::ConfigScope;

#[cfg(test)]
#[path = "tests/settings_tests.rs"]
mod settings_tests;

/// Editable setting sources that can be read from and written to disk.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EditableSettingSource {
    /// ~/.ai/settings.json
    UserSettings,
    /// <cwd>/.ai/settings.json
    ProjectSettings,
    /// <cwd>/.ai/settings.local.json
    LocalSettings,
}

/// All setting sources including non-editable ones
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SettingSource {
    UserSettings,
    ProjectSettings,
    LocalSettings,
    PolicySettings,
    FlagSettings,
}

/// Get the file path for a given settings source.
/// Returns None for sources without a file path.
pub fn get_settings_file_path_for_source(source: &EditableSettingSource) -> Option<PathBuf> {
    match source {
        EditableSettingSource::UserSettings => {
            dirs::home_dir().map(|home| home.join(".ai").join("settings.json"))
        }
        EditableSettingSource::ProjectSettings => {
            std::env::current_dir().ok().map(|cwd| cwd.join(".ai").join("settings.json"))
        }
        EditableSettingSource::LocalSettings => {
            std::env::current_dir().ok().map(|cwd| cwd.join(".ai").join("settings.local.json"))
        }
    }
}

/// Read settings JSON from a file. Returns None if file doesn't exist or can't be read.
pub fn read_settings_file(path: &Path) -> Option<Value> {
    let content = std::fs::read_to_string(path).ok()?;
    if content.trim().is_empty() {
        return Some(Value::Object(serde_json::Map::new()));
    }
    serde_json::from_str(&content).ok()
}

/// Deep merge two JSON values. `overlay` is merged into `base`.
/// Arrays in `overlay` replace arrays in `base` entirely.
/// `null` values in `overlay` delete keys from `base`.
fn deep_merge(base: &Value, overlay: &Value) -> Value {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            let mut result = base_map.clone();
            for (key, overlay_val) in overlay_map {
                if overlay_val.is_null() {
                    result.remove(key);
                } else {
                    let base_val = result.get(key);
                    result.insert(
                        key.clone(),
                        match base_val {
                            Some(b) => deep_merge(b, overlay_val),
                            None => overlay_val.clone(),
                        },
                    );
                }
            }
            Value::Object(result)
        }
        (_, Value::Array(overlay_arr)) => overlay.clone(),
        (_, other) => other.clone(),
    }
}

/// Get settings for a source by reading from disk.
/// Returns None if the settings file doesn't exist or is invalid.
pub fn get_settings_for_source(source: &EditableSettingSource) -> Option<Value> {
    let path = get_settings_file_path_for_source(source)?;
    read_settings_file(&path)
}

/// Merge `settings` into the existing settings file for `source`.
/// Creates the directory and file if they don't exist.
/// Returns Err on I/O or JSON parse errors.
pub fn update_settings_for_source(
    source: &EditableSettingSource,
    settings: &Value,
) -> Result<(), String> {
    let file_path =
        get_settings_file_path_for_source(source).ok_or("Cannot determine settings path")?;

    // Create directory if needed
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create settings directory: {}", e))?;
    }

    // Read existing settings
    let existing = read_settings_file(&file_path).unwrap_or(Value::Object(serde_json::Map::new()));

    // Deep merge new settings into existing
    let merged = deep_merge(&existing, settings);

    // Write back
    let json_str = serde_json::to_string_pretty(&merged)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&file_path, json_str + "\n")
        .map_err(|e| format!("Failed to write settings file: {}", e))?;

    Ok(())
}

/// Add permission rules to settings file.
/// This is the core persistence function called by persist_permission_update.
pub fn add_permission_rules_to_settings(
    rules: &[String],
    behavior: &str, // "allow", "deny", or "ask"
    source: &EditableSettingSource,
) -> Result<(), String> {
    let existing = get_settings_for_source(source).unwrap_or(Value::Object(serde_json::Map::new()));

    // Get current permission rules for this behavior
    let current_rules: Vec<String> = existing
        .get("permissions")
        .and_then(|p| p.get(behavior))
        .and_then(|r| r.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    // Add new rules, avoiding duplicates
    let mut all_rules = current_rules;
    for rule in rules {
        if !all_rules.contains(rule) {
            all_rules.push(rule.clone());
        }
    }

    // Build settings to merge
    let mut perms = serde_json::Map::new();
    perms.insert(
        behavior.to_string(),
        Value::Array(all_rules.into_iter().map(Value::String).collect()),
    );
    let settings = Value::Object(
        [("permissions".to_string(), Value::Object(perms))]
            .into_iter()
            .collect::<Map<_, _>>(),
    );

    update_settings_for_source(source, &settings)
}

/// Remove permission rules from settings file.
pub fn remove_permission_rules_from_settings(
    rules: &[String],
    behavior: &str,
    source: &EditableSettingSource,
) -> Result<(), String> {
    let current_rules: Vec<String> = match get_settings_for_source(source) {
        Some(s) => s.get("permissions")
            .and_then(|p| p.get(behavior))
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default(),
        None => Vec::new(),
    };

    let rules_to_remove: std::collections::HashSet<&str> = rules.iter().map(|s| s.as_str()).collect();
    let filtered: Vec<String> =
        current_rules.into_iter().filter(|r| !rules_to_remove.contains(r.as_str())).collect();

    let mut perms = serde_json::Map::new();
    perms.insert(
        behavior.to_string(),
        Value::Array(filtered.into_iter().map(Value::String).collect()),
    );
    let settings = Value::Object(
        [("permissions".to_string(), Value::Object(perms))]
            .into_iter()
            .collect::<Map<_, _>>(),
    );

    update_settings_for_source(source, &settings)
}

/// Replace all permission rules for a behavior in settings file.
pub fn replace_permission_rules_in_settings(
    rules: &[String],
    behavior: &str,
    source: &EditableSettingSource,
) -> Result<(), String> {
    let mut perms = serde_json::Map::new();
    perms.insert(
        behavior.to_string(),
        Value::Array(rules.iter().map(|r| Value::String(r.clone())).collect()),
    );
    let settings = Value::Object(
        [("permissions".to_string(), Value::Object(perms))]
            .into_iter()
            .collect::<Map<_, _>>(),
    );

    update_settings_for_source(source, &settings)
}

/// Manage additional directories in settings
pub fn add_directories_to_settings(
    directories: &[String],
    source: &EditableSettingSource,
) -> Result<(), String> {
    let current_dirs: Vec<String> = match get_settings_for_source(source) {
        Some(s) => s.get("permissions")
            .and_then(|p| p.get("additionalDirectories"))
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default(),
        None => Vec::new(),
    };

    let existing: std::collections::HashSet<String> = current_dirs.iter().cloned().collect();
    let mut all_dirs = current_dirs;
    for dir in directories {
        if !existing.contains(dir) {
            all_dirs.push(dir.clone());
        }
    }

    let mut perms = serde_json::Map::new();
    perms.insert(
        "additionalDirectories".to_string(),
        Value::Array(all_dirs.into_iter().map(Value::String).collect()),
    );
    let settings = Value::Object(
        [("permissions".to_string(), Value::Object(perms))]
            .into_iter()
            .collect::<Map<_, _>>(),
    );

    update_settings_for_source(source, &settings)
}

/// Remove directories from settings
pub fn remove_directories_from_settings(
    directories: &[String],
    source: &EditableSettingSource,
) -> Result<(), String> {
    let current_dirs: Vec<String> = match get_settings_for_source(source) {
        Some(s) => s.get("permissions")
            .and_then(|p| p.get("additionalDirectories"))
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default(),
        None => Vec::new(),
    };

    let dirs_to_remove: std::collections::HashSet<&str> =
        directories.iter().map(|s| s.as_str()).collect();
    let filtered: Vec<String> =
        current_dirs.into_iter().filter(|d| !dirs_to_remove.contains(d.as_str())).collect();

    let mut perms = serde_json::Map::new();
    perms.insert(
        "additionalDirectories".to_string(),
        Value::Array(filtered.into_iter().map(Value::String).collect()),
    );
    let settings = Value::Object(
        [("permissions".to_string(), Value::Object(perms))]
            .into_iter()
            .collect::<Map<_, _>>(),
    );

    update_settings_for_source(source, &settings)
}

/// Set permission mode in settings
pub fn set_permission_mode_in_settings(
    mode: &str,
    source: &EditableSettingSource,
) -> Result<(), String> {
    let mut perms = serde_json::Map::new();
    perms.insert("defaultMode".to_string(), Value::String(mode.to_string()));
    let settings = Value::Object(
        [("permissions".to_string(), Value::Object(perms))]
            .into_iter()
            .collect::<Map<_, _>>(),
    );

    update_settings_for_source(source, &settings)
}
