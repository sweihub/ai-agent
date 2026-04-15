// Source: ~/claudecode/openclaudecode/src/utils/collapseHookSummaries.ts
//! Collapses consecutive hook summary messages with the same hookLabel
//! (e.g. PostToolUse) into a single summary. This happens when parallel
//! tool calls each emit their own hook summary.

#![allow(dead_code)]

use crate::types::message::Message;

/// Check if a message is a labeled hook summary.
fn is_labeled_hook_summary(msg: &Message) -> bool {
    match msg {
        Message::System(sys) => {
            sys.subtype.as_deref() == Some("stop_hook_summary")
                && sys.extra.get("hookLabel").is_some()
        }
        _ => false,
    }
}

/// Get the hook label from a message if it's a labeled hook summary.
fn get_hook_label(msg: &Message) -> Option<&str> {
    if !is_labeled_hook_summary(msg) {
        return None;
    }
    match msg {
        Message::System(sys) => sys
            .extra
            .get("hookLabel")
            .and_then(|v| v.as_str()),
        _ => None,
    }
}

/// Collapses consecutive hook summary messages with the same hookLabel
/// into a single summary. This happens when parallel tool calls each emit
/// their own hook summary.
pub fn collapse_hook_summaries(messages: &[Message]) -> Vec<Message> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < messages.len() {
        let msg = &messages[i];
        if is_labeled_hook_summary(msg) {
            if let Some(label) = get_hook_label(msg) {
                let label = label.to_string();
                let mut group = Vec::new();
                while i < messages.len() {
                    let next = &messages[i];
                    if !is_labeled_hook_summary(next)
                        || get_hook_label(next) != Some(&label)
                    {
                        break;
                    }
                    group.push(next.clone());
                    i += 1;
                }
                if group.len() == 1 {
                    result.push(msg.clone());
                } else {
                    // Merge the group into a single summary
                    let merged = merge_hook_summaries(&group, &label);
                    result.push(merged);
                }
            } else {
                result.push(msg.clone());
                i += 1;
            }
        } else {
            result.push(msg.clone());
            i += 1;
        }
    }

    result
}

/// Merge a group of hook summaries into a single summary.
fn merge_hook_summaries(group: &[Message], label: &str) -> Message {
    use serde_json::json;

    let hook_count: i64 = group
        .iter()
        .filter_map(|m| m.extra.get("hookCount").and_then(|v| v.as_i64()))
        .sum();

    let hook_infos: Vec<serde_json::Value> = group
        .iter()
        .filter_map(|m| m.extra.get("hookInfos").cloned())
        .flat_map(|v| {
            v.as_array()
                .map(|arr| arr.to_vec())
                .unwrap_or_else(|| vec![v])
        })
        .collect();

    let hook_errors: Vec<serde_json::Value> = group
        .iter()
        .filter_map(|m| m.extra.get("hookErrors").cloned())
        .flat_map(|v| {
            v.as_array()
                .map(|arr| arr.to_vec())
                .unwrap_or_else(|| vec![v])
        })
        .collect();

    let prevented_continuation = group.iter().any(|m| {
        m.extra
            .get("preventedContinuation")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    });

    let has_output = group
        .iter()
        .any(|m| m.extra.get("hasOutput").and_then(|v| v.as_bool()).unwrap_or(false));

    // Parallel tool calls' hooks overlap; max is closest to wall-clock.
    let total_duration_ms = group
        .iter()
        .filter_map(|m| m.extra.get("totalDurationMs").and_then(|v| v.as_i64()))
        .max()
        .unwrap_or(0);

    // Build the merged message
    if let Message::System(first) = &group[0] {
        let mut merged_extra = first.extra.clone();
        merged_extra.insert("hookLabel".to_string(), json!(label));
        merged_extra.insert("hookCount".to_string(), json!(hook_count));
        merged_extra.insert("hookInfos".to_string(), json!(hook_infos));
        merged_extra.insert("hookErrors".to_string(), json!(hook_errors));
        merged_extra.insert(
            "preventedContinuation".to_string(),
            json!(prevented_continuation),
        );
        merged_extra.insert("hasOutput".to_string(), json!(has_output));
        merged_extra.insert("totalDurationMs".to_string(), json!(total_duration_ms));

        Message::System(crate::types::message::SystemMessage {
            base: first.base.clone(),
            message_type: "system".to_string(),
            subtype: Some("stop_hook_summary".to_string()),
            level: first.level.clone(),
            message: first.message.clone(),
            extra: merged_extra,
        })
    } else {
        group[0].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::message::{MessageBase, SystemMessage};

    fn make_hook_summary(label: &str, hook_count: i64, duration_ms: i64) -> Message {
        use serde_json::json;
        let mut extra = std::collections::HashMap::new();
        extra.insert("hookLabel".to_string(), json!(label));
        extra.insert("hookCount".to_string(), json!(hook_count));
        extra.insert("hookInfos".to_string(), json!([]));
        extra.insert("hookErrors".to_string(), json!([]));
        extra.insert("preventedContinuation".to_string(), json!(false));
        extra.insert("hasOutput".to_string(), json!(false));
        extra.insert("totalDurationMs".to_string(), json!(duration_ms));

        Message::System(SystemMessage {
            base: MessageBase::default(),
            message_type: "system".to_string(),
            subtype: Some("stop_hook_summary".to_string()),
            level: None,
            message: None,
            extra,
        })
    }

    #[test]
    fn test_collapse_same_label() {
        let messages = vec![
            make_hook_summary("PostToolUse", 1, 100),
            make_hook_summary("PostToolUse", 1, 200),
        ];
        let result = collapse_hook_summaries(&messages);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_no_collapse_different_labels() {
        let messages = vec![
            make_hook_summary("PostToolUse", 1, 100),
            make_hook_summary("PreToolUse", 1, 200),
        ];
        let result = collapse_hook_summaries(&messages);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_single_no_collapse() {
        let messages = vec![make_hook_summary("PostToolUse", 1, 100)];
        let result = collapse_hook_summaries(&messages);
        assert_eq!(result.len(), 1);
    }
}
