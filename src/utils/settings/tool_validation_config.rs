// Source: ~/claudecode/openclaudecode/src/utils/settings/toolValidationConfig.ts
//! Tool validation configuration for permission rule patterns.

/// Tools that accept file glob patterns (e.g., *.ts, src/**)
const FILE_PATTERN_TOOLS: &[&str] = &[
    "Read", "Write", "Edit", "Glob", "NotebookRead", "NotebookEdit",
];

/// Tools that accept bash wildcard patterns and legacy :* prefix syntax
const BASH_PREFIX_TOOLS: &[&str] = &["Bash"];

/// Check if a tool uses file glob patterns
pub fn is_file_pattern_tool(tool_name: &str) -> bool {
    FILE_PATTERN_TOOLS
        .iter()
        .any(|t| t.eq_ignore_ascii_case(tool_name))
}

/// Check if a tool uses bash prefix patterns
pub fn is_bash_prefix_tool(tool_name: &str) -> bool {
    BASH_PREFIX_TOOLS
        .iter()
        .any(|t| t.eq_ignore_ascii_case(tool_name))
}

/// Custom validation result
pub type CustomValidationResult = CustomValidateResult;

/// Result of a custom validation check
#[derive(Debug, Clone)]
pub struct CustomValidateResult {
    pub valid: bool,
    pub error: Option<String>,
    pub suggestion: Option<String>,
    pub examples: Option<Vec<String>>,
}

/// Get custom validation for a specific tool
pub fn get_custom_validation(content: &str) -> Option<CustomValidateResult> {
    // WebSearch doesn't support wildcards or complex patterns
    if content.starts_with("WebSearch") {
        let check = if content.contains('*') || content.contains('?') {
            CustomValidateResult {
                valid: false,
                error: Some("WebSearch does not support wildcards".into()),
                suggestion: Some("Use exact search terms without * or ?".into()),
                examples: Some(vec![
                    "WebSearch(claude ai)".into(),
                    "WebSearch(typescript tutorial)".into(),
                ]),
            }
        } else {
            CustomValidateResult {
                valid: true,
                error: None,
                suggestion: None,
                examples: None,
            }
        };
        return Some(check);
    }

    // WebFetch uses domain: prefix for hostname-based permissions
    if content.starts_with("WebFetch") {
        if content.contains("://") || content.starts_with("http") {
            return Some(CustomValidateResult {
                valid: false,
                error: Some("WebFetch permissions use domain format, not URLs".into()),
                suggestion: Some("Use \"domain:hostname\" format".into()),
                examples: Some(vec![
                    "WebFetch(domain:example.com)".into(),
                    "WebFetch(domain:github.com)".into(),
                ]),
            });
        }
        if !content.contains("domain:") {
            return Some(CustomValidateResult {
                valid: false,
                error: Some("WebFetch permissions must use \"domain:\" prefix".into()),
                suggestion: Some("Use \"domain:hostname\" format".into()),
                examples: Some(vec![
                    "WebFetch(domain:example.com)".into(),
                    "WebFetch(domain:*.google.com)".into(),
                ]),
            });
        }
        return Some(CustomValidateResult {
            valid: true,
            error: None,
            suggestion: None,
            examples: None,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_file_pattern_tool() {
        assert!(is_file_pattern_tool("Read"));
        assert!(is_file_pattern_tool("Write"));
        assert!(is_file_pattern_tool("Edit"));
        assert!(is_file_pattern_tool("Glob"));
        assert!(!is_file_pattern_tool("Bash"));
        assert!(!is_file_pattern_tool("WebSearch"));
    }

    #[test]
    fn test_is_bash_prefix_tool() {
        assert!(is_bash_prefix_tool("Bash"));
        assert!(!is_bash_prefix_tool("Read"));
        assert!(!is_bash_prefix_tool("WebSearch"));
    }

    #[test]
    fn test_custom_validation_websearch() {
        let result = get_custom_validation("WebSearch(claude ai)");
        assert!(result.is_none() || result.unwrap().valid);

        // Wildcards should fail
        let result = get_custom_validation("WebSearch(claude*)");
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(!result.valid);
    }

    #[test]
    fn test_custom_validation_webfetch() {
        // Valid domain format
        let result = get_custom_validation("WebFetch(domain:example.com)");
        assert!(result.is_some());
        assert!(result.unwrap().valid);

        // URL format should fail
        let result = get_custom_validation("WebFetch(https://example.com)");
        assert!(result.is_some());
        assert!(!result.unwrap().valid);
    }

    #[test]
    fn test_no_custom_validation_for_other_tools() {
        assert!(get_custom_validation("Bash(npm install)").is_none());
        assert!(get_custom_validation("Read(*.ts)").is_none());
    }
}
