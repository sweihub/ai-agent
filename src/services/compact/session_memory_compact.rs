// Source: ~/claudecode/openclaudecode/src/services/compact/sessionMemoryCompact.ts
//! Session memory compaction.
//!
//! Uses pre-extracted session memory as the summary instead of making an API call.
//! Keeps recent messages above minimum thresholds and preserves API invariants.

use crate::compact::estimate_token_count;
use crate::services::compact::microcompact::estimate_message_tokens;
use crate::tools::config_tools::TOOL_SEARCH_TOOL_NAME;
use crate::types::{Message, MessageRole};
use crate::utils::env_utils;
use std::sync::atomic::{AtomicBool, Ordering};

/// Configuration for session memory compaction thresholds
#[derive(Debug, Clone)]
pub struct SessionMemoryCompactConfig {
    /// Minimum tokens to preserve after compaction
    pub min_tokens: usize,
    /// Minimum number of messages with text blocks to keep
    pub min_text_block_messages: usize,
    /// Maximum tokens to preserve after compaction (hard cap)
    pub max_tokens: usize,
}

impl Default for SessionMemoryCompactConfig {
    fn default() -> Self {
        Self {
            min_tokens: 10_000,
            min_text_block_messages: 5,
            max_tokens: 40_000,
        }
    }
}

// Current configuration
static SM_COMPACT_CONFIG: std::sync::LazyLock<std::sync::Mutex<SessionMemoryCompactConfig>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(SessionMemoryCompactConfig::default()));
static CONFIG_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Get the current session memory compact configuration
pub fn get_session_memory_compact_config() -> SessionMemoryCompactConfig {
    SM_COMPACT_CONFIG.lock().unwrap().clone()
}

/// Check if session memory compaction should be used
pub fn should_use_session_memory_compaction() -> bool {
    // Allow env override for testing
    if env_utils::is_env_truthy(std::env::var("ENABLE_CLAUDE_CODE_SM_COMPACT").ok().as_deref()) {
        return true;
    }
    if env_utils::is_env_truthy(std::env::var("DISABLE_CLAUDE_CODE_SM_COMPACT").ok().as_deref()) {
        return false;
    }

    // For now, default to false (feature-gated in TypeScript)
    false
}

/// Check if a message contains text blocks
pub fn has_text_blocks(message: &Message) -> bool {
    match &message.role {
        MessageRole::Assistant => !message.content.is_empty(),
        MessageRole::User => !message.content.is_empty(),
        _ => false,
    }
}

/// Check if a message is a compact boundary message
pub fn is_compact_boundary_message(message: &Message) -> bool {
    matches!(message.role, MessageRole::System)
        && (message.content.contains("[Previous conversation summarized]")
            || message.content.contains("compacted")
            || message.content.contains("summarized"))
}

/// Collect tool_result IDs from a user message
fn get_tool_result_ids(message: &Message) -> Vec<String> {
    if !matches!(message.role, MessageRole::Tool) {
        return Vec::new();
    }
    message.tool_call_id.clone().into_iter().collect()
}

/// Check if an assistant message contains tool_use blocks with any of the given ids
fn has_tool_use_with_ids(message: &Message, tool_use_ids: &std::collections::HashSet<String>) -> bool {
    if !matches!(message.role, MessageRole::Assistant) {
        return false;
    }
    if let Some(tool_calls) = &message.tool_calls {
        for tc in tool_calls {
            if tool_use_ids.contains(&tc.id) {
                return true;
            }
        }
    }
    false
}

/// Adjust the start index to ensure we don't split tool_use/tool_result pairs
/// or thinking blocks that share the same message.id with kept assistant messages.
pub fn adjust_index_to_preserve_api_invariants(
    messages: &[Message],
    start_index: usize,
) -> usize {
    if start_index <= 0 || start_index >= messages.len() {
        return start_index;
    }

    let mut adjusted_index = start_index;

    // Step 1: Handle tool_use/tool_result pairs
    // Collect tool_result IDs from ALL messages in the kept range
    let all_tool_result_ids: std::collections::HashSet<String> = messages[start_index..]
        .iter()
        .flat_map(get_tool_result_ids)
        .collect();

    if !all_tool_result_ids.is_empty() {
        // Collect tool_use IDs already in the kept range
        let tool_use_ids_in_kept_range: std::collections::HashSet<String> = messages[start_index..]
            .iter()
            .filter(|m| matches!(m.role, MessageRole::Assistant))
            .flat_map(|m| m.tool_calls.iter().flatten().map(|tc| tc.id.clone()))
            .collect();

        // Only look for tool_uses that are NOT already in the kept range
        let needed_tool_use_ids: std::collections::HashSet<String> = all_tool_result_ids
            .difference(&tool_use_ids_in_kept_range)
            .cloned()
            .collect();

        // Find the assistant message(s) with matching tool_use blocks
        for i in (0..adjusted_index).rev() {
            if has_tool_use_with_ids(&messages[i], &needed_tool_use_ids) {
                adjusted_index = i;
                // Remove found tool_use_ids from the set
                if let Some(tool_calls) = &messages[i].tool_calls {
                    for tc in tool_calls {
                        if needed_tool_use_ids.contains(&tc.id) {
                            // Can't remove from HashSet in this loop, just continue
                        }
                    }
                }
            }
        }
    }

    // Step 2: Handle thinking blocks that share message.id with kept assistant messages
    // Note: api_types::Message doesn't have message_id field, so skip this logic
    // In the original TypeScript, this handled thinking blocks that share IDs with assistant messages

    adjusted_index
}

