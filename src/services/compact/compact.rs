// Source: ~/claudecode/openclaudecode/src/services/compact/compact.ts
//! Main compact service - handles conversation compaction
//!
//! Orchestrates the full conversation summarization flow:
//! 1. Analyzes token counts across messages
//! 2. Selects which messages to remove based on direction and token budget
//! 3. Creates boundary/summary messages to replace removed content
//! 4. Returns the compacted message set and summary

use crate::types::api_types::{Message, MessageRole};
use serde::{Deserialize, Serialize};

/// Compact direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum CompactDirection {
    /// Compact from the beginning (oldest messages)
    Head,
    /// Compact from the end (newest messages)
    Tail,
    /// Smart compaction based on token budget
    #[default]
    Smart,
}

/// Compact result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactResult {
    /// Whether compaction was successful
    pub success: bool,
    /// Number of messages removed
    pub messages_removed: usize,
    /// Token count before compaction
    pub tokens_before: u64,
    /// Token count after compaction
    pub tokens_after: u64,
    /// Direction used
    pub direction: CompactDirection,
    /// The generated summary of compacted conversation
    pub summary: String,
    /// Messages to keep (after compaction)
    pub messages_to_keep: Vec<Message>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Compact options
#[derive(Debug, Clone, Default)]
pub struct CompactOptions {
    /// Maximum tokens to keep after compaction
    pub max_tokens: Option<u64>,
    /// Direction to compact
    pub direction: CompactDirection,
    /// Whether to create a boundary message
    pub create_boundary: bool,
    /// Custom system prompt to include
    pub system_prompt: Option<String>,
}

/// A group of messages that can be compacted together
#[derive(Debug, Clone)]
struct MessageGroup {
    /// Index of the first message in this group
    start_index: usize,
    /// Messages in this group
    messages: Vec<Message>,
    /// Estimated token count for this group
    token_count: u64,
    /// Whether this group is at the boundary (recent messages)
    is_boundary: bool,
}

/// Group messages into logical units for compaction.
/// Groups are: (user message + assistant response + tool results) turns.
fn group_messages(messages: &[Message]) -> Vec<MessageGroup> {
    let mut groups = Vec::new();
    let mut current_group = MessageGroup {
        start_index: 0,
        messages: Vec::new(),
        token_count: 0,
        is_boundary: false,
    };

    for (i, msg) in messages.iter().enumerate() {
        match &msg.role {
            MessageRole::User => {
                // Start a new group for each user message
                if !current_group.messages.is_empty() {
                    groups.push(std::mem::replace(
                        &mut current_group,
                        MessageGroup {
                            start_index: i,
                            messages: Vec::new(),
                            token_count: 0,
                            is_boundary: false,
                        },
                    ));
                }
                current_group.messages.push(msg.clone());
                current_group.token_count += estimate_tokens_for_message(msg);
            }
            MessageRole::Assistant | MessageRole::Tool | MessageRole::System => {
                current_group.messages.push(msg.clone());
                current_group.token_count += estimate_tokens_for_message(msg);
            }
        }
    }

    // Push the last group
    if !current_group.messages.is_empty() {
        groups.push(current_group);
    }

    // Mark the last group as boundary (keep these messages)
    if let Some(last) = groups.last_mut() {
        last.is_boundary = true;
    }

    groups
}

/// Estimate token count for a single message.
fn estimate_tokens_for_message(msg: &Message) -> u64 {
    // Rough estimation: ~4 chars per token for English text
    let content_tokens = (msg.content.len() as u64 + 3) / 4;

    // Tool calls add extra tokens
    let tool_call_tokens = msg
        .tool_calls
        .as_ref()
        .map(|calls| {
            calls
                .iter()
                .map(|tc| {
                    let name_tokens = (tc.name.len() as u64 + 3) / 4;
                    let args_tokens = (tc.arguments.to_string().len() as u64 + 3) / 4;
                    name_tokens + args_tokens + 2 // overhead for structure
                })
                .sum::<u64>()
        })
        .unwrap_or(0);

    // Role token overhead
    let role_overhead: u64 = 4;

    content_tokens + tool_call_tokens + role_overhead
}

