// Source: ~/claudecode/openclaudecode/src/utils/permissions/permissionSetup.ts
#![allow(dead_code)]

//! Permission setup — dangerous permission detection, auto-mode preparation,
//! and mode transition logic.

use std::path::{Path, MAIN_SEPARATOR};
use crate::types::permissions::{
    PermissionRule, PermissionRuleSource, PermissionRuleValue,
    ToolPermissionContext, ToolPermissionRulesBySource,
};
use super::dangerous_patterns::{CROSS_PLATFORM_CODE_EXEC, dangerous_bash_patterns};
use super::permission_rule_parser::{
    normalize_legacy_tool_name, permission_rule_value_from_string,
    permission_rule_value_to_string,
};

/// Checks if a Bash permission rule is dangerous for auto mode.
pub fn is_dangerous_bash_permission(
    tool_name: &str,
    rule_content: Option<&str>,
) -> bool {
    if tool_name != "Bash" {
        return false;
    }

    if rule_content.is_none() || rule_content == Some("") {
        return true;
    }

    let content = rule_content.unwrap().trim().to_lowercase();

    if content == "*" {
        return true;
    }

    for pattern in dangerous_bash_patterns() {
        let lower_pattern = pattern.to_lowercase();

        if content == lower_pattern {
            return true;
        }
        if content == format!("{}:*", lower_pattern) {
            return true;
        }
        if content == format!("{}*", lower_pattern) {
            return true;
        }
        if content == format!("{} *", lower_pattern) {
            return true;
        }
        if content.starts_with(&format!("{} -", lower_pattern)) && content.ends_with('*') {
            return true;
        }
    }

    false
}

/// Checks if a PowerShell permission rule is dangerous for auto mode.
pub fn is_dangerous_power_shell_permission(
    tool_name: &str,
    rule_content: Option<&str>,
) -> bool {
    if tool_name != "PowerShell" {
        return false;
    }

    if rule_content.is_none() || rule_content == Some("") {
        return true;
    }

    let content = rule_content.unwrap().trim().to_lowercase();

    if content == "*" {
        return true;
    }

    // PS-specific patterns
    let mut patterns: Vec<&str> = CROSS_PLATFORM_CODE_EXEC.to_vec();
    patterns.extend(&[
        "pwsh", "powershell", "cmd", "wsl",
        "iex", "invoke-expression", "icm", "invoke-command",
        "start-process", "saps", "start", "start-job", "sajb",
        "start-threadjob",
        "register-objectevent", "register-engineevent",
        "register-wmievent", "register-scheduledjob",
        "new-pssession", "nsn", "enter-pssession", "etsn",
        "add-type", "new-object",
    ]);

    for pattern in patterns {
        if content == pattern { return true; }
        if content == format!("{}:*", pattern) { return true; }
        if content == format!("{}*", pattern) { return true; }
        if content == format!("{} *", pattern) { return true; }
        if content.starts_with(&format!("{} -", pattern)) && content.ends_with('*') { return true; }

        // .exe variants
        let sp = pattern.find(' ');
        let exe = match sp {
            None => format!("{}.exe", pattern),
            Some(idx) => format!("{}.exe{}", &pattern[..idx], &pattern[idx..]),
        };
        if content == exe { return true; }
        if content == format!("{}:*", exe) { return true; }
        if content == format!("{}*", exe) { return true; }
        if content == format!("{} *", exe) { return true; }
        if content.starts_with(&format!("{} -", exe)) && content.ends_with('*') { return true; }
    }

    false
}

/// Checks if an Agent permission rule is dangerous for auto mode.
pub fn is_dangerous_task_permission(
    tool_name: &str,
    _rule_content: Option<&str>,
) -> bool {
    normalize_legacy_tool_name(tool_name) == "Agent"
}

/// Information about a dangerous permission.
pub struct DangerousPermissionInfo {
    pub rule_value: PermissionRuleValue,
    pub source: PermissionRuleSource,
    pub rule_display: String,
    pub source_display: String,
}

