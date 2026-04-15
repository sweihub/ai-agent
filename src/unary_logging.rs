//! Unary logging types for completion events.

#[derive(Debug, Clone)]
pub enum CompletionType {
    StrReplaceSingle,
    StrReplaceMulti,
    WriteFileSingle,
    ToolUseSingle,
}

impl CompletionType {
    pub fn as_str(&self) -> &str {
        match self {
            CompletionType::StrReplaceSingle => "str_replace_single",
            CompletionType::StrReplaceMulti => "str_replace_multi",
            CompletionType::WriteFileSingle => "write_file_single",
            CompletionType::ToolUseSingle => "tool_use_single",
        }
    }
}

#[derive(Debug, Clone)]
pub enum LogEvent {
    Accept,
    Reject,
    Response,
}

impl LogEvent {
    pub fn as_str(&self) -> &str {
        match self {
            LogEvent::Accept => "accept",
            LogEvent::Reject => "reject",
            LogEvent::Response => "response",
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnaryLogEvent {
    pub completion_type: CompletionType,
    pub event: LogEvent,
    pub metadata: UnaryLogMetadata,
}

#[derive(Debug, Clone)]
pub struct UnaryLogMetadata {
    pub language_name: String,
    pub message_id: String,
    pub platform: String,
    pub has_feedback: Option<bool>,
}
