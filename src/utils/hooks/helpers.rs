// Source: /data/home/swei/claudecode/openclaudecode/src/utils/dxt/helpers.ts
//! Hook helper functions for structured output and argument substitution.
//!
//! This module provides utilities for:
//! - Creating structured output tools for hook responses
//! - Substituting $ARGUMENTS placeholders in prompts

use serde::{Deserialize, Serialize};

/// Schema for hook responses (shared by prompt and agent hooks)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookResponse {
    /// Whether the condition was met
    pub ok: bool,
    /// Reason, if the condition was not met
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl HookResponse {
    /// Create a successful response
    pub fn ok() -> Self {
        Self {
            ok: true,
            reason: None,
        }
    }

    /// Create a successful response with reason
    pub fn ok_with_reason(reason: impl Into<String>) -> Self {
        Self {
            ok: true,
            reason: Some(reason.into()),
        }
    }

    /// Create a failure response
    pub fn not_ok(reason: impl Into<String>) -> Self {
        Self {
            ok: false,
            reason: Some(reason.into()),
        }
    }
}

/// Hook response schema as JSON Schema (for tool input)
pub fn hook_response_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "ok": {
                "type": "boolean",
                "description": "Whether the condition was met"
            },
            "reason": {
                "type": "string",
                "description": "Reason, if the condition was not met"
            }
        },
        "required": ["ok"],
        "additionalProperties": false
    })
}

/// Parse an arguments string into a vector of individual arguments.
/// Uses simple whitespace splitting for shell-like argument parsing.
///
/// Examples:
/// - "foo bar baz" => ["foo", "bar", "baz"]
/// - "foo \"hello world\" baz" => preserves quoted handling would need shell-quote
pub fn parse_arguments(args: &str) -> Vec<String> {
    if args.trim().is_empty() {
        return vec![];
    }
    args.split_whitespace().map(|s| s.to_string()).collect()
}

/// Parse argument names from the frontmatter 'arguments' field.
/// Accepts either a space-separated string or a vector of strings.
///
/// Examples:
/// - "foo bar baz" => ["foo", "bar", "baz"]
/// - ["foo", "bar"] => ["foo", "bar"]
pub fn parse_argument_names(argument_names: Option<Vec<String>>) -> Vec<String> {
    match argument_names {
        Some(names) => names
            .into_iter()
            .filter(|name| !name.trim().is_empty() && !name.chars().all(|c| c.is_ascii_digit()))
            .collect(),
        None => vec![],
    }
}

/// Substitute $ARGUMENTS placeholders in content with actual argument values.
///
/// Supports:
/// - $ARGUMENTS - replaced with the full arguments string
/// - $ARGUMENTS[0], $ARGUMENTS[1], etc. - replaced with individual indexed arguments
/// - $0, $1, etc. - shorthand for $ARGUMENTS[0], $ARGUMENTS[1]
/// - Named arguments (e.g., $foo, $bar) - when argument names are defined
///
/// # Arguments
/// * `content` - The content containing placeholders
/// * `args` - The raw arguments string (None means no args provided)
/// * `append_if_no_placeholder` - If true and no placeholders are found, appends "ARGUMENTS: {args}" to content
/// * `argument_names` - Optional vector of named arguments (e.g., ["foo", "bar"]) that map to indexed positions
///
/// # Returns
/// The content with placeholders substituted
pub fn substitute_arguments(
    content: &str,
    args: Option<&str>,
    append_if_no_placeholder: bool,
    argument_names: Vec<String>,
) -> String {
    // None means no args provided - return content unchanged
    // empty string is a valid input that should replace placeholders with empty
    let args = match args {
        Some(a) => a,
        None => return content.to_string(),
    };

    let parsed_args = parse_arguments(args);
    let original_content = content.to_string();
    let mut content = original_content.clone();

    // Replace named arguments (e.g., $foo, $bar) with their values
    // Named arguments map to positions: argument_names[0] -> parsed_args[0], etc.
    for (i, name) in argument_names.iter().enumerate() {
        if name.is_empty() {
            continue;
        }

        // Manual replacement to avoid look-around regex issues
        // Match $name followed by non-word/bracket character or end of string
        let needle = format!("${}", name);
        let mut search_start = 0;
        while let Some(dollar_pos) = content[search_start..].find(&needle) {
            let actual_pos = search_start + dollar_pos;
            let after_name_pos = actual_pos + needle.len();
            let after = content
                .get(after_name_pos..after_name_pos + 1)
                .unwrap_or("");
            // Only replace if followed by non-word/non-bracket char or end
            if after.is_empty()
                || (!after
                    .chars()
                    .next()
                    .map(|c| c.is_alphanumeric() || c == '_' || c == '[')
                    .unwrap_or(true))
            {
                let replacement = parsed_args.get(i).map(|s| s.as_str()).unwrap_or("");
                content = format!(
                    "{}{}{}",
                    &content[..actual_pos],
                    replacement,
                    &content[after_name_pos..]
                );
                search_start = actual_pos + replacement.len();
            } else {
                search_start = after_name_pos;
            }
        }
    }

    // Replace indexed arguments ($ARGUMENTS[0], $ARGUMENTS[1], etc.)
    if let Ok(re) = regex::Regex::new(r"\$ARGUMENTS\[(\d+)\]") {
        content = re
            .replace_all(&content, |caps: &regex::Captures| {
                let index: usize = caps[1].parse().unwrap_or(0);
                parsed_args.get(index).map(|s| s.as_str()).unwrap_or("")
            })
            .to_string();
    }

    // Replace shorthand indexed arguments ($0, $1, etc.)
    // Use a custom loop to preserve surrounding characters
    let mut result = content.to_string();
    let mut search_start = 0;
    while let Some(dollar_pos) = result[search_start..].find('$') {
        let actual_pos = search_start + dollar_pos;
        // Check if this is a digit after the $
        if let Some(ch) = result.chars().nth(actual_pos + 1) {
            if ch.is_ascii_digit() {
                // Extract the full number
                let rest = &result[actual_pos + 1..];
                let num_chars: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(index) = num_chars.parse::<usize>() {
                    let after_num_pos = actual_pos + 1 + num_chars.len();
                    // Check what follows: must be end of string or non-word/non-digit
                    let after = result.get(after_num_pos..after_num_pos + 1).unwrap_or("");
                    if after.is_empty()
                        || (!after
                            .chars()
                            .next()
                            .map(|c| c.is_alphanumeric())
                            .unwrap_or(true))
                    {
                        // Replace just the $digits, preserving the delimiter
                        let replacement = parsed_args.get(index).map(|s| s.as_str()).unwrap_or("");
                        result = format!(
                            "{}{}{}",
                            &result[..actual_pos],
                            replacement,
                            &result[after_num_pos..]
                        );
                        search_start = actual_pos + replacement.len();
                        continue;
                    }
                }
            }
        }
        search_start = actual_pos + 1;
    }
    content = result;

    // Replace $ARGUMENTS with the full arguments string
    content = content.replace("$ARGUMENTS", args);

    // If no placeholders were found and append_if_no_placeholder is true, append
    // But only if args is non-empty (empty string means command invoked with no args)
    if content == original_content && append_if_no_placeholder && !args.is_empty() {
        content = format!("{}\n\nARGUMENTS: {}", content, args);
    }

    content
}