/// Checks if a permission rule is dangerous for auto mode.
fn is_dangerous_classifier_permission(
    tool_name: &str,
    rule_content: Option<&str>,
) -> bool {
    if std::env::var("USER_TYPE").as_deref() == Ok("ant") {
        if tool_name == "Tmux" {
            return true;
        }
    }
    is_dangerous_bash_permission(tool_name, rule_content)
        || is_dangerous_power_shell_permission(tool_name, rule_content)
        || is_dangerous_task_permission(tool_name, rule_content)
}

/// Finds all dangerous permissions from rules.
pub fn find_dangerous_classifier_permissions(
    rules: &[PermissionRule],
    cli_allowed_tools: &[String],
) -> Vec<DangerousPermissionInfo> {
    let mut dangerous = Vec::new();

    for rule in rules {
        if matches!(rule.rule_behavior, crate::types::permissions::PermissionBehavior::Allow)
            && is_dangerous_classifier_permission(
                &rule.rule_value.tool_name,
                rule.rule_value.rule_content.as_deref(),
            )
        {
            let rule_string = rule.rule_value.rule_content.as_ref().map_or_else(
                || format!("{}(*)", rule.rule_value.tool_name),
                |c| format!("{}({})", rule.rule_value.tool_name, c),
            );
            dangerous.push(DangerousPermissionInfo {
                rule_value: rule.rule_value.clone(),
                source: rule.source.clone(),
                rule_display: rule_string,
                source_display: format_permission_source(&rule.source),
            });
        }
    }

    for tool_spec in cli_allowed_tools {
        let parsed = permission_rule_value_from_string(tool_spec);
        if is_dangerous_classifier_permission(
            &parsed.tool_name,
            parsed.rule_content.as_deref(),
        ) {
            dangerous.push(DangerousPermissionInfo {
                rule_display: parsed.rule_content.as_ref().map_or_else(
                    || format!("{}(*)", parsed.tool_name),
                    |c| format!("{}({})", parsed.tool_name, c),
                ),
                source: PermissionRuleSource::CliArg,
                source_display: "--allowed-tools".to_string(),
                rule_value: parsed,
            });
        }
    }

    dangerous
}

/// Checks if a Bash allow rule is overly broad.
pub fn is_overly_broad_bash_allow_rule(rule_value: &PermissionRuleValue) -> bool {
    rule_value.tool_name == "Bash" && rule_value.rule_content.is_none()
}

/// Checks if a PowerShell allow rule is overly broad.
pub fn is_overly_broad_power_shell_allow_rule(rule_value: &PermissionRuleValue) -> bool {
    rule_value.tool_name == "PowerShell" && rule_value.rule_content.is_none()
}

/// Finds all overly broad Bash permissions.
pub fn find_overly_broad_bash_permissions(
    rules: &[PermissionRule],
    cli_allowed_tools: &[String],
) -> Vec<DangerousPermissionInfo> {
    let mut overly_broad = Vec::new();

    for rule in rules {
        if matches!(rule.rule_behavior, crate::types::permissions::PermissionBehavior::Allow)
            && is_overly_broad_bash_allow_rule(&rule.rule_value)
        {
            overly_broad.push(DangerousPermissionInfo {
                rule_value: rule.rule_value.clone(),
                source: rule.source.clone(),
                rule_display: "Bash(*)".to_string(),
                source_display: format_permission_source(&rule.source),
            });
        }
    }

    for tool_spec in cli_allowed_tools {
        let parsed = permission_rule_value_from_string(tool_spec);
        if is_overly_broad_bash_allow_rule(&parsed) {
            overly_broad.push(DangerousPermissionInfo {
                rule_value: parsed,
                source: PermissionRuleSource::CliArg,
                rule_display: "Bash(*)".to_string(),
                source_display: "--allowed-tools".to_string(),
            });
        }
    }

    overly_broad
}

