// Source: /data/home/swei/claudecode/openclaudecode/src/commands/compact/compact.ts
//! Context compaction module.
//!
//! Handles automatic context compaction when the conversation gets too long.
//! This includes token threshold detection, summary generation, and message management.

use crate::constants::env::{ai, ai_code};
pub use crate::services::token_estimation::{
    rough_token_count_estimation, rough_token_count_estimation_for_message,
};
use crate::types::*;

/// Default context window sizes by model (in tokens)
pub const DEFAULT_CONTEXT_WINDOW: u32 = 200_000;

/// Get default context window from environment or use default
pub fn get_default_context_window() -> u32 {
    if let Ok(override_val) = std::env::var(ai::CONTEXT_WINDOW) {
        if let Ok(parsed) = override_val.parse::<u32>() {
            if parsed > 0 {
                return parsed;
            }
        }
    }
    DEFAULT_CONTEXT_WINDOW
}

/// Get the prompt for generating conversation summary
/// Translated from: getCompactPrompt in prompt.ts
pub fn get_compact_prompt() -> String {
    r#"CRITICAL: Respond with TEXT ONLY. Do NOT call any tools.

- Do NOT use Read, Bash, Grep, Glob, Edit, Write, or ANY other tool.
- You already have all the context you need in the conversation above.
- Tool calls will be REJECTED and will waste your only turn — you will fail the task.
- Your entire response must be plain text: an <analysis> block followed by a <summary> block.

Your task is to create a detailed summary of the conversation so far, paying close attention to the user's explicit requests and your previous actions.
This summary should be thorough in capturing technical details, code patterns, and architectural decisions that would be essential for continuing development work without losing context.

Before providing your final summary, wrap your analysis in <analysis> tags to organize your thoughts and ensure you've covered all necessary points. In your analysis process:

1. Chronologically analyze each message and section of the conversation. For each section thoroughly identify:
   - The user's explicit requests and intents
   - Your approach to addressing the user's requests
   - Key decisions, technical concepts and code patterns
   - Specific details like:
     - file names
     - full code snippets
     - function signatures
     - file edits
   - Errors that you ran into and how you fixed them
   - Pay special attention to specific user feedback that you received, especially if the user told you to do something differently.
2. Double-check for technical accuracy and completeness, addressing each required element thoroughly.

Your summary should include the following sections:

1. Primary Request and Intent: Capture all of the user's explicit requests and intents in detail
2. Key Technical Concepts: List all important technical concepts, technologies, and frameworks discussed.
3. Files and Code Sections: Enumerate specific files and code sections examined, modified, or created. Pay special attention to the most recent messages and include full code snippets where applicable and include a summary of why this file read or edit is important.
4. Errors and fixes: List all errors that you ran into, and how you fixed them. Pay special attention to specific user feedback that you received, especially if the user told you to do something differently.
5. Problem Solving: Document problems solved and any ongoing troubleshooting efforts.
6. All user messages: List ALL user messages that are not tool results. These are critical for understanding the users' feedback and changing intent.
7. Pending Tasks: Outline any pending tasks that you have explicitly been asked to work on.
8. Current Work: Describe in detail precisely what was being worked on immediately before this summary request, paying special attention to the most recent messages from both user and assistant. Include file names and code snippets where applicable.
9. Context for Continuing Work: Key context, decisions, or state needed to continue the work.

IMPORTANT: Be extremely thorough — include ALL important technical details, code patterns, and architectural decisions. This summary must provide enough context for the next turn to continue seamlessly.

REMINDER: Do NOT call any tools. Respond with plain text only — an <analysis> block followed by a <summary> block. Tool calls will be rejected and you will fail the task.
"#.to_string()
}

/// Reserve tokens for output during compaction
/// Based on p99.99 of compact summary output
pub const MAX_OUTPUT_TOKENS_FOR_SUMMARY: u32 = 20_000;

/// Buffer tokens for auto-compact trigger
pub const AUTOCOMPACT_BUFFER_TOKENS: u32 = 13_000;

/// Buffer tokens for warning threshold
pub const WARNING_THRESHOLD_BUFFER_TOKENS: u32 = 20_000;

/// Buffer tokens for error threshold
pub const ERROR_THRESHOLD_BUFFER_TOKENS: u32 = 20_000;

