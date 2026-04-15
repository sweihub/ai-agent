// Source: ~/claudecode/openclaudecode/src/utils/permissions/permissionsLoader.ts
#![allow(dead_code)]

//! Permissions loader — loads and persists permission rules from settings files.

use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::types::permissions::{
    PermissionBehavior, PermissionRule, PermissionRuleSource,
    PermissionRuleValue,
};
use super::permission_rule_parser::{
    permission_rule_value_from_string, permission_rule_value_to_string,
};

/// Settings JSON structure for permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsPermissions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "additionalDirectories")]
    pub additional_directories: Option<Vec<String>>,
}

/// Settings JSON structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<SettingsPermissions>,
}

/// Checks if managed permission rules only mode is enabled.
pub fn should_allow_managed_permission_rules_only() -> bool {
    // In a full implementation, this would read from policy settings
    false
}

/// Checks if "always allow" options should be shown in permission prompts.
pub fn should_show_always_allow_options() -> bool {
    !should_allow_managed_permission_rules_only()
}

/// Supported rule behaviors.
const SUPPORTED_RULE_BEHAVIORS: &[&str] = &["allow", "deny", "ask"];

/// Converts settings JSON to permission rules.
fn settings_json_to_rules(
    data: Option<&SettingsJson>,
    source: PermissionRuleSource,
) -> Vec<PermissionRule> {
    let Some(settings) = data else {
        return vec![];
    };
    let Some(permissions) = &settings.permissions else {
        return vec![];
    };

    let mut rules = Vec::new();

    let behavior_map = [
        ("allow", &permissions.allow),
        ("deny", &permissions.deny),
        ("ask", &permissions.ask),
    ];

    for (behavior_str, rules_opt) in &behavior_map {
        if let Some(rule_strings) = rules_opt {
            let behavior = match *behavior_str {
                "allow" => PermissionBehavior::Allow,
                "deny" => PermissionBehavior::Deny,
                "ask" => PermissionBehavior::Ask,
                _ => continue,
            };
            for rule_string in rule_strings {
                rules.push(PermissionRule {
                    source: source.clone(),
                    rule_behavior: behavior.clone(),
                    rule_value: permission_rule_value_from_string(rule_string),
                });
            }
        }
    }

    rules
}

/// Loads all permission rules from disk.
pub fn load_all_permission_rules_from_disk() -> Vec<PermissionRule> {
    if should_allow_managed_permission_rules_only() {
        return get_permission_rules_for_source(&PermissionRuleSource::PolicySettings);
    }

    let mut rules = Vec::new();
    let sources = [
        PermissionRuleSource::UserSettings,
        PermissionRuleSource::ProjectSettings,
        PermissionRuleSource::LocalSettings,
    ];
    for source in sources {
        rules.extend(get_permission_rules_for_source(&source));
    }
    rules
}

/// Gets permission rules for a specific source.
pub fn get_permission_rules_for_source(source: &PermissionRuleSource) -> Vec<PermissionRule> {
    let settings = get_settings_for_source(source);
    settings_json_to_rules(settings.as_ref(), source.clone())
}

/// Gets settings for a source (stub — full implementation needs file I/O).
fn get_settings_for_source(_source: &PermissionRuleSource) -> Option<SettingsJson> {
    // In a full implementation, this would read from the appropriate settings file
    None
}

/// Deletes a permission rule from settings.
pub fn delete_permission_rule_from_settings(
    rule: &PermissionRule,
) -> bool {
    // Only editable sources
    if !matches!(
        rule.source,
        PermissionRuleSource::UserSettings
            | PermissionRuleSource::ProjectSettings
            | PermissionRuleSource::LocalSettings
    ) {
        return false;
    }

    let rule_string = permission_rule_value_to_string(&rule.rule_value);
    let settings = match get_settings_for_source(&rule.source) {
        Some(s) => s,
        None => return false,
    };

    let permissions = match &settings.permissions {
        Some(p) => p,
        None => return false,
    };

    let existing_rules = match rule.rule_behavior {
        PermissionBehavior::Allow => &permissions.allow,
        PermissionBehavior::Deny => &permissions.deny,
        PermissionBehavior::Ask => &permissions.ask,
    };

    let Some(existing) = existing_rules else {
        return false;
    };

    let normalize_entry = |raw: &str| -> String {
        permission_rule_value_to_string(&permission_rule_value_from_string(raw))
    };

    if !existing.iter().any(|raw| normalize_entry(raw) == rule_string) {
        return false;
    }

    // In a full implementation, this would write to the settings file
    log::debug!("Deleted permission rule: {}", rule_string);
    true
}

/// Adds permission rules to settings.
pub fn add_permission_rules_to_settings(
    rule_values: &[PermissionRuleValue],
    rule_behavior: &PermissionBehavior,
    source: &PermissionRuleSource,
) -> bool {
    if should_allow_managed_permission_rules_only() {
        return false;
    }

    if rule_values.is_empty() {
        return true;
    }

    let rule_strings: Vec<String> = rule_values
        .iter()
        .map(permission_rule_value_to_string)
        .collect();

    let settings = get_settings_for_source(source).unwrap_or(SettingsJson {
        permissions: None,
    });

    let mut permissions = settings.permissions.unwrap_or(SettingsPermissions {
        allow: None, deny: None, ask: None, additional_directories: None,
    });

    let existing_rules = match rule_behavior {
        PermissionBehavior::Allow => &mut permissions.allow,
        PermissionBehavior::Deny => &mut permissions.deny,
        PermissionBehavior::Ask => &mut permissions.ask,
    };

    let existing_set: HashSet<String> = existing_rules
        .as_ref()
        .map(|rules| {
            rules
                .iter()
                .map(|raw| permission_rule_value_to_string(&permission_rule_value_from_string(raw)))
                .collect()
        })
        .unwrap_or_default();

    let new_rules: Vec<String> = rule_strings
        .iter()
        .filter(|rule| !existing_set.contains(&permission_rule_value_to_string(
            &permission_rule_value_from_string(rule),
        )))
        .cloned()
        .collect();

    if new_rules.is_empty() {
        return true;
    }

    if let Some(rules) = existing_rules {
        rules.extend(new_rules);
    } else {
        *existing_rules = Some(new_rules);
    }

    // In a full implementation, this would write to the settings file
    log::debug!("Added permission rules to {:?}", source);
    true
}