/// Finds all overly broad PowerShell permissions.
pub fn find_overly_broad_power_shell_permissions(
    rules: &[PermissionRule],
    cli_allowed_tools: &[String],
) -> Vec<DangerousPermissionInfo> {
    let mut overly_broad = Vec::new();

    for rule in rules {
        if matches!(rule.rule_behavior, crate::types::permissions::PermissionBehavior::Allow)
            && is_overly_broad_power_shell_allow_rule(&rule.rule_value)
        {
            overly_broad.push(DangerousPermissionInfo {
                rule_value: rule.rule_value.clone(),
                source: rule.source.clone(),
                rule_display: "PowerShell(*)".to_string(),
                source_display: format_permission_source(&rule.source),
            });
        }
    }

    for tool_spec in cli_allowed_tools {
        let parsed = permission_rule_value_from_string(tool_spec);
        if is_overly_broad_power_shell_allow_rule(&parsed) {
            overly_broad.push(DangerousPermissionInfo {
                rule_value: parsed,
                source: PermissionRuleSource::CliArg,
                rule_display: "PowerShell(*)".to_string(),
                source_display: "--allowed-tools".to_string(),
            });
        }
    }

    overly_broad
}

/// Formats a permission source for display.
fn format_permission_source(source: &PermissionRuleSource) -> String {
    source.as_str().to_string()
}

/// Removes dangerous permissions from the in-memory context.
pub fn remove_dangerous_permissions(
    context: &ToolPermissionContext,
    dangerous_permissions: &[DangerousPermissionInfo],
) -> ToolPermissionContext {
    use super::permission_update::apply_permission_update;

    let mut updated_context = context.clone();

    for perm in dangerous_permissions {
        if !is_permission_update_destination(&perm.source) {
            continue;
        }

        let destination = match &perm.source {
            PermissionRuleSource::UserSettings => "userSettings",
            PermissionRuleSource::ProjectSettings => "projectSettings",
            PermissionRuleSource::LocalSettings => "localSettings",
            PermissionRuleSource::CliArg => "cliArg",
            PermissionRuleSource::Session => "session",
            _ => continue,
        };

        updated_context = apply_permission_update(updated_context, &crate::types::permissions::PermissionUpdate::RemoveRules {
            destination: destination_to_enum(destination),
            rules: vec![perm.rule_value.clone()],
            behavior: crate::types::permissions::PermissionBehavior::Allow,
        });
    }

    updated_context
}

/// Checks if a source can be a permission update destination.
fn is_permission_update_destination(source: &PermissionRuleSource) -> bool {
    matches!(
        source,
        PermissionRuleSource::UserSettings
            | PermissionRuleSource::ProjectSettings
            | PermissionRuleSource::LocalSettings
            | PermissionRuleSource::CliArg
            | PermissionRuleSource::Session
    )
}

/// Converts a destination string to enum.
fn destination_to_enum(dest: &str) -> crate::types::permissions::PermissionUpdateDestination {
    match dest {
        "userSettings" => crate::types::permissions::PermissionUpdateDestination::UserSettings,
        "projectSettings" => crate::types::permissions::PermissionUpdateDestination::ProjectSettings,
        "localSettings" => crate::types::permissions::PermissionUpdateDestination::LocalSettings,
        "session" => crate::types::permissions::PermissionUpdateDestination::Session,
        "cliArg" => crate::types::permissions::PermissionUpdateDestination::CliArg,
        _ => crate::types::permissions::PermissionUpdateDestination::Session,
    }
}

