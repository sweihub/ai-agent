// Source: ~/claudecode/openclaudecode/src/utils/permissions/permissionRuleParser.ts
#![allow(dead_code)]

//! Permission rule parsing — converts rule strings to/from structured values.

use crate::types::permissions::PermissionRuleValue;

/// Maps legacy tool names to their current canonical names.
fn legacy_tool_name_aliases() -> &'static [(&'static str, &'static str)] {
    &[
        ("Task", "Agent"),
        ("KillShell", "TaskStop"),
        ("AgentOutputTool", "TaskOutput"),
        ("BashOutputTool", "TaskOutput"),
    ]
}

/// Normalizes a legacy tool name to its canonical form.
pub fn normalize_legacy_tool_name(name: &str) -> String {
    for (legacy, canonical) in legacy_tool_name_aliases() {
        if *legacy == name {
            return canonical.to_string();
        }
    }
    name.to_string()
}

/// Gets all legacy tool names that map to a canonical name.
pub fn get_legacy_tool_names(canonical_name: &str) -> Vec<String> {
    legacy_tool_name_aliases()
        .iter()
        .filter(|(_, canonical)| *canonical == canonical_name)
        .map(|(legacy, _)| legacy.to_string())
        .collect()
}

/// Escapes special characters in rule content for safe storage.
pub fn escape_rule_content(content: &str) -> String {
    content
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

/// Unescapes special characters in rule content after parsing.
pub fn unescape_rule_content(content: &str) -> String {
    content
        .replace("\\(", "(")
        .replace("\\)", ")")
        .replace("\\\\", "\\")
}

/// Finds the index of the first unescaped occurrence of a character.
fn find_first_unescaped_char(s: &str, ch: char) -> Option<usize> {
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c == ch {
            let mut backslash_count = 0;
            let mut j = i as i32 - 1;
            while j >= 0 && chars[j as usize] == '\\' {
                backslash_count += 1;
                j -= 1;
            }
            if backslash_count % 2 == 0 {
                return Some(i);
            }
        }
    }
    None
}

/// Finds the index of the last unescaped occurrence of a character.
fn find_last_unescaped_char(s: &str, ch: char) -> Option<usize> {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    for i in (0..len).rev() {
        if chars[i] == ch {
            let mut backslash_count = 0;
            let mut j = i as i32 - 1;
            while j >= 0 && chars[j as usize] == '\\' {
                backslash_count += 1;
                j -= 1;
            }
            if backslash_count % 2 == 0 {
                return Some(i);
            }
        }
    }
    None
}

/// Parses a permission rule string into its components.
/// Format: "ToolName" or "ToolName(content)"
pub fn permission_rule_value_from_string(rule_string: &str) -> PermissionRuleValue {
    let open_paren_idx = find_first_unescaped_char(rule_string, '(');

    if open_paren_idx.is_none() {
        return PermissionRuleValue {
            tool_name: normalize_legacy_tool_name(rule_string),
            rule_content: None,
        };
    }

    let open_idx = open_paren_idx.unwrap();
    let close_idx = find_last_unescaped_char(rule_string, ')');

    if close_idx.is_none() || close_idx.unwrap() <= open_idx {
        return PermissionRuleValue {
            tool_name: normalize_legacy_tool_name(rule_string),
            rule_content: None,
        };
    }

    let close_idx = close_idx.unwrap();

    if close_idx != rule_string.len() - 1 {
        return PermissionRuleValue {
            tool_name: normalize_legacy_tool_name(rule_string),
            rule_content: None,
        };
    }

    let tool_name = &rule_string[..open_idx];
    let raw_content = &rule_string[open_idx + 1..close_idx];

    if tool_name.is_empty() {
        return PermissionRuleValue {
            tool_name: normalize_legacy_tool_name(rule_string),
            rule_content: None,
        };
    }

    if raw_content.is_empty() || raw_content == "*" {
        return PermissionRuleValue {
            tool_name: normalize_legacy_tool_name(tool_name),
            rule_content: None,
        };
    }

    let rule_content = unescape_rule_content(raw_content);
    PermissionRuleValue {
        tool_name: normalize_legacy_tool_name(tool_name),
        rule_content: Some(rule_content),
    }
}

/// Converts a permission rule value to its string representation.
pub fn permission_rule_value_to_string(rule_value: &PermissionRuleValue) -> String {
    if rule_value.rule_content.is_none() {
        return rule_value.tool_name.clone();
    }
    let escaped_content = escape_rule_content(rule_value.rule_content.as_ref().unwrap());
    format!("{}({})", rule_value.tool_name, escaped_content)
}
