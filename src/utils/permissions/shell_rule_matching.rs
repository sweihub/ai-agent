// Source: ~/claudecode/openclaudecode/src/utils/permissions/shellRuleMatching.ts
#![allow(dead_code)]

//! Shared permission rule matching utilities for shell tools.
//!
//! Extracts common logic for:
//! - Parsing permission rules (exact, prefix, wildcard)
//! - Matching commands against rules
//! - Generating permission suggestions

use crate::types::permissions::PermissionUpdate;

/// Null-byte sentinel placeholders for wildcard pattern escaping.
const ESCAPED_STAR_PLACEHOLDER: &str = "\x00ESCAPED_STAR\x00";
const ESCAPED_BACKSLASH_PLACEHOLDER: &str = "\x00ESCAPED_BACKSLASH\x00";

/// Parsed permission rule.
#[derive(Debug, Clone)]
pub enum ShellPermissionRule {
    Exact { command: String },
    Prefix { prefix: String },
    Wildcard { pattern: String },
}

/// Extract prefix from legacy :* syntax (e.g., "npm:*" -> "npm").
pub fn permission_rule_extract_prefix(permission_rule: &str) -> Option<String> {
    if permission_rule.ends_with(":*") {
        Some(permission_rule[..permission_rule.len() - 2].to_string())
    } else {
        None
    }
}

/// Checks if a pattern contains unescaped wildcards.
pub fn has_wildcards(pattern: &str) -> bool {
    if pattern.ends_with(":*") {
        return false;
    }

    let chars: Vec<char> = pattern.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c == '*' {
            let mut backslash_count = 0;
            let mut j = i as i32 - 1;
            while j >= 0 && chars[j as usize] == '\\' {
                backslash_count += 1;
                j -= 1;
            }
            if backslash_count % 2 == 0 {
                return true;
            }
        }
    }
    false
}

/// Matches a command against a wildcard pattern.
pub fn match_wildcard_pattern(
    pattern: &str,
    command: &str,
    case_insensitive: bool,
) -> bool {
    let trimmed = pattern.trim();

    // Process escape sequences
    let mut processed = String::new();
    let chars: Vec<char> = trimmed.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                '*' => {
                    processed.push_str(ESCAPED_STAR_PLACEHOLDER);
                    i += 2;
                    continue;
                }
                '\\' => {
                    processed.push_str(ESCAPED_BACKSLASH_PLACEHOLDER);
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }
        processed.push(chars[i]);
        i += 1;
    }

    // Escape regex special characters except *
    let escaped = processed
        .replace('.', "\\.")
        .replace('+', "\\+")
        .replace('?', "\\?")
        .replace('^', "\\^")
        .replace('$', "\\$")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"");

    // Convert unescaped * to .*
    let with_wildcards = escaped.replace('*', ".*");

    // Convert placeholders back
    let regex_pattern = with_wildcards
        .replace(ESCAPED_STAR_PLACEHOLDER, "\\*")
        .replace(ESCAPED_BACKSLASH_PLACEHOLDER, "\\\\");

    // Handle trailing ' *' (space + single wildcard) — make args optional
    let unescaped_star_count = processed.matches('*').count();
    let mut final_pattern = regex_pattern;
    if final_pattern.ends_with(" .*") && unescaped_star_count == 1 {
        let without_trailing = &final_pattern[..final_pattern.len() - 3];
        final_pattern = format!("{}( .*)?", without_trailing);
    }

    let flags = if case_insensitive { "(?i)" } else { "" };
    let regex_str = format!("{}^{}$", flags, final_pattern);

    match regex::Regex::new(&regex_str) {
        Ok(re) => re.is_match(command),
        Err(_) => false,
    }
}

/// Parses a permission rule string into a structured rule.
pub fn parse_permission_rule(permission_rule: &str) -> ShellPermissionRule {
    // Check legacy :* prefix syntax
    if let Some(prefix) = permission_rule_extract_prefix(permission_rule) {
        return ShellPermissionRule::Prefix { prefix };
    }

    // Check wildcard syntax
    if has_wildcards(permission_rule) {
        return ShellPermissionRule::Wildcard {
            pattern: permission_rule.to_string(),
        };
    }

    // Exact match
    ShellPermissionRule::Exact {
        command: permission_rule.to_string(),
    }
}

/// Generates permission update suggestion for an exact command match.
pub fn suggestion_for_exact_command(tool_name: &str, command: &str) -> Vec<PermissionUpdate> {
    vec![PermissionUpdate::AddRules {
        rules: vec![crate::types::permissions::PermissionRuleValue {
            tool_name: tool_name.to_string(),
            rule_content: Some(command.to_string()),
        }],
        behavior: crate::types::permissions::PermissionBehavior::Allow,
        destination: crate::types::permissions::PermissionUpdateDestination::LocalSettings,
    }]
}

/// Generates permission update suggestion for a prefix match.
pub fn suggestion_for_prefix(tool_name: &str, prefix: &str) -> Vec<PermissionUpdate> {
    vec![PermissionUpdate::AddRules {
        rules: vec![crate::types::permissions::PermissionRuleValue {
            tool_name: tool_name.to_string(),
            rule_content: Some(format!("{}:*", prefix)),
        }],
        behavior: crate::types::permissions::PermissionBehavior::Allow,
        destination: crate::types::permissions::PermissionUpdateDestination::LocalSettings,
    }]
}