/// Get the blocking limit (when to block further input)
pub fn get_blocking_limit(model: &str) -> u32 {
    let effective_window = get_effective_context_window_size(model);
    let default_blocking_limit = effective_window.saturating_sub(MANUAL_COMPACT_BUFFER_TOKENS);

    // Allow override for testing
    if let Ok(override_val) = std::env::var(ai::BLOCKING_LIMIT_OVERRIDE) {
        if let Ok(parsed) = override_val.parse::<u32>() {
            if parsed > 0 {
                return parsed;
            }
        }
    }

    default_blocking_limit
}

/// Manual compact uses smaller buffer (more aggressive)
pub const MANUAL_COMPACT_BUFFER_TOKENS: u32 = 3_000;

/// Maximum consecutive auto-compact failures before giving up
pub const MAX_CONSECUTIVE_AUTOCOMPACT_FAILURES: u32 = 3;

/// Post-compaction: max files to restore
pub const POST_COMPACT_MAX_FILES_TO_RESTORE: u32 = 5;

/// Post-compaction: token budget for restored files
pub const POST_COMPACT_TOKEN_BUDGET: u32 = 50_000;

/// Post-compaction: max tokens per file
pub const POST_COMPACT_MAX_TOKENS_PER_FILE: u32 = 5_000;

/// Post-compaction: max tokens per skill
pub const POST_COMPACT_MAX_TOKENS_PER_SKILL: u32 = 5_000;

/// Post-compaction: skills token budget
pub const POST_COMPACT_SKILLS_TOKEN_BUDGET: u32 = 25_000;

/// Get effective context window size (total - output reserve)
pub fn get_effective_context_window_size(model: &str) -> u32 {
    let context_window = get_context_window_for_model(model);
    context_window.saturating_sub(MAX_OUTPUT_TOKENS_FOR_SUMMARY)
}

/// Get context window size for a model
pub fn get_context_window_for_model(model: &str) -> u32 {
    // Check environment override for auto compact window
    if let Ok(override_val) = std::env::var(ai::AUTO_COMPACT_WINDOW) {
        if let Ok(parsed) = override_val.parse::<u32>() {
            if parsed > 0 {
                return parsed;
            }
        }
    }

    // Default context windows by model
    let lower = model.to_lowercase();
    if lower.contains("sonnet") {
        // Claude Sonnet models typically have 200K context
        get_default_context_window()
    } else if lower.contains("haiku") {
        // Haiku has 200K context
        get_default_context_window()
    } else if lower.contains("opus") {
        // Opus models typically have 200K context
        get_default_context_window()
    } else {
        get_default_context_window()
    }
}

/// Get the auto-compact threshold (when to trigger compaction)
pub fn get_auto_compact_threshold(model: &str) -> u32 {
    let effective_window = get_effective_context_window_size(model);

    let autocompact_threshold = effective_window.saturating_sub(AUTOCOMPACT_BUFFER_TOKENS);

    // Override for easier testing of autocompact
    if let Ok(env_percent) = std::env::var(ai::AUTOCOMPACT_PCT_OVERRIDE) {
        if let Ok(parsed) = env_percent.parse::<f64>() {
            if parsed > 0.0 && parsed <= 100.0 {
                let percentage_threshold =
                    ((effective_window as f64 * (parsed / 100.0)) as u32).min(effective_window);
                return percentage_threshold.min(autocompact_threshold);
            }
        }
    }

    autocompact_threshold
}

/// Calculate token warning state
/// Translated from: calculateTokenWarningState in autoCompact.ts
#[derive(Debug, Clone)]
pub struct TokenWarningState {
    pub percent_left: f64,
    pub is_above_warning_threshold: bool,
    pub is_above_error_threshold: bool,
    pub is_above_auto_compact_threshold: bool,
    pub is_at_blocking_limit: bool,
}

