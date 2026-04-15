// Source: ~/claudecode/openclaudecode/src/services/compact/apiMicrocompact.ts
//! API context management - native API context editing strategies.
//!
//! Generates ContextManagementConfig with ContextEditStrategy entries for
//! the API's native context editing feature.

use crate::tools::config_tools::{
    FILE_EDIT_TOOL_NAME, FILE_READ_TOOL_NAME, FILE_WRITE_TOOL_NAME, GLOB_TOOL_NAME, GREP_TOOL_NAME,
    NOTEBOOK_EDIT_TOOL_NAME, POWERSHELL_TOOL_NAME, WEB_FETCH_TOOL_NAME, WEB_SEARCH_TOOL_NAME,
    BASH_TOOL_NAME,
};
use crate::utils::env_utils;

/// Default values for context management strategies
const DEFAULT_MAX_INPUT_TOKENS: usize = 180_000;
const DEFAULT_TARGET_INPUT_TOKENS: usize = 40_000;

/// Tools whose results can be cleared
fn tools_clearable_results() -> Vec<&'static str> {
    vec![
        BASH_TOOL_NAME,
        POWERSHELL_TOOL_NAME,
        GLOB_TOOL_NAME,
        GREP_TOOL_NAME,
        FILE_READ_TOOL_NAME,
        WEB_FETCH_TOOL_NAME,
        WEB_SEARCH_TOOL_NAME,
    ]
}

/// Tools whose uses can be cleared (edit-type tools)
fn tools_clearable_uses() -> Vec<&'static str> {
    vec![
        FILE_EDIT_TOOL_NAME,
        FILE_WRITE_TOOL_NAME,
        NOTEBOOK_EDIT_TOOL_NAME,
    ]
}