/// Add hook input JSON to prompt, either replacing $ARGUMENTS placeholder or appending.
/// Also supports indexed arguments like $ARGUMENTS[0], $ARGUMENTS[1], or shorthand $0, $1, etc.
pub fn add_arguments_to_prompt(prompt: &str, json_input: &str) -> String {
    substitute_arguments(prompt, Some(json_input), true, vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_response_ok() {
        let resp = HookResponse::ok();
        assert!(resp.ok);
        assert!(resp.reason.is_none());
    }

    #[test]
    fn test_hook_response_ok_with_reason() {
        let resp = HookResponse::ok_with_reason("success");
        assert!(resp.ok);
        assert_eq!(resp.reason, Some("success".to_string()));
    }

    #[test]
    fn test_hook_response_not_ok() {
        let resp = HookResponse::not_ok("failed");
        assert!(!resp.ok);
        assert_eq!(resp.reason, Some("failed".to_string()));
    }

    #[test]
    fn test_parse_arguments() {
        assert_eq!(parse_arguments("foo bar baz"), vec!["foo", "bar", "baz"]);
        assert_eq!(parse_arguments(""), Vec::<&str>::new());
        assert_eq!(parse_arguments("  "), Vec::<&str>::new());
    }

    #[test]
    fn test_parse_argument_names() {
        assert_eq!(
            parse_argument_names(Some(vec!["foo".to_string(), "bar".to_string()])),
            vec!["foo", "bar"]
        );
        assert_eq!(parse_argument_names(None), Vec::<&str>::new());
        // Filter out numeric-only names
        assert_eq!(
            parse_argument_names(Some(vec!["0".to_string(), "foo".to_string()])),
            vec!["foo"]
        );
    }

    #[test]
    fn test_substitute_arguments_basic() {
        let result = substitute_arguments("Hello $ARGUMENTS", Some("world"), true, vec![]);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_substitute_arguments_indexed() {
        let result =
            substitute_arguments("$ARGUMENTS[0] $ARGUMENTS[1]", Some("foo bar"), true, vec![]);
        assert_eq!(result, "foo bar");
    }

    #[test]
    fn test_substitute_arguments_shorthand() {
        let result = substitute_arguments("$0 $1", Some("foo bar"), true, vec![]);
        assert_eq!(result, "foo bar");
    }

    #[test]
    fn test_substitute_arguments_named() {
        let result = substitute_arguments(
            "$greeting $name",
            Some("hello world"),
            true,
            vec!["greeting".to_string(), "name".to_string()],
        );
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_substitute_arguments_no_args() {
        let result = substitute_arguments("Hello $ARGUMENTS", None, true, vec![]);
        assert_eq!(result, "Hello $ARGUMENTS");
    }

    #[test]
    fn test_substitute_arguments_append() {
        let result = substitute_arguments("Hello", Some("world"), true, vec![]);
        assert!(result.contains("ARGUMENTS: world"));
    }

    #[test]
    fn test_add_arguments_to_prompt() {
        let result = add_arguments_to_prompt("Run: $ARGUMENTS", "ls -la");
        assert!(result.contains("ls -la"));
    }
}