pub fn calculate_token_warning_state(token_usage: u32, model: &str) -> TokenWarningState {
    let auto_compact_threshold = get_auto_compact_threshold(model);
    let effective_window = get_effective_context_window_size(model);

    // Use auto_compact_threshold if enabled, otherwise use effective window
    let threshold = if is_auto_compact_enabled_for_calculation() {
        auto_compact_threshold
    } else {
        effective_window
    };

    let percent_left = if threshold > 0 {
        ((threshold.saturating_sub(token_usage) as f64 / threshold as f64) * 100.0).max(0.0)
    } else {
        100.0
    };

    let warning_threshold = threshold.saturating_sub(WARNING_THRESHOLD_BUFFER_TOKENS);
    let error_threshold = threshold.saturating_sub(ERROR_THRESHOLD_BUFFER_TOKENS);

    let is_above_warning_threshold = token_usage >= warning_threshold;
    let is_above_error_threshold = token_usage >= error_threshold;
    let is_above_auto_compact_threshold =
        is_auto_compact_enabled_for_calculation() && token_usage >= auto_compact_threshold;

    // Calculate blocking limit
    let default_blocking_limit = effective_window.saturating_sub(MANUAL_COMPACT_BUFFER_TOKENS);

    // Allow override for testing (translate from CLAUDE_CODE_BLOCKING_LIMIT_OVERRIDE)
    let blocking_limit = if let Ok(override_val) = std::env::var(ai_code::BLOCKING_LIMIT_OVERRIDE) {
        if let Ok(parsed) = override_val.parse::<u32>() {
            if parsed > 0 {
                parsed
            } else {
                default_blocking_limit
            }
        } else {
            default_blocking_limit
        }
    } else {
        default_blocking_limit
    };

    let is_at_blocking_limit = token_usage >= blocking_limit;

    TokenWarningState {
        percent_left,
        is_above_warning_threshold,
        is_above_error_threshold,
        is_above_auto_compact_threshold,
        is_at_blocking_limit,
    }
}

/// Check if auto-compact is enabled (used in calculation)
/// Translated from: isAutoCompactEnabled in autoCompact.ts
fn is_auto_compact_enabled_for_calculation() -> bool {
    use crate::utils::env_utils::is_env_truthy;

    if is_env_truthy(Some("DISABLE_COMPACT")) {
        return false;
    }
    if is_env_truthy(Some("DISABLE_AUTO_COMPACT")) {
        return false;
    }
    // Check user config - for now default to true
    // In full implementation: getGlobalConfig().autoCompactEnabled
    true
}

/// Compact result containing the new messages after compaction
#[derive(Debug, Clone)]
pub struct CompactionResult {
    /// The boundary marker message
    pub boundary_marker: Message,
    /// Summary messages to keep
    pub summary_messages: Vec<Message>,
    /// Messages that were kept (not summarized)
    pub messages_to_keep: Option<Vec<Message>>,
    /// Attachments to include
    pub attachments: Vec<Message>,
    /// Pre-compaction token count
    pub pre_compact_token_count: u32,
    /// Post-compaction token count
    pub post_compact_token_count: u32,
}

/// Strip images from messages before sending for compaction
/// Images are replaced with `[image]` text markers, documents with `[document]` markers
/// to prevent compaction API from hitting prompt-too-long
pub fn strip_images_from_messages(messages: &[Message]) -> Vec<Message> {
    use crate::types::MessageRole;

    messages
        .iter()
        .map(|msg| {
            match msg.role {
                MessageRole::User | MessageRole::Assistant => {
                    // For user/assistant messages, strip image/document blocks
                    // In the simple String content model, we look for image-like patterns
                    let content = msg.content.clone();
                    // Check for image markdown patterns
                    if content.contains("![") || content.contains("<img") {
                        // Strip markdown images: ![alt](url)
                        let stripped = strip_image_markdown(&content);
                        if stripped != content {
                            return Message {
                                role: msg.role.clone(),
                                content: stripped,
                                ..msg.clone()
                            };
                        }
                    }
                    msg.clone()
                }
                MessageRole::Tool => {
                    // Tool results might contain image references
                    let content = msg.content.clone();
                    if content.contains("![")
                        || content.contains("<img")
                        || content.contains("image")
                        || content.contains("document")
                    {
                        let stripped = strip_image_markdown(&content);
                        if stripped != content {
                            return Message {
                                role: msg.role.clone(),
                                content: stripped,
                                ..msg.clone()
                            };
                        }
                    }
                    msg.clone()
                }
                MessageRole::System => msg.clone(),
            }
        })
        .collect()
}

/// Strip markdown image patterns from content, replacing with text markers
fn strip_image_markdown(content: &str) -> String {
    // Replace markdown images ![alt](url) with [image]
    let mut result = content.to_string();

    // Simple regex-like replacement for markdown images
    // ![...](...) → [image]
    let mut output = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '!' && i + 1 < chars.len() && chars[i + 1] == '[' {
            // Find the closing ](
            if let Some(close_bracket) = chars[i..].iter().position(|&c| c == ']') {
                let bracket_pos = i + close_bracket;
                if bracket_pos + 1 < chars.len() && chars[bracket_pos + 1] == '(' {
                    // Find the closing )
                    if let Some(close_paren) =
                        chars[bracket_pos + 2..].iter().position(|&c| c == ')')
                    {
                        let paren_pos = bracket_pos + 2 + close_paren;
                        // Extract alt text
                        let alt: String = chars[i + 2..bracket_pos].iter().collect();
                        let marker = if alt.to_lowercase().contains("doc")
                            || alt.to_lowercase().contains("pdf")
                            || alt.to_lowercase().contains("file")
                        {
                            "[document]"
                        } else {
                            "[image]"
                        };
                        output.push_str(marker);
                        i = paren_pos + 1;
                        continue;
                    }
                }
            }
        }
        output.push(chars[i]);
        i += 1;
    }

    output
}