/// Context edit strategy type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContextEditStrategy {
    /// Clear old tool uses/results based on input token thresholds
    ClearToolUses20250919 {
        /// Trigger threshold
        #[serde(skip_serializing_if = "Option::is_none")]
        trigger: Option<TriggerConfig>,
        /// How many tool results to keep
        #[serde(skip_serializing_if = "Option::is_none")]
        keep: Option<KeepConfig>,
        /// Which tools to clear results for
        #[serde(skip_serializing_if = "Option::is_none")]
        clear_tool_inputs: Option<Vec<String>>,
        /// Which tools to exclude from clearing
        #[serde(skip_serializing_if = "Option::is_none")]
        exclude_tools: Option<Vec<String>>,
        /// Minimum amount to clear
        #[serde(skip_serializing_if = "Option::is_none")]
        clear_at_least: Option<TriggerConfig>,
    },
    /// Clear thinking blocks, optionally keeping last N turns
    ClearThinking20251015 {
        /// How many thinking turns to keep
        keep: ThinkingKeepConfig,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TriggerConfig {
    #[serde(rename = "type")]
    pub trigger_type: String,
    pub value: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeepConfig {
    #[serde(rename = "type")]
    pub keep_type: String,
    pub value: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ThinkingKeepConfig {
    KeepAll,
    KeepTurns {
        #[serde(rename = "type")]
        keep_type: String,
        value: usize,
    },
}

/// Context management configuration wrapper
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextManagementConfig {
    pub edits: Vec<ContextEditStrategy>,
}

/// Options for API context management
pub struct ContextManagementOptions {
    pub has_thinking: bool,
    pub is_redact_thinking_active: bool,
    pub clear_all_thinking: bool,
}

impl Default for ContextManagementOptions {
    fn default() -> Self {
        Self {
            has_thinking: false,
            is_redact_thinking_active: false,
            clear_all_thinking: false,
        }
    }
}

/// Get API context management configuration.
/// Returns None if no context management strategies are applicable.
pub fn get_api_context_management(
    options: Option<ContextManagementOptions>,
) -> Option<ContextManagementConfig> {
    let opts = options.unwrap_or_default();
    let mut strategies = Vec::new();

    // Preserve thinking blocks in previous assistant turns
    // Skip when redact-thinking is active (redacted blocks have no model-visible content)
    // When clear_all_thinking is set (>1h idle = cache miss), keep only the last thinking turn
    if opts.has_thinking && !opts.is_redact_thinking_active {
        let keep = if opts.clear_all_thinking {
            ThinkingKeepConfig::KeepTurns {
                keep_type: "thinking_turns".to_string(),
                value: 1,
            }
        } else {
            ThinkingKeepConfig::KeepAll
        };
        strategies.push(ContextEditStrategy::ClearThinking20251015 { keep });
    }

    // Tool clearing strategies - only for ant builds
    // For external builds, skip tool clearing
    let use_clear_tool_results = env_utils::is_env_truthy(
        std::env::var("USE_API_CLEAR_TOOL_RESULTS").ok().as_deref(),
    );
    let use_clear_tool_uses = env_utils::is_env_truthy(
        std::env::var("USE_API_CLEAR_TOOL_USES").ok().as_deref(),
    );

    // If no tool clearing strategy is enabled, return early
    if !use_clear_tool_results && !use_clear_tool_uses {
        if strategies.is_empty() {
            return None;
        }
        return Some(ContextManagementConfig { edits: strategies });
    }

    if use_clear_tool_results {
        let trigger_threshold = std::env::var("API_MAX_INPUT_TOKENS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MAX_INPUT_TOKENS);
        let keep_target = std::env::var("API_TARGET_INPUT_TOKENS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_TARGET_INPUT_TOKENS);

        strategies.push(ContextEditStrategy::ClearToolUses20250919 {
            trigger: Some(TriggerConfig {
                trigger_type: "input_tokens".to_string(),
                value: trigger_threshold,
            }),
            keep: None,
            clear_tool_inputs: Some(
                tools_clearable_results()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
            exclude_tools: None,
            clear_at_least: Some(TriggerConfig {
                trigger_type: "input_tokens".to_string(),
                value: trigger_threshold.saturating_sub(keep_target),
            }),
        });
    }

    if use_clear_tool_uses {
        let trigger_threshold = std::env::var("API_MAX_INPUT_TOKENS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MAX_INPUT_TOKENS);
        let keep_target = std::env::var("API_TARGET_INPUT_TOKENS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_TARGET_INPUT_TOKENS);

        strategies.push(ContextEditStrategy::ClearToolUses20250919 {
            trigger: Some(TriggerConfig {
                trigger_type: "input_tokens".to_string(),
                value: trigger_threshold,
            }),
            keep: None,
            clear_tool_inputs: None,
            exclude_tools: Some(
                tools_clearable_uses()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
            clear_at_least: Some(TriggerConfig {
                trigger_type: "input_tokens".to_string(),
                value: trigger_threshold.saturating_sub(keep_target),
            }),
        });
    }

    if strategies.is_empty() {
        None
    } else {
        Some(ContextManagementConfig { edits: strategies })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_api_context_management_no_thinking() {
        let result = get_api_context_management(None);
        // Without thinking and without env vars, should return None
        assert!(result.is_none());
    }

    #[test]
    fn test_get_api_context_management_with_thinking() {
        let opts = ContextManagementOptions {
            has_thinking: true,
            ..Default::default()
        };
        let result = get_api_context_management(Some(opts));
        // Should have thinking clear strategy
        assert!(result.is_some());
        let config = result.unwrap();
        assert!(!config.edits.is_empty());
        assert!(matches!(
            &config.edits[0],
            ContextEditStrategy::ClearThinking20251015 { .. }
        ));
    }

    #[test]
    fn test_get_api_context_management_clear_all_thinking() {
        let opts = ContextManagementOptions {
            has_thinking: true,
            clear_all_thinking: true,
            ..Default::default()
        };
        let result = get_api_context_management(Some(opts));
        assert!(result.is_some());
        let config = result.unwrap();
        // Should have thinking clear with value: 1
        match &config.edits[0] {
            ContextEditStrategy::ClearThinking20251015 { keep } => {
                match keep {
                    ThinkingKeepConfig::KeepTurns { value, .. } => {
                        assert_eq!(*value, 1);
                    }
                    _ => panic!("Expected KeepTurns"),
                }
            }
            _ => panic!("Expected ClearThinking20251015"),
        }
    }
}
