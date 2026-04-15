// Source: ~/claudecode/openclaudecode/src/utils/permissions/PermissionUpdate.ts
#![allow(dead_code)]

//! Permission update application and persistence.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::types::permissions::{
    AdditionalWorkingDirectory, PermissionBehavior, PermissionRuleValue,
    ToolPermissionContext, ToolPermissionRulesBySource,
    PermissionUpdate as PermissionUpdateType,
    PermissionUpdateDestination,
};
use super::permission_rule_parser::{
    permission_rule_value_from_string, permission_rule_value_to_string,
};
use super::filesystem::to_posix_path;

/// Re-export the update type.
pub use crate::types::permissions::PermissionUpdate;

/// Extracts rules from permission updates.
pub fn extract_rules(updates: &[PermissionUpdateType]) -> Vec<PermissionRuleValue> {
    updates
        .iter()
        .flat_map(|update| {
            if let PermissionUpdateType::AddRules { rules, .. } = update {
                rules.clone()
            } else {
                vec![]
            }
        })
        .collect()
}

/// Checks if there are any rules in the updates.
pub fn has_rules(updates: &[PermissionUpdateType]) -> bool {
    !extract_rules(updates).is_empty()
}

/// Applies a single permission update to the context.
pub fn apply_permission_update(
    context: ToolPermissionContext,
    update: &PermissionUpdateType,
) -> ToolPermissionContext {
    match update {
        PermissionUpdateType::SetMode { mode, .. } => {
            log::debug!("Applying permission update: Setting mode to '{}'", mode);
            ToolPermissionContext {
                mode: mode.clone(),
                ..context
            }
        }
        PermissionUpdateType::AddRules {
            rules,
            behavior,
            destination,
        } => {
            let rule_strings: Vec<String> =
                rules.iter().map(permission_rule_value_to_string).collect();
            log::debug!(
                "Applying permission update: Adding {} {:?} rule(s) to destination '{:?}': {:?}",
                rules.len(),
                behavior,
                destination,
                rule_strings,
            );

            let mut ctx = context.clone();
            add_rules_to_context(&mut ctx, behavior, destination, &rule_strings);
            ctx
        }
        PermissionUpdateType::ReplaceRules {
            rules,
            behavior,
            destination,
        } => {
            let rule_strings: Vec<String> =
                rules.iter().map(permission_rule_value_to_string).collect();
            log::debug!(
                "Replacing all {:?} rules for destination '{:?}' with {} rule(s): {:?}",
                behavior,
                destination,
                rules.len(),
                rule_strings,
            );

            let mut ctx = context.clone();
            replace_rules_in_context(&mut ctx, behavior, destination, &rule_strings);
            ctx
        }
        PermissionUpdateType::AddDirectories {
            directories,
            destination,
        } => {
            log::debug!(
                "Applying permission update: Adding {} director{} with destination '{:?}'",
                directories.len(),
                if directories.len() == 1 { "y" } else { "ies" },
                destination,
            );
            let mut new_dirs = context.additional_working_directories.clone();
            for dir in directories {
                new_dirs.insert(
                    dir.clone(),
                    AdditionalWorkingDirectory {
                        path: dir.clone(),
                        source: crate::types::permissions::PermissionRuleSource::Session,
                    },
                );
            }
            ToolPermissionContext {
                additional_working_directories: new_dirs,
                ..context
            }
        }
        PermissionUpdateType::RemoveRules {
            rules,
            behavior,
            destination,
        } => {
            let rule_strings: Vec<String> =
                rules.iter().map(permission_rule_value_to_string).collect();
            log::debug!(
                "Applying permission update: Removing {} {:?} rule(s) from source '{:?}': {:?}",
                rules.len(),
                behavior,
                destination,
                rule_strings,
            );

            let mut ctx = context.clone();
            remove_rules_from_context(&mut ctx, behavior, destination, &rule_strings);
            ctx
        }
        PermissionUpdateType::RemoveDirectories { directories, .. } => {
            log::debug!(
                "Applying permission update: Removing {} director{}",
                directories.len(),
                if directories.len() == 1 { "y" } else { "ies" },
            );
            let mut new_dirs = context.additional_working_directories.clone();
            for dir in directories {
                new_dirs.remove(dir);
            }
            ToolPermissionContext {
                additional_working_directories: new_dirs,
                ..context
            }
        }
    }
}

fn add_rules_to_context(
    ctx: &mut ToolPermissionContext,
    behavior: &PermissionBehavior,
    destination: &PermissionUpdateDestination,
    rule_strings: &[String],
) {
    let rules = get_rules_mut(ctx, behavior, destination);
    rules.extend(rule_strings.iter().cloned());
}

fn replace_rules_in_context(
    ctx: &mut ToolPermissionContext,
    behavior: &PermissionBehavior,
    destination: &PermissionUpdateDestination,
    rule_strings: &[String],
) {
    let rules = get_rules_mut(ctx, behavior, destination);
    rules.clear();
    rules.extend(rule_strings.iter().cloned());
}