/// Calculate the starting index for messages to keep after compaction.
pub fn calculate_messages_to_keep_index(
    messages: &[Message],
    last_summarized_index: usize,
) -> usize {
    if messages.is_empty() {
        return 0;
    }

    let config = get_session_memory_compact_config();

    // Start from the message after last_summarized_index
    let mut start_index = if last_summarized_index < messages.len() {
        last_summarized_index + 1
    } else {
        messages.len()
    };

    // Calculate current tokens and text-block message count from start_index to end
    let mut total_tokens = 0;
    let mut text_block_message_count = 0;

    for i in start_index..messages.len() {
        total_tokens += estimate_message_tokens(&[messages[i].clone()]);
        if has_text_blocks(&messages[i]) {
            text_block_message_count += 1;
        }
    }

    // Check if we already hit the max cap
    if total_tokens >= config.max_tokens {
        return adjust_index_to_preserve_api_invariants(messages, start_index);
    }

    // Check if we already meet both minimums
    if total_tokens >= config.min_tokens
        && text_block_message_count >= config.min_text_block_messages
    {
        return adjust_index_to_preserve_api_invariants(messages, start_index);
    }

    // Expand backwards until we meet both minimums or hit max cap
    // Floor at the last compact boundary
    let floor = messages
        .iter()
        .rposition(|m| is_compact_boundary_message(m))
        .map(|idx| idx + 1)
        .unwrap_or(0);

    let mut i = if start_index > 0 { start_index - 1 } else { 0 };
    loop {
        if i < floor {
            break;
        }
        let msg = &messages[i];
        let msg_tokens = estimate_message_tokens(&[msg.clone()]);
        total_tokens += msg_tokens;
        if has_text_blocks(msg) {
            text_block_message_count += 1;
        }
        start_index = i;

        // Stop if we hit the max cap
        if total_tokens >= config.max_tokens {
            break;
        }

        // Stop if we meet both minimums
        if total_tokens >= config.min_tokens
            && text_block_message_count >= config.min_text_block_messages
        {
            break;
        }

        if i == 0 {
            break;
        }
        i -= 1;
    }

    adjust_index_to_preserve_api_invariants(messages, start_index)
}

/// Try to use session memory for compaction instead of traditional compaction.
/// Returns None if session memory compaction cannot be used.
pub async fn try_session_memory_compaction(
    messages: &[Message],
    _agent_id: Option<&str>,
    auto_compact_threshold: Option<usize>,
) -> Option<SessionMemoryCompactResult> {
    if !should_use_session_memory_compaction() {
        return None;
    }

    // In a full implementation, this would:
    // 1. Wait for any in-progress session memory extraction
    // 2. Get session memory content
    // 3. Check if session memory is non-empty
    // 4. Calculate messages to keep
    // 5. Create compaction result

    // For now, return None (feature not yet implemented)
    None
}

/// Result from session memory compaction
#[derive(Debug, Clone)]
pub struct SessionMemoryCompactResult {
    pub compacted: bool,
    pub messages_to_keep: Vec<Message>,
    pub session_memory_content: String,
    pub pre_compact_token_count: usize,
    pub post_compact_token_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = get_session_memory_compact_config();
        assert_eq!(config.min_tokens, 10_000);
        assert_eq!(config.min_text_block_messages, 5);
        assert_eq!(config.max_tokens, 40_000);
    }

    #[test]
    fn test_has_text_blocks() {
        let msg = Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
            ..Default::default()
        };
        assert!(has_text_blocks(&msg));

        let empty = Message {
            role: MessageRole::User,
            content: String::new(),
            ..Default::default()
        };
        assert!(!has_text_blocks(&empty));
    }

    #[test]
    fn test_adjust_index_empty_messages() {
        assert_eq!(adjust_index_to_preserve_api_invariants(&[], 0), 0);
    }

    #[test]
    fn test_calculate_messages_to_keep_empty() {
        assert_eq!(calculate_messages_to_keep_index(&[], 0), 0);
    }

    #[test]
    fn test_is_compact_boundary_message() {
        let boundary = Message {
            role: MessageRole::System,
            content: "[Previous conversation summarized]".to_string(),
            ..Default::default()
        };
        assert!(is_compact_boundary_message(&boundary));

        let normal = Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
            ..Default::default()
        };
        assert!(!is_compact_boundary_message(&normal));
    }
}
