// Source: ~/claudecode/openclaudecode/src/utils/collapseBackgroundBashNotifications.ts
//! Collapses consecutive completed-background-bash task-notifications into a
//! single synthetic "N background commands completed" notification. Failed/killed
//! tasks and agent/workflow notifications are left alone. Monitor stream
//! events (enqueue_stream_event) have no <status> tag and never match.
//!
//! Pass-through in verbose mode so ctrl+O shows each completion.

#![allow(dead_code)]

use crate::constants::xml_tags::{STATUS_TAG, SUMMARY_TAG, TASK_NOTIFICATION_TAG};
use crate::types::message::{Message, MessageBase, UserMessage};

const BACKGROUND_BASH_SUMMARY_PREFIX: &str = "background command";

/// Extract the content of an XML tag from text.
fn extract_tag(text: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = text.find(&open)?;
    let end = text.find(&close)?;
    if end > start + open.len() {
        Some(text[start + open.len()..end].to_string())
    } else {
        Some(String::new())
    }
}

/// Check if a message is a completed background bash notification.
fn is_completed_background_bash(msg: &Message) -> bool {
    let text = match msg {
        Message::User(user) => match &user.message.content {
            crate::types::message::UserContent::Text(t) => t.clone(),
            crate::types::message::UserContent::Blocks(blocks) => blocks
                .first()
                .and_then(|b| b.text.as_ref())
                .cloned()
                .unwrap_or_default(),
        },
        _ => return false,
    };

    // Only collapse successful completions — failed/killed stay visible individually.
    if extract_tag(&text, STATUS_TAG).as_deref() != Some("completed") {
        return false;
    }

    // The prefix constant distinguishes bash-kind LocalShellTask completions from
    // agent/workflow/monitor notifications. Monitor-kind completions have their
    // own summary wording and deliberately don't collapse here.
    extract_tag(&text, SUMMARY_TAG)
        .map(|s| s.starts_with(BACKGROUND_BASH_SUMMARY_PREFIX))
        .unwrap_or(false)
}

/// Collapses consecutive completed-background-bash task-notifications into a
/// single synthetic "N background commands completed" notification.
///
/// Pass-through in verbose mode so ctrl+O shows each completion.
pub fn collapse_background_bash_notifications(
    messages: &[Message],
    verbose: bool,
) -> Vec<Message> {
    // In fullscreen mode, collapse. Otherwise pass through.
    if !is_fullscreen_env_enabled() {
        return messages.to_vec();
    }
    if verbose {
        return messages.to_vec();
    }

    let mut result = Vec::new();
    let mut i = 0;

    while i < messages.len() {
        let msg = &messages[i];
        if is_completed_background_bash(msg) {
            let mut count = 0;
            while i < messages.len()
                && is_completed_background_bash(&messages[i])
            {
                count += 1;
                i += 1;
            }
            if count == 1 {
                result.push(msg.clone());
            } else {
                // Synthesize a task-notification that UserAgentNotificationMessage
                // already knows how to render — no new renderer needed.
                if let Message::User(user) = msg {
                    let new_text = format!(
                        "<{TASK_NOTIFICATION_TAG}><{STATUS_TAG}>completed</{STATUS_TAG}><{SUMMARY_TAG}>{count} background commands completed</{SUMMARY_TAG}></{TASK_NOTIFICATION_TAG}>"
                    );
                    result.push(Message::User(UserMessage {
                        message: crate::types::message::UserMessageContent {
                            content: crate::types::message::UserContent::Text(new_text),
                            extra: user.message.extra.clone(),
                        },
                        is_meta: user.is_meta,
                        is_visible_in_transcript_only: user.is_visible_in_transcript_only,
                        is_virtual: user.is_virtual,
                        parent_uuid: user.parent_uuid.clone(),
                        timestamp: user.timestamp,
                        tool_use_result: user.tool_use_result.clone(),
                    }));
                }
            }
        } else {
            result.push(msg.clone());
            i += 1;
        }
    }

    result
}

/// Check if fullscreen env is enabled.
fn is_fullscreen_env_enabled() -> bool {
    crate::utils::fullscreen::is_fullscreen_env_enabled()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::message::{UserContent, UserMessageContent};

    fn make_background_bash(summary: &str) -> Message {
        let text = format!(
            "<{TASK_NOTIFICATION_TAG}><{STATUS_TAG}>completed</{STATUS_TAG}><{SUMMARY_TAG}>{summary}</{SUMMARY_TAG}></{TASK_NOTIFICATION_TAG}>"
        );
        Message::User(UserMessage {
            message: UserMessageContent {
                content: UserContent::Text(text),
                extra: Default::default(),
            },
            is_meta: None,
            is_visible_in_transcript_only: None,
            is_virtual: None,
            parent_uuid: None,
            timestamp: None,
            tool_use_result: None,
        })
    }

    fn make_other_message() -> Message {
        Message::User(UserMessage {
            message: UserMessageContent {
                content: UserContent::Text("some other message".to_string()),
                extra: Default::default(),
            },
            is_meta: None,
            is_visible_in_transcript_only: None,
            is_virtual: None,
            parent_uuid: None,
            timestamp: None,
            tool_use_result: None,
        })
    }

    #[test]
    fn test_collapse_multiple_background_bash() {
        // Without fullscreen enabled, pass through
        let messages = vec![
            make_background_bash("background command ls"),
            make_background_bash("background command cat"),
        ];
        let result = collapse_background_bash_notifications(&messages, false);
        // Since fullscreen is not enabled in test env, pass through
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_verbose_mode_passes_through() {
        let messages = vec![
            make_background_bash("background command ls"),
            make_background_bash("background command cat"),
        ];
        let result = collapse_background_bash_notifications(&messages, true);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_non_background_messages_pass_through() {
        let messages = vec![make_other_message()];
        let result = collapse_background_bash_notifications(&messages, false);
        assert_eq!(result.len(), 1);
    }
}
