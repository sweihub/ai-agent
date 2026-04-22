// Source: ~/claudecode/openclaudecode/src/utils/permissions/shadowedRuleDetection.ts
#![allow(dead_code)]

//! Shadowed rule detection — identifies unreachable permission rules.

use super::permissions::{get_allow_rules, get_ask_rules, get_deny_rules};
use crate::types::permissions::{PermissionRule, PermissionRuleSource, ToolPermissionContext};

/// Type of shadowing that makes a rule unreachable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowType {
    Ask,
    Deny,
}

/// Represents an unreachable permission rule with explanation.
pub struct UnreachableRule {
    pub rule: PermissionRule,
    pub reason: String,
    pub shadowed_by: PermissionRule,
    pub shadow_type: ShadowType,
    pub fix: String,
}

/// Options for detecting unreachable rules.
pub struct DetectUnreachableRulesOptions {
    /// Whether sandbox auto-allow is enabled for Bash commands.
    pub sandbox_auto_allow_enabled: bool,
}

/// Checks if a permission rule source is shared (visible to other users).
pub fn is_shared_setting_source(source: &PermissionRuleSource) -> bool {
    matches!(
        source,
        PermissionRuleSource::ProjectSettings
            | PermissionRuleSource::PolicySettings
            | PermissionRuleSource::Command
    )
}

/// Formats a rule source for display.
fn format_source(source: &PermissionRuleSource) -> String {
    source.as_str().to_string()
}

/// Generates a fix suggestion based on the shadow type.
fn generate_fix_suggestion(
    shadow_type: &ShadowType,
    shadowing_rule: &PermissionRule,
    shadowed_rule: &PermissionRule,
) -> String {
    let shadowing_source = format_source(&shadowing_rule.source);
    let shadowed_source = format_source(&shadowed_rule.source);
    let tool_name = &shadowing_rule.rule_value.tool_name;

    match shadow_type {
        ShadowType::Deny => {
            format!(
                "Remove the \"{}\" deny rule from {}, or remove the specific allow rule from {}",
                tool_name, shadowing_source, shadowed_source
            )
        }
        ShadowType::Ask => {
            format!(
                "Remove the \"{}\" ask rule from {}, or remove the specific allow rule from {}",
                tool_name, shadowing_source, shadowed_source
            )
        }
    }
}

/// Checks if an allow rule is shadowed by an ask rule.
fn is_allow_rule_shadowed_by_ask_rule(
    allow_rule: &PermissionRule,
    ask_rules: &[PermissionRule],
    options: &DetectUnreachableRulesOptions,
) -> Option<(PermissionRule, ShadowType)> {
    let rule_content = &allow_rule.rule_value.rule_content;

    // Only check allow rules with specific content
    if rule_content.is_none() {
        return None;
    }

    let tool_name = &allow_rule.rule_value.tool_name;

    // Find any tool-wide ask rule for the same tool
    let shadowing_ask_rule = ask_rules.iter().find(|ask_rule| {
        ask_rule.rule_value.tool_name == *tool_name && ask_rule.rule_value.rule_content.is_none()
    });

    let Some(shadowing_ask_rule) = shadowing_ask_rule else {
        return None;
    };

    // Special case: Bash with sandbox auto-allow from personal settings
    if tool_name == "Bash" && options.sandbox_auto_allow_enabled {
        if !is_shared_setting_source(&shadowing_ask_rule.source) {
            return None;
        }
    }

    Some((shadowing_ask_rule.clone(), ShadowType::Ask))
}

/// Checks if an allow rule is shadowed by a deny rule.
fn is_allow_rule_shadowed_by_deny_rule(
    allow_rule: &PermissionRule,
    deny_rules: &[PermissionRule],
) -> Option<(PermissionRule, ShadowType)> {
    let rule_content = &allow_rule.rule_value.rule_content;

    // Only check allow rules with specific content
    if rule_content.is_none() {
        return None;
    }

    let tool_name = &allow_rule.rule_value.tool_name;

    // Find any tool-wide deny rule for the same tool
    let shadowing_deny_rule = deny_rules.iter().find(|deny_rule| {
        deny_rule.rule_value.tool_name == *tool_name && deny_rule.rule_value.rule_content.is_none()
    });

    shadowing_deny_rule.map(|rule| (rule.clone(), ShadowType::Deny))
}

/// Detects all unreachable permission rules in the given context.
pub fn detect_unreachable_rules(
    context: &ToolPermissionContext,
    options: &DetectUnreachableRulesOptions,
) -> Vec<UnreachableRule> {
    let mut unreachable: Vec<UnreachableRule> = Vec::new();

    let allow_rules = get_allow_rules(context);
    let ask_rules = get_ask_rules(context);
    let deny_rules = get_deny_rules(context);

    for allow_rule in allow_rules {
        // Check deny shadowing first (more severe)
        if let Some((shadowing_rule, shadow_type)) =
            is_allow_rule_shadowed_by_deny_rule(&allow_rule, &deny_rules)
        {
            let shadow_source = format_source(&shadowing_rule.source);
            unreachable.push(UnreachableRule {
                rule: allow_rule.clone(),
                reason: format!(
                    "Blocked by \"{}\" deny rule (from {})",
                    shadowing_rule.rule_value.tool_name, shadow_source
                ),
                shadowed_by: shadowing_rule.clone(),
                shadow_type,
                fix: generate_fix_suggestion(&shadow_type, &shadowing_rule, &allow_rule),
            });
            continue;
        }

        // Check ask shadowing
        if let Some((shadowing_rule, shadow_type)) =
            is_allow_rule_shadowed_by_ask_rule(&allow_rule, &ask_rules, options)
        {
            let shadow_source = format_source(&shadowing_rule.source);
            unreachable.push(UnreachableRule {
                rule: allow_rule.clone(),
                reason: format!(
                    "Shadowed by \"{}\" ask rule (from {})",
                    shadowing_rule.rule_value.tool_name, shadow_source
                ),
                shadowed_by: shadowing_rule.clone(),
                shadow_type,
                fix: generate_fix_suggestion(&shadow_type, &shadowing_rule, &allow_rule),
            });
        }
    }

    unreachable
}