/// Execute conversation compaction using smart message selection.
///
/// This function analyzes the messages, determines which ones can be compacted,
/// and returns a compact result with the messages to keep and a summary.
///
/// In production, the summary would be generated by an LLM. Here we provide
/// the structural logic for message selection and grouping.
pub async fn compact_messages(
    messages: &[Message],
    options: CompactOptions,
) -> Result<CompactResult, String> {
    if messages.is_empty() {
        return Ok(CompactResult {
            success: true,
            messages_removed: 0,
            tokens_before: 0,
            tokens_after: 0,
            direction: options.direction,
            summary: String::new(),
            messages_to_keep: Vec::new(),
            error: None,
        });
    }

    // Calculate total tokens before compaction
    let tokens_before: u64 = messages.iter().map(estimate_tokens_for_message).sum();

    let target_tokens = options.max_tokens.unwrap_or(tokens_before);

    // If already within budget, no compaction needed
    if tokens_before <= target_tokens {
        return Ok(CompactResult {
            success: true,
            messages_removed: 0,
            tokens_before,
            tokens_after: tokens_before,
            direction: options.direction,
            summary: String::new(),
            messages_to_keep: messages.to_vec(),
            error: None,
        });
    }

    // Group messages into logical turns
    let groups = group_messages(messages);

    // Determine direction
    let direction = if options.direction == CompactDirection::Smart {
        get_recommended_direction(messages.len(), tokens_before, target_tokens)
    } else {
        options.direction
    };

    // Select which groups to compact
    let (kept_groups, compacted_groups) =
        select_groups_to_compact(&groups, target_tokens, direction);

    // Build the result
    let messages_to_keep: Vec<Message> = kept_groups
        .iter()
        .flat_map(|g| g.messages.clone())
        .collect();

    let messages_removed: usize = compacted_groups.iter().map(|g| g.messages.len()).sum();

    // Create a summary of compacted content
    let summary = create_compact_summary(&compacted_groups);

    // Calculate tokens after compaction
    let tokens_after: u64 = messages_to_keep
        .iter()
        .map(estimate_tokens_for_message)
        .sum();

    log::info!(
        "[compact] Compacted {} messages: {} -> {} tokens (direction: {:?})",
        messages_removed,
        tokens_before,
        tokens_after,
        direction
    );

    Ok(CompactResult {
        success: true,
        messages_removed,
        tokens_before,
        tokens_after,
        direction,
        summary,
        messages_to_keep,
        error: None,
    })
}

/// Select which message groups to compact based on direction and token budget.
fn select_groups_to_compact(
    groups: &[MessageGroup],
    target_tokens: u64,
    direction: CompactDirection,
) -> (Vec<&MessageGroup>, Vec<&MessageGroup>) {
    // Always keep the boundary group (most recent conversation)
    let (boundary, non_boundary): (Vec<_>, Vec<_>) = groups.iter().partition(|g| g.is_boundary);

    // Calculate remaining budget after keeping boundary
    let boundary_tokens: u64 = boundary.iter().map(|g| g.token_count).sum();
    let mut remaining_budget = target_tokens.saturating_sub(boundary_tokens);

    let mut kept = boundary;
    let mut compacted = Vec::new();

    match direction {
        CompactDirection::Head => {
            // Compact from the beginning (oldest messages first)
            let mut non_boundary_iter = non_boundary.into_iter().peekable();
            while let Some(group) = non_boundary_iter.next() {
                if remaining_budget >= group.token_count {
                    kept.push(group);
                    remaining_budget -= group.token_count;
                } else {
                    compacted.push(group);
                    // Also compact all remaining groups
                    compacted.extend(non_boundary_iter);
                    break;
                }
            }
        }
        CompactDirection::Tail => {
            // Compact from the end (oldest messages kept, newest non-boundary compacted)
            let mut non_boundary_iter = non_boundary.into_iter().rev().peekable();
            while let Some(group) = non_boundary_iter.next() {
                if remaining_budget >= group.token_count {
                    kept.push(group);
                    remaining_budget -= group.token_count;
                } else {
                    compacted.push(group);
                    // Also compact all remaining groups
                    compacted.extend(non_boundary_iter);
                    break;
                }
            }
        }
        CompactDirection::Smart => {
            // Smart: prefer compacting from head to preserve recent context
            // Same as Head direction
            let mut non_boundary_iter = non_boundary.into_iter().peekable();
            while let Some(group) = non_boundary_iter.next() {
                if remaining_budget >= group.token_count {
                    kept.push(group);
                    remaining_budget -= group.token_count;
                } else {
                    compacted.push(group);
                    compacted.extend(non_boundary_iter);
                    break;
                }
            }
        }
    }

    // Sort kept by original order for consistency
    kept.sort_by_key(|g| g.start_index);

    (kept, compacted)
}

/// Create a summary string from compacted message groups.
fn create_compact_summary(compacted_groups: &[&MessageGroup]) -> String {
    if compacted_groups.is_empty() {
        return String::new();
    }

    let mut summary = String::new();
    let total_compacted: usize = compacted_groups.iter().map(|g| g.messages.len()).sum();
    let total_tokens: u64 = compacted_groups.iter().map(|g| g.token_count).sum();

    summary.push_str(&format!(
        "Compacted {} messages (~{} tokens) from the conversation history.\n\n",
        total_compacted, total_tokens
    ));

    // Summarize the types of content that was compacted
    let mut user_messages = 0;
    let mut assistant_messages = 0;
    let mut tool_messages = 0;

    for group in compacted_groups {
        for msg in &group.messages {
            match &msg.role {
                MessageRole::User => user_messages += 1,
                MessageRole::Assistant => assistant_messages += 1,
                MessageRole::Tool => tool_messages += 1,
                MessageRole::System => {}
            }
        }
    }

    if user_messages > 0 || assistant_messages > 0 {
        summary.push_str(&format!(
            "The compacted section contained {} user messages and {} assistant responses",
            user_messages, assistant_messages
        ));
        if tool_messages > 0 {
            summary.push_str(&format!(" with {} tool results", tool_messages));
        }
        summary.push_str(".\n");
    }

    summary
}