/// Strip reinjected attachments (skill_discovery/skill_listing) that will be
/// re-injected post-compaction anyway
pub fn strip_reinjected_attachments(messages: &[Message]) -> Vec<Message> {
    // In the simple String content model, we look for skill attachment patterns
    messages
        .iter()
        .map(|msg| {
            if msg.content.contains("skill_discovery") || msg.content.contains("skill_listing") {
                Message {
                    role: msg.role.clone(),
                    content: "[Skill attachment content cleared for compaction]".to_string(),
                    ..msg.clone()
                }
            } else {
                msg.clone()
            }
        })
        .collect()
}

/// Estimate token count for messages (rough estimation)
/// Uses 4 chars per token for regular text (matching original TypeScript)
/// Uses 2 chars per token for tool results (JSON is more token-efficient)
/// Takes optional max_output_tokens to ensure we leave room for the response
pub fn estimate_token_count(messages: &[Message], max_output_tokens: u32) -> u32 {
    // Regular text: 4 chars per token (original TypeScript default)
    let non_tool_chars: usize = messages
        .iter()
        .filter(|msg| msg.role != MessageRole::Tool)
        .map(|msg| msg.content.len())
        .sum();

    // Tool results (JSON): 2 chars per token (more efficient encoding)
    // Original: "Dense JSON has many single-character tokens..."
    let tool_result_chars: usize = messages
        .iter()
        .filter(|msg| msg.role == MessageRole::Tool)
        .map(|msg| msg.content.len())
        .sum();

    let base_estimate = (non_tool_chars / 4) as u32;
    let tool_buffer = (tool_result_chars / 2) as u32; // More efficient for JSON

    // Add the requested output tokens to ensure we leave room for the response
    base_estimate + tool_buffer + max_output_tokens
}

/// Check if conversation should be compacted
pub fn should_compact(token_usage: u32, model: &str) -> bool {
    let state = calculate_token_warning_state(token_usage, model);
    state.is_above_auto_compact_threshold
}

