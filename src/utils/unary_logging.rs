// Source: ~/claudecode/openclaudecode/src/utils/unaryLogging.rs

use serde::Serialize;

/// Completion type for unary events.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionType {
    StrReplaceSingle,
    StrReplaceMulti,
    WriteFileSingle,
    ToolUseSingle,
}

/// Event type for unary logging.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnaryEvent {
    Accept,
    Reject,
    Response,
}

/// Metadata for unary event.
pub struct UnaryEventMetadata {
    pub language_name: String,
    pub message_id: String,
    pub platform: String,
    pub has_feedback: Option<bool>,
}

/// Log event for unary operations.
pub struct LogEvent {
    pub completion_type: CompletionType,
    pub event: UnaryEvent,
    pub metadata: UnaryEventMetadata,
}

/// Log a unary event.
pub async fn log_unary_event(event: LogEvent) {
    let event_str = match event.event {
        UnaryEvent::Accept => "accept",
        UnaryEvent::Reject => "reject",
        UnaryEvent::Response => "response",
    };

    let completion_type_str = match event.completion_type {
        CompletionType::StrReplaceSingle => "str_replace_single",
        CompletionType::StrReplaceMulti => "str_replace_multi",
        CompletionType::WriteFileSingle => "write_file_single",
        CompletionType::ToolUseSingle => "tool_use_single",
    };

    // Log the event
    tracing::info!(
        event = event_str,
        completion_type = completion_type_str,
        language_name = %event.metadata.language_name,
        message_id = %event.metadata.message_id,
        platform = %event.metadata.platform,
        has_feedback = ?event.metadata.has_feedback,
        "tengu_unary_event"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log_unary_event() {
        let event = LogEvent {
            completion_type: CompletionType::StrReplaceSingle,
            event: UnaryEvent::Accept,
            metadata: UnaryEventMetadata {
                language_name: "rust".to_string(),
                message_id: "msg-123".to_string(),
                platform: "linux".to_string(),
                has_feedback: Some(true),
            },
        };

        log_unary_event(event).await;
        // Should not panic
    }
}