/// Get the recommended compact direction based on message count and tokens
pub fn get_recommended_direction(
    message_count: usize,
    total_tokens: u64,
    max_tokens: u64,
) -> CompactDirection {
    if total_tokens <= max_tokens {
        return CompactDirection::Smart;
    }

    // If more than half the messages are from the user, compact from head
    // to preserve recent assistant responses
    if message_count > 10 {
        CompactDirection::Head
    } else {
        CompactDirection::Smart
    }
}

/// Calculate the number of messages to remove for compact
pub fn calculate_messages_to_remove(
    current_tokens: u64,
    target_tokens: u64,
    avg_tokens_per_message: u64,
) -> usize {
    if current_tokens <= target_tokens {
        return 0;
    }

    let tokens_to_remove = current_tokens - target_tokens;
    (tokens_to_remove / avg_tokens_per_message) as usize
}

/// Estimate token count for a string (rough approximation).
pub fn rough_token_estimation(text: &str) -> u64 {
    // ~4 chars per token for English text
    (text.len() as u64 + 3) / 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_direction_default() {
        let options = CompactOptions::default();
        assert_eq!(options.direction, CompactDirection::Smart);
    }

    #[test]
    fn test_get_recommended_direction_no_compact() {
        let dir = get_recommended_direction(5, 1000, 2000);
        assert_eq!(dir, CompactDirection::Smart);
    }

    #[test]
    fn test_calculate_messages_to_remove() {
        let count = calculate_messages_to_remove(5000, 2000, 500);
        assert_eq!(count, 6);
    }

    #[test]
    fn test_calculate_messages_to_remove_no_need() {
        let count = calculate_messages_to_remove(1000, 2000, 500);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_rough_token_estimation() {
        let text = "Hello, this is a test message with some content.";
        let tokens = rough_token_estimation(text);
        assert!(tokens > 0);
        // Should be roughly text.len() / 4
        assert!(tokens <= (text.len() as u64 + 3) / 4 + 1);
    }

    #[test]
    fn test_estimate_tokens_for_message() {
        let msg = Message {
            role: MessageRole::User,
            content: "Hello, how are you?".to_string(),
            ..Default::default()
        };
        let tokens = estimate_tokens_for_message(&msg);
        assert!(tokens > 0);
    }

    #[test]
    fn test_group_messages_basic() {
        let messages = vec![
            Message {
                role: MessageRole::User,
                content: "Question 1".to_string(),
                ..Default::default()
            },
            Message {
                role: MessageRole::Assistant,
                content: "Answer 1".to_string(),
                ..Default::default()
            },
            Message {
                role: MessageRole::User,
                content: "Question 2".to_string(),
                ..Default::default()
            },
            Message {
                role: MessageRole::Assistant,
                content: "Answer 2".to_string(),
                ..Default::default()
            },
        ];

        let groups = group_messages(&messages);
        // Should have 2 groups (one per user turn), last one is boundary
        assert_eq!(groups.len(), 2);
        assert!(!groups[0].is_boundary);
        assert!(groups[1].is_boundary);
    }

    #[tokio::test]
    async fn test_compact_messages_empty() {
        let result = compact_messages(&[], CompactOptions::default())
            .await
            .unwrap();
        assert!(result.success);
        assert_eq!(result.messages_removed, 0);
    }

    #[tokio::test]
    async fn test_compact_messages_within_budget() {
        let messages = vec![Message {
            role: MessageRole::User,
            content: "Short message".to_string(),
            ..Default::default()
        }];
        let options = CompactOptions {
            max_tokens: Some(1000000),
            ..Default::default()
        };
        let result = compact_messages(&messages, options).await.unwrap();
        assert!(result.success);
        assert_eq!(result.messages_removed, 0);
    }

    #[tokio::test]
    async fn test_create_compact_summary() {
        let msg1 = Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
            ..Default::default()
        };
        let msg2 = Message {
            role: MessageRole::Assistant,
            content: "Hi there".to_string(),
            ..Default::default()
        };
        let g1 = MessageGroup {
            start_index: 0,
            messages: vec![msg1],
            token_count: 10,
            is_boundary: false,
        };
        let g2 = MessageGroup {
            start_index: 1,
            messages: vec![msg2],
            token_count: 10,
            is_boundary: false,
        };
        let groups = vec![&g1, &g2];

        let summary = create_compact_summary(&groups);
        assert!(summary.contains("2 messages"));
    }
}