/// Truncate messages to fit within a safe token limit for summarization
/// This is used when the conversation is too large to fit in context
/// Skips ALL system messages (they contain huge compaction summaries)
/// Returns (truncated_messages, estimated_tokens)
pub fn truncate_messages_for_summary(
    messages: &[Message],
    model: &str,
    max_output_tokens: u32,
) -> (Vec<Message>, u32) {
    let context_window = get_context_window_for_model(model);
    // Leave room for output tokens and buffer - use 50% of available space for safety
    let safe_limit = ((context_window.saturating_sub(max_output_tokens)) as f64 * 0.50) as u32;

    let total_messages = messages.len();
    if total_messages == 0 {
        return (vec![], 0);
    }

    // Skip ALL system messages - they contain huge compaction summaries from previous rounds
    // For summarization, we only need the conversation history (user/assistant/tool messages)
    let non_system_messages: Vec<Message> = messages
        .iter()
        .filter(|m| m.role != MessageRole::System)
        .cloned()
        .collect();

    // Now take most recent non-system messages using proper token estimation
    let mut current_tokens = 0u32;
    let mut history_messages = Vec::new();

    for msg in non_system_messages.iter().rev() {
        let msg_tokens = rough_token_count_estimation_for_message(msg) as u32;
        if current_tokens + msg_tokens > safe_limit {
            break;
        }
        current_tokens += msg_tokens;
        history_messages.insert(0, msg.clone());
    }

    // If we couldn't fit any history, try to at least get recent messages
    if history_messages.is_empty() && !non_system_messages.is_empty() {
        // Take just the last message, truncated if needed
        let last_msg = non_system_messages.last().unwrap();
        let max_chars = (safe_limit as usize) * 4;
        let chars_to_keep = last_msg.content.len().min(max_chars);
        let truncated_content = last_msg
            .content
            .chars()
            .take(chars_to_keep)
            .collect::<String>();

        current_tokens = rough_token_count_estimation(&truncated_content, 4.0) as u32;

        history_messages = vec![Message {
            role: last_msg.role.clone(),
            content: truncated_content,
            ..Default::default()
        }];
    }

    let total_estimated = current_tokens;

    (history_messages, total_estimated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_context_window() {
        let window = get_effective_context_window_size("claude-sonnet-4-6");
        // 200000 - 20000 = 180000
        assert_eq!(window, 180_000);
    }

    #[test]
    fn test_auto_compact_threshold() {
        let threshold = get_auto_compact_threshold("claude-sonnet-4-6");
        // 180000 - 13000 = 167000
        assert_eq!(threshold, 167_000);
    }

    #[test]
    fn test_token_warning_state_normal() {
        let state = calculate_token_warning_state(50_000, "claude-sonnet-4-6");
        assert!(!state.is_above_warning_threshold);
        assert!(!state.is_above_error_threshold);
        assert!(!state.is_above_auto_compact_threshold);
        assert!(state.percent_left > 50.0);
    }

    #[test]
    fn test_token_warning_state_warning() {
        // warning at 180000 - 20000 = 160000
        let state = calculate_token_warning_state(165_000, "claude-sonnet-4-6");
        assert!(state.is_above_warning_threshold);
        // error uses same buffer, so this is also above error threshold
        assert!(state.is_above_error_threshold);
        assert!(!state.is_above_auto_compact_threshold);
    }

    #[test]
    fn test_token_warning_state_compact() {
        let state = calculate_token_warning_state(170_000, "claude-sonnet-4-6");
        assert!(state.is_above_warning_threshold);
        assert!(state.is_above_auto_compact_threshold);
    }

    #[test]
    fn test_should_compact() {
        assert!(!should_compact(50_000, "claude-sonnet-4-6"));
        assert!(should_compact(170_000, "claude-sonnet-4-6"));
    }

    #[test]
    fn test_estimate_token_count() {
        let messages = vec![
            Message {
                role: MessageRole::User,
                content: "Hello, this is a test message".to_string(),
                ..Default::default()
            },
            Message {
                role: MessageRole::Assistant,
                content: "Hi! How can I help you today?".to_string(),
                ..Default::default()
            },
        ];

        let count = estimate_token_count(&messages, 0);
        // ~60 chars / 4 = 15 tokens
        assert!(count > 0);
    }
}

// ============================================================================
// Compact Command Module (translated from commands/compact/)
// ============================================================================

/// Compact command definition
/// Translates: /data/home/swei/claudecode/openclaudecode/src/commands/compact/index.ts

/// Check if an environment variable is truthy (copied from bridge_enabled)
fn is_env_truthy(env_var: &str) -> bool {
    if env_var.is_empty() {
        return false;
    }
    let binding = env_var.to_lowercase();
    let normalized = binding.trim();
    matches!(normalized, "1" | "true" | "yes" | "on")
}

/// Compact command configuration
#[derive(Debug, Clone)]
pub struct CompactCommand {
    /// Command type
    pub command_type: String,
    /// Command name
    pub name: String,
    /// Command description
    pub description: String,
    /// Whether the command is enabled
    pub is_enabled: fn() -> bool,
    /// Whether it supports non-interactive mode
    pub supports_non_interactive: bool,
    /// Argument hint text
    pub argument_hint: String,
}

impl Default for CompactCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl CompactCommand {
    /// Create a new compact command
    pub fn new() -> Self {
        Self {
            command_type: "local".to_string(),
            name: "compact".to_string(),
            description: "Clear conversation history but keep a summary in context. Optional: /compact [instructions for summarization]".to_string(),
            is_enabled: || !is_env_truthy("AI_DISABLE_COMPACT"),
            supports_non_interactive: true,
            argument_hint: "<optional custom summarization instructions>".to_string(),
        }
    }

    /// Check if the command is enabled
    pub fn is_enabled(&self) -> bool {
        (self.is_enabled)()
    }
}

/// Get the compact command
pub fn get_compact_command() -> CompactCommand {
    CompactCommand::new()
}

/// Compact command error messages
pub mod compact_errors {
    /// Error message for incomplete response
    pub const ERROR_MESSAGE_INCOMPLETE_RESPONSE: &str =
        "Incomplete response from model during compaction";
    /// Error message for not enough messages
    pub const ERROR_MESSAGE_NOT_ENOUGH_MESSAGES: &str = "Not enough messages to compact";
    /// Error message for user abort
    pub const ERROR_MESSAGE_USER_ABORT: &str = "User aborted compaction";
}