/// Strips dangerous permissions for auto mode.
pub fn strip_dangerous_permissions_for_auto_mode(
    context: &ToolPermissionContext,
) -> ToolPermissionContext {
    let mut rules = Vec::new();
    let all_allow_rules = [
        &context.always_allow_rules.user_settings,
        &context.always_allow_rules.project_settings,
        &context.always_allow_rules.local_settings,
        &context.always_allow_rules.flag_settings,
        &context.always_allow_rules.policy_settings,
        &context.always_allow_rules.cli_arg,
        &context.always_allow_rules.command,
        &context.always_allow_rules.session,
    ];

    for rule_strings in all_allow_rules {
        if let Some(strings) = rule_strings {
            for rule_string in strings {
                let rule_value = permission_rule_value_from_string(rule_string);
                rules.push(PermissionRule {
                    source: PermissionRuleSource::Session,
                    rule_behavior: crate::types::permissions::PermissionBehavior::Allow,
                    rule_value,
                });
            }
        }
    }

    let dangerous_permissions = find_dangerous_classifier_permissions(&rules, &[]);
    if dangerous_permissions.is_empty() {
        let mut ctx = context.clone();
        if ctx.stripped_dangerous_rules.is_none() {
            ctx.stripped_dangerous_rules = Some(ToolPermissionRulesBySource {
                user_settings: None, project_settings: None, local_settings: None,
                flag_settings: None, policy_settings: None, cli_arg: None,
                command: None, session: None,
            });
        }
        return ctx;
    }

    for perm in &dangerous_permissions {
        log::debug!(
            "Ignoring dangerous permission {} from {} (bypasses classifier)",
            perm.rule_display,
            perm.source_display,
        );
    }

    let mut stripped = ToolPermissionRulesBySource {
        user_settings: None, project_settings: None, local_settings: None,
        flag_settings: None, policy_settings: None, cli_arg: None,
        command: None, session: None,
    };

    for perm in &dangerous_permissions {
        if !is_permission_update_destination(&perm.source) {
            continue;
        }
        let rule_string = permission_rule_value_to_string(&perm.rule_value);
        match perm.source {
            PermissionRuleSource::UserSettings => {
                stripped.user_settings.get_or_insert_with(Vec::new).push(rule_string);
            }
            PermissionRuleSource::ProjectSettings => {
                stripped.project_settings.get_or_insert_with(Vec::new).push(rule_string);
            }
            PermissionRuleSource::LocalSettings => {
                stripped.local_settings.get_or_insert_with(Vec::new).push(rule_string);
            }
            PermissionRuleSource::CliArg => {
                stripped.cli_arg.get_or_insert_with(Vec::new).push(rule_string);
            }
            PermissionRuleSource::Session => {
                stripped.session.get_or_insert_with(Vec::new).push(rule_string);
            }
            _ => {}
        }
    }

    let mut result = remove_dangerous_permissions(context, &dangerous_permissions);
    result.stripped_dangerous_rules = Some(stripped);
    result
}

/// Restores dangerous permissions previously stripped.
pub fn restore_dangerous_permissions(context: &ToolPermissionContext) -> ToolPermissionContext {
    use super::permission_update::apply_permission_update;

    let stash = match &context.stripped_dangerous_rules {
        Some(s) => s,
        None => return context.clone(),
    };

    let mut result = context.clone();

    let all_rules = [
        (&stash.user_settings, "userSettings"),
        (&stash.project_settings, "projectSettings"),
        (&stash.local_settings, "localSettings"),
        (&stash.cli_arg, "cliArg"),
        (&stash.session, "session"),
    ];

    for (rule_strings, dest_str) in all_rules {
        if let Some(strings) = rule_strings {
            if strings.is_empty() {
                continue;
            }
            let rules: Vec<PermissionRuleValue> = strings
                .iter()
                .map(|s| permission_rule_value_from_string(s))
                .collect();
            result = apply_permission_update(result, &crate::types::permissions::PermissionUpdate::AddRules {
                destination: destination_to_enum(dest_str),
                rules,
                behavior: crate::types::permissions::PermissionBehavior::Allow,
            });
        }
    }

    result.stripped_dangerous_rules = None;
    result
}