fn remove_rules_from_context(
    ctx: &mut ToolPermissionContext,
    behavior: &PermissionBehavior,
    destination: &PermissionUpdateDestination,
    rule_strings: &[String],
) {
    let rules = get_rules_mut(ctx, behavior, destination);
    let to_remove: std::collections::HashSet<String> =
        rule_strings.iter().cloned().collect();
    rules.retain(|r| !to_remove.contains(r));
}

fn get_rules_mut<'a>(
    ctx: &'a mut ToolPermissionContext,
    behavior: &PermissionBehavior,
    destination: &PermissionUpdateDestination,
) -> &'a mut Vec<String> {
    let rules_by_source = match behavior {
        PermissionBehavior::Allow => &mut ctx.always_allow_rules,
        PermissionBehavior::Deny => &mut ctx.always_deny_rules,
        PermissionBehavior::Ask => &mut ctx.always_ask_rules,
    };

    match destination {
        PermissionUpdateDestination::UserSettings => &mut rules_by_source.user_settings,
        PermissionUpdateDestination::ProjectSettings => &mut rules_by_source.project_settings,
        PermissionUpdateDestination::LocalSettings => &mut rules_by_source.local_settings,
        PermissionUpdateDestination::Session => &mut rules_by_source.session,
        PermissionUpdateDestination::CliArg => &mut rules_by_source.cli_arg,
    }
    .get_or_insert_with(Vec::new)
}

/// Applies multiple permission updates to the context.
pub fn apply_permission_updates(
    context: ToolPermissionContext,
    updates: &[PermissionUpdateType],
) -> ToolPermissionContext {
    let mut updated_context = context;
    for update in updates {
        updated_context = apply_permission_update(updated_context, update);
    }
    updated_context
}

/// Checks if a destination supports persistence.
pub fn supports_persistence(destination: &PermissionUpdateDestination) -> bool {
    matches!(
        destination,
        PermissionUpdateDestination::LocalSettings
            | PermissionUpdateDestination::UserSettings
            | PermissionUpdateDestination::ProjectSettings
    )
}

/// Persists a single permission update.
pub fn persist_permission_update(_update: &PermissionUpdateType) {
    if !supports_persistence(&match_destination(_update)) {
        return;
    }
    // In a full implementation, this would write to the settings file
}

fn match_destination(update: &PermissionUpdateType) -> PermissionUpdateDestination {
    match update {
        PermissionUpdateType::SetMode { destination, .. }
        | PermissionUpdateType::AddRules { destination, .. }
        | PermissionUpdateType::ReplaceRules { destination, .. }
        | PermissionUpdateType::RemoveRules { destination, .. }
        | PermissionUpdateType::AddDirectories { destination, .. }
        | PermissionUpdateType::RemoveDirectories { destination, .. } => destination.clone(),
    }
}

/// Persists multiple permission updates.
pub fn persist_permission_updates(updates: &[PermissionUpdateType]) {
    for update in updates {
        persist_permission_update(update);
    }
}

/// Converts rules to updates.
pub fn convert_rules_to_updates(
    rules: &[crate::types::permissions::PermissionRule],
    update_type: &str,
) -> Vec<PermissionUpdateType> {
    let mut grouped: HashMap<String, Vec<PermissionRuleValue>> = HashMap::new();

    for rule in rules {
        let key = format!("{}:{:?}", rule.source.as_str(), rule.rule_behavior);
        grouped.entry(key).or_default().push(rule.rule_value.clone());
    }

    let mut updates = Vec::new();
    for (key, rule_values) in grouped {
        let parts: Vec<&str> = key.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue;
        }
        let behavior = match parts[1] {
            "Allow" => PermissionBehavior::Allow,
            "Deny" => PermissionBehavior::Deny,
            "Ask" => PermissionBehavior::Ask,
            _ => continue,
        };
        updates.push(match update_type {
            "addRules" => PermissionUpdateType::AddRules {
                rules: rule_values,
                behavior,
                destination: PermissionUpdateDestination::Session,
            },
            "replaceRules" => PermissionUpdateType::ReplaceRules {
                rules: rule_values,
                behavior,
                destination: PermissionUpdateDestination::Session,
            },
            _ => continue,
        });
    }

    updates
}

/// Creates a Read rule suggestion for a directory.
pub fn create_read_rule_suggestion(
    dir_path: &str,
    destination: Option<PermissionUpdateDestination>,
) -> Option<PermissionUpdateType> {
    let path_for_pattern = to_posix_path(dir_path);

    // Root directory is too broad
    if path_for_pattern == "/" {
        return None;
    }

    let rule_content = if path_for_pattern.starts_with('/') {
        format!("/{}/**", path_for_pattern)
    } else {
        format!("{}/**", path_for_pattern)
    };

    Some(PermissionUpdateType::AddRules {
        rules: vec![PermissionRuleValue {
            tool_name: "Read".to_string(),
            rule_content: Some(rule_content),
        }],
        behavior: PermissionBehavior::Allow,
        destination: destination.unwrap_or(PermissionUpdateDestination::Session),
    })
}
