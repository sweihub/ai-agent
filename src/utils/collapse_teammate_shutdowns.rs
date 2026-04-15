// Source: ~/claudecode/openclaudecode/src/utils/collapseTeammateShutdowns.ts
//! Collapses consecutive in-process teammate shutdown task_status attachments
//! into a single `teammate_shutdown_batch` attachment with a count.

#![allow(dead_code)]

use crate::types::message::{AttachmentMessage, Message, MessageBase};
use serde::{Deserialize, Serialize};

/// Check if a message is a teammate shutdown attachment.
fn is_teammate_shutdown_attachment(msg: &Message) -> bool {
    match msg {
        Message::Attachment(att) => {
            let attachment_type = att.extra.get("type").and_then(|v| v.as_str());
            let task_type = att.extra.get("taskType").and_then(|v| v.as_str());
            let status = att.extra.get("status").and_then(|v| v.as_str());
            attachment_type == Some("task_status")
                && task_type == Some("in_process_teammate")
                && status == Some("completed")
        }
        _ => false,
    }
}

/// Attachment type for a batch of teammate shutdowns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeammateShutdownBatchAttachment {
    #[serde(rename = "type")]
    pub attachment_type: String,
    pub count: usize,
}

/// Collapses consecutive in-process teammate shutdown task_status attachments
/// into a single `teammate_shutdown_batch` attachment with a count.
pub fn collapse_teammate_shutdowns(messages: &[Message]) -> Vec<Message> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < messages.len() {
        let msg = &messages[i];
        if is_teammate_shutdown_attachment(msg) {
            let mut count = 0;
            while i < messages.len() && is_teammate_shutdown_attachment(&messages[i]) {
                count += 1;
                i += 1;
            }
            if count == 1 {
                result.push(msg.clone());
            } else {
                // Create a batch attachment
                let batch = TeammateShutdownBatchAttachment {
                    attachment_type: "teammate_shutdown_batch".to_string(),
                    count,
                };
                let batch_json = serde_json::to_value(&batch).unwrap_or_default();

                if let Message::Attachment(att) = msg {
                    let mut new_extra = att.extra.clone();
                    new_extra.insert("type".to_string(), batch_json["type"].clone());
                    new_extra.insert("count".to_string(), batch_json["count"].clone());

                    result.push(Message::Attachment(AttachmentMessage {
                        base: att.base.clone(),
                        message_type: "attachment".to_string(),
                        path: att.path.clone(),
                        extra: new_extra,
                    }));
                } else {
                    // Fallback: create a new attachment message
                    result.push(Message::Attachment(AttachmentMessage {
                        base: MessageBase::default(),
                        message_type: "attachment".to_string(),
                        path: None,
                        extra: {
                            let mut map = std::collections::HashMap::new();
                            map.insert("type".to_string(), batch_json["type"].clone());
                            map.insert("count".to_string(), batch_json["count"].clone());
                            map
                        },
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_teammate_shutdown(uuid: Option<String>) -> Message {
        let mut extra = std::collections::HashMap::new();
        extra.insert("type".to_string(), json!("task_status"));
        extra.insert("taskType".to_string(), json!("in_process_teammate"));
        extra.insert("status".to_string(), json!("completed"));

        Message::Attachment(AttachmentMessage {
            base: MessageBase {
                uuid,
                ..Default::default()
            },
            message_type: "attachment".to_string(),
            path: None,
            extra,
        })
    }

    #[test]
    fn test_collapse_multiple_shutdowns() {
        let messages = vec![
            make_teammate_shutdown(Some("uuid-1".to_string())),
            make_teammate_shutdown(Some("uuid-2".to_string())),
            make_teammate_shutdown(Some("uuid-3".to_string())),
        ];
        let result = collapse_teammate_shutdowns(&messages);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_single_no_collapse() {
        let messages = vec![make_teammate_shutdown(Some("uuid-1".to_string()))];
        let result = collapse_teammate_shutdowns(&messages);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_non_teammate_messages_pass_through() {
        let mut extra = std::collections::HashMap::new();
        extra.insert("type".to_string(), json!("other"));
        let other_msg = Message::Attachment(AttachmentMessage {
            base: MessageBase::default(),
            message_type: "attachment".to_string(),
            path: None,
            extra,
        });

        let messages = vec![
            other_msg.clone(),
            make_teammate_shutdown(Some("uuid-1".to_string())),
            make_teammate_shutdown(Some("uuid-2".to_string())),
            other_msg.clone(),
        ];
        let result = collapse_teammate_shutdowns(&messages);
        assert_eq!(result.len(), 3);
    }
}
