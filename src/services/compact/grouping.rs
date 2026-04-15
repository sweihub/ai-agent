// Source: ~/claudecode/openclaudecode/src/services/compact/grouping.ts
use crate::types::api_types::Message;
use crate::types::MessageRole;

/// Groups messages at API-round boundaries: one group per API round-trip.
/// A boundary fires when a NEW assistant response begins (different
/// message_id from the prior assistant). For well-formed conversations
/// this is an API-safe split point -- the API contract requires every
/// tool_use to be resolved before the next assistant turn, so pairing
/// validity falls out of the assistant-id boundary. For malformed inputs
/// (dangling tool_use after resume/truncation) the fork's
/// ensureToolResultPairing repairs the split at API time.
///
/// Replaces the prior human-turn grouping (boundaries only at real user
/// prompts) with finer-grained API-round grouping, allowing reactive
/// compact to operate on single-prompt agentic sessions (SDK/CCR/eval
/// callers) where the entire workload is one human turn.
///
/// Extracted to its own file to break the compact.ts <-> compactMessages.ts
/// cycle (CC-1180) -- the cycle shifted module-init order enough to surface
/// a latent ws CJS/ESM resolution race in CI shard-2.
pub fn group_messages_by_api_round(messages: &[Message]) -> Vec<Vec<Message>> {
    let mut groups: Vec<Vec<Message>> = Vec::new();
    let mut current: Vec<Message> = Vec::new();
    // Track when we last saw an assistant message to detect round boundaries.
    // In the simple String-based Message model, we use content as a proxy
    // for message identity. A boundary fires when we see a NEW assistant response
    // (different content from the prior assistant message) AFTER we've already
    // seen at least one assistant message.
    let mut last_assistant_content: Option<String> = None;
    let mut has_seen_assistant = false;

    for msg in messages {
        if matches!(msg.role, MessageRole::Assistant) {
            // Check if this is a genuinely new assistant response
            // (only create boundary if we've already seen an assistant message)
            let is_new = has_seen_assistant && last_assistant_content.as_ref() != Some(&msg.content);
            if is_new && !current.is_empty() {
                groups.push(current);
                current = vec![msg.clone()];
            } else {
                current.push(msg.clone());
            }
            last_assistant_content = Some(msg.content.clone());
            has_seen_assistant = true;
        } else {
            current.push(msg.clone());
        }
    }

    if !current.is_empty() {
        groups.push(current);
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_user_message(content: &str) -> Message {
        Message {
            role: MessageRole::User,
            content: content.to_string(),
            ..Default::default()
        }
    }

    fn make_assistant_message(content: &str, _id: &str) -> Message {
        Message {
            role: MessageRole::Assistant,
            content: content.to_string(),
            ..Default::default()
        }
    }

    fn make_tool_message(content: &str, tool_call_id: &str) -> Message {
        Message {
            role: MessageRole::Tool,
            content: content.to_string(),
            tool_call_id: Some(tool_call_id.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_empty_messages() {
        let result = group_messages_by_api_round(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_round() {
        let messages = vec![
            make_user_message("Hello"),
            make_assistant_message("Hi there", "msg-1"),
        ];
        let groups = group_messages_by_api_round(&messages);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 2);
    }

    #[test]
    fn test_multiple_rounds() {
        let messages = vec![
            make_user_message("Hello"),
            make_assistant_message("Hi there", "msg-1"),
            make_user_message("How are you?"),
            make_assistant_message("I'm good", "msg-2"),
        ];
        let groups = group_messages_by_api_round(&messages);
        assert_eq!(groups.len(), 2);
        // First group: [User1, Asst1, User2] - user message before new assistant stays in first group
        // Second group: [Asst2] - new assistant starts new group
        assert_eq!(groups[0].len(), 3);
        assert_eq!(groups[1].len(), 1);
    }

    #[test]
    fn test_streaming_interleaved_tool_results() {
        // Streaming interleaved tool_results should stay in same group
        // because they share the same assistant content (or empty)
        let messages = vec![
            make_assistant_message("Calling tool", "msg-1"),
            make_tool_message("Tool result 1", "call-1"),
            make_tool_message("Tool result 2", "call-2"),
            make_assistant_message("Calling tool", "msg-1"), // Same content = same round
        ];
        let groups = group_messages_by_api_round(&messages);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 4);
    }

    #[test]
    fn test_different_assistant_contents_create_boundary() {
        let messages = vec![
            make_assistant_message("First response", "msg-1"),
            make_assistant_message("Second response", "msg-2"),
        ];
        let groups = group_messages_by_api_round(&messages);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].len(), 1);
        assert_eq!(groups[1].len(), 1);
    }
}
