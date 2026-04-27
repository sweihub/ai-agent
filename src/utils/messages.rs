// Source: /data/home/swei/claudecode/openclaudecode/src/constants/messages.ts
//! Message utilities and helpers
//! Translated from /data/home/swei/claudecode/openclaudecode/src/utils/messages.ts

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// Message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    User(UserMessage),
    Assistant(AssistantMessage),
    Progress(ProgressMessage),
    Attachment(AttachmentMessage),
    System(SystemMessage),
}

/// User message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub message: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_meta: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_visible_in_transcript_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_virtual: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_compact_summary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summarize_metadata: Option<SummarizeMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_meta: Option<serde_json::Value>,
    pub uuid: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_paste_ids: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_tool_assistant_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<MessageOrigin>,
}

/// Summarize metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizeMetadata {
    pub messages_summarized: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
}

/// Message origin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageOrigin {
    #[serde(rename = "type")]
    pub origin_type: String,
}

/// Message content (can be string or array of content blocks)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    String(String),
    Blocks(Vec<ContentBlock>),
}

/// Content block
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text {
        text: String,
    },
    Image {
        source: ImageSource,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        content: Option<Vec<ContentBlock>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
    // Server-side tool results
    #[serde(rename = "server_tool_use")]
    ServerToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "mcp_tool_use")]
    McpToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "advisor_tool_result")]
    AdvisorToolResult {
        tool_use_id: String,
        content: serde_json::Value,
    },
    #[serde(rename = "web_search_tool_result")]
    WebSearchToolResult {
        tool_use_id: String,
        content: serde_json::Value,
    },
    #[serde(rename = "web_fetch_tool_result")]
    WebFetchToolResult {
        tool_use_id: String,
        content: serde_json::Value,
    },
    #[serde(rename = "tool_reference")]
    ToolReference {
        tool_name: String,
    },
}

/// Image source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

/// Assistant message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub message: AssistantMessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_error: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_api_error_message: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_virtual: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_meta: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisor_model: Option<String>,
    pub uuid: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_uuid: Option<String>,
}

/// Assistant message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessageContent {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
    pub model: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
    #[serde(rename = "type")]
    pub message_type: String,
    pub usage: Option<Usage>,
    pub content: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_management: Option<serde_json::Value>,
}

/// Usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    #[serde(rename = "input_tokens")]
    pub input_tokens: u32,
    #[serde(rename = "output_tokens")]
    pub output_tokens: u32,
    #[serde(rename = "cache_creation_input_tokens")]
    pub cache_creation_input_tokens: u32,
    #[serde(rename = "cache_read_input_tokens")]
    pub cache_read_input_tokens: u32,
    #[serde(rename = "server_tool_use")]
    pub server_tool_use: ServerToolUse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation: Option<CacheCreation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inference_geo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iterations: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
}

/// Server tool use stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerToolUse {
    #[serde(rename = "web_search_requests")]
    pub web_search_requests: u32,
    #[serde(rename = "web_fetch_requests")]
    pub web_fetch_requests: u32,
}

/// Cache creation stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheCreation {
    #[serde(rename = "ephemeral_1h_input_tokens")]
    pub ephemeral_1h_input_tokens: u32,
    #[serde(rename = "ephemeral_5m_input_tokens")]
    pub ephemeral_5m_input_tokens: u32,
}

/// Progress message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressMessage<T = serde_json::Value> {
    #[serde(rename = "type")]
    pub data_type: String,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
    pub uuid: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_uuid: Option<String>,
}

/// Attachment message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMessage {
    pub attachment: serde_json::Value,
    pub uuid: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_uuid: Option<String>,
}

/// System message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    pub message: SystemMessageContent,
    pub uuid: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_uuid: Option<String>,
}

/// System message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessageContent {
    #[serde(rename = "type")]
    pub message_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<SystemMessageLevel>,
}

/// System message level
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SystemMessageLevel {
    Info,
    Warning,
    Error,
}

// === Constants ===

pub const INTERRUPT_MESSAGE: &str = "[Request interrupted by user]";
pub const INTERRUPT_MESSAGE_FOR_TOOL_USE: &str = "[Request interrupted by user for tool use]";
pub const CANCEL_MESSAGE: &str = "The user doesn't want to take this action right now. STOP what you are doing and wait for the user to tell you how to proceed.";
pub const REJECT_MESSAGE: &str = "The user doesn't want to proceed with this tool use. The tool use was rejected (eg. if it was a file edit, the new_string was NOT written to the file). STOP what you are doing and wait for the user to tell you how to proceed.";
pub const REJECT_MESSAGE_WITH_REASON_PREFIX: &str = "The user doesn't want to proceed with this tool use. The tool use was rejected (eg. if it was a file edit, the new_string was NOT written to the file). To tell you how to proceed, the user said:\n";
pub const SUBAGENT_REJECT_MESSAGE: &str = "Permission for this tool use was denied. The tool use was rejected (eg. if it was a file edit, the new_string was NOT written to the file). Try a different approach or report the limitation to complete your task.";
pub const SUBAGENT_REJECT_MESSAGE_WITH_REASON_PREFIX: &str = "Permission for this tool use was denied. The tool use was rejected (eg. if it was a file edit, the new_string was NOT written to the file). The user said:\n";
pub const NO_RESPONSE_REQUESTED: &str = "No response requested.";
pub const SYNTHETIC_MODEL: &str = "<synthetic>";

// Denial workaround guidance
pub const DENIAL_WORKAROUND_GUIDANCE: &str = "IMPORTANT: You *may* attempt to accomplish this action using other tools that might naturally be used to accomplish this goal, e.g. using head instead of cat. But you *should not* attempt to work around this denial in malicious ways, e.g. do not use your ability to run tests to execute non-test actions. You should only try to work around this restriction in reasonable ways that do not attempt to bypass the intent behind this denial. If you believe this capability is essential to complete your request, STOP and explain to the user what you were trying to do and why you need this permission. Let the user decide how to proceed.";

const AUTO_MODE_REJECTION_PREFIX: &str = "Permission for this action has been denied. Reason: ";

// === Functions ===

/// Build rejection message for auto mode classifier denials
pub fn build_yolo_rejection_message(reason: &str) -> String {
    format!(
        "{}{}. If you have other tasks that don't depend on this action, continue working on those. {} To allow this type of action in the future, the user can add a Bash permission rule to their settings.",
        AUTO_MODE_REJECTION_PREFIX, reason, DENIAL_WORKAROUND_GUIDANCE
    )
}

/// Build message for when classifier is unavailable
pub fn build_classifier_unavailable_message(tool_name: &str, classifier_model: &str) -> String {
    format!(
        "{} is temporarily unavailable, so auto mode cannot determine the safety of {} right now. Wait briefly and then try this action again. If it keeps failing, continue with other tasks that don't require this action and come back to it later. Note: reading files, searching code, and other read-only operations do not require the classifier and can still be used.",
        classifier_model, tool_name
    )
}

/// Check if tool result message is a classifier denial
pub fn is_classifier_denial(content: &str) -> bool {
    content.starts_with(AUTO_MODE_REJECTION_PREFIX)
}

/// Auto reject message
pub fn auto_reject_message(tool_name: &str) -> String {
    format!(
        "Permission to use {} has been denied. {}",
        tool_name, DENIAL_WORKAROUND_GUIDANCE
    )
}

/// Don't ask reject message
pub fn dont_ask_reject_message(tool_name: &str) -> String {
    format!(
        "Permission to use {} has been denied because Claude Code is running in don't ask mode. {}",
        tool_name, DENIAL_WORKAROUND_GUIDANCE
    )
}

/// Derive short message ID (6-char base36) from UUID
pub fn derive_short_message_id(uuid: &str) -> String {
    // Take first 10 hex chars from the UUID (skipping dashes)
    let hex: String = uuid.replace('-', "").chars().take(10).collect();
    // Convert to base36 for shorter representation, take 6 chars
    let parsed = u64::from_str_radix(&hex, 16).unwrap_or(0);
    let base36 = format!("{:36}", parsed);
    base36.chars().take(6).collect()
}

/// Create a user message
pub fn create_user_message(content: impl Into<MessageContent>) -> UserMessage {
    let content = content.into();
    UserMessage {
        message: content,
        is_meta: None,
        is_visible_in_transcript_only: None,
        is_virtual: None,
        is_compact_summary: None,
        summarize_metadata: None,
        tool_use_result: None,
        mcp_meta: None,
        uuid: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        image_paste_ids: None,
        source_tool_assistant_uuid: None,
        permission_mode: None,
        origin: None,
    }
}

/// Create assistant message
pub fn create_assistant_message(content: Vec<serde_json::Value>) -> AssistantMessage {
    AssistantMessage {
        message: AssistantMessageContent {
            id: uuid::Uuid::new_v4().to_string(),
            container: None,
            model: SYNTHETIC_MODEL.to_string(),
            role: "assistant".to_string(),
            stop_reason: Some("stop_sequence".to_string()),
            stop_sequence: Some("".to_string()),
            message_type: "message".to_string(),
            usage: Some(Usage::default()),
            content,
            context_management: None,
        },
        request_id: None,
        api_error: None,
        error: None,
        error_details: None,
        is_api_error_message: Some(false),
        is_virtual: None,
        is_meta: None,
        advisor_model: None,
        uuid: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        parent_uuid: None,
    }
}

/// Create progress message
pub fn create_progress_message(
    tool_use_id: &str,
    parent_tool_use_id: &str,
    data: serde_json::Value,
) -> ProgressMessage {
    ProgressMessage {
        data_type: "progress".to_string(),
        data,
        tool_use_id: Some(tool_use_id.to_string()),
        parent_tool_use_id: Some(parent_tool_use_id.to_string()),
        uuid: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        parent_uuid: None,
    }
}

/// Create tool result stop message
pub fn create_tool_result_stop_message(tool_use_id: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "tool_result",
        "content": CANCEL_MESSAGE,
        "is_error": true,
        "tool_use_id": tool_use_id
    })
}

/// XML tags for command input/output
pub const COMMAND_MESSAGE_TAG: &str = "command-message";
pub const COMMAND_NAME_TAG: &str = "command-name";

/// Create a synthetic user caveat message (informs the model the user typed something)
pub fn create_synthetic_user_caveat_message() -> Message {
    let content = "The user didn't say anything. Continue working.".to_string();
    Message::User(UserMessage {
        message: MessageContent::String(content),
        is_meta: Some(true),
        is_visible_in_transcript_only: None,
        is_virtual: None,
        is_compact_summary: None,
        summarize_metadata: None,
        tool_use_result: None,
        mcp_meta: None,
        uuid: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        image_paste_ids: None,
        source_tool_assistant_uuid: None,
        permission_mode: None,
        origin: None,
    })
}

/// Create a system message
pub fn create_system_message(content: impl Into<String>, level: SystemMessageLevel) -> Message {
    Message::System(SystemMessage {
        message: SystemMessageContent {
            message_type: "system".to_string(),
            subtype: None,
            content: content.into(),
            level: Some(level),
        },
        uuid: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        parent_uuid: None,
    })
}

/// Create a system local command message (for command output that's visible but not sent to model)
pub fn create_system_local_command_message(content: impl Into<String>) -> Message {
    Message::System(SystemMessage {
        message: SystemMessageContent {
            message_type: "system".to_string(),
            subtype: Some("local_command".to_string()),
            content: content.into(),
            level: None,
        },
        uuid: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        parent_uuid: None,
    })
}

/// Create a user interruption message
pub fn create_user_interruption_message(tool_use: bool) -> Message {
    let content = if tool_use {
        INTERRUPT_MESSAGE_FOR_TOOL_USE.to_string()
    } else {
        INTERRUPT_MESSAGE.to_string()
    };
    Message::User(UserMessage {
        message: MessageContent::String(content),
        is_meta: None,
        is_visible_in_transcript_only: None,
        is_virtual: None,
        is_compact_summary: None,
        summarize_metadata: None,
        tool_use_result: None,
        mcp_meta: None,
        uuid: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        image_paste_ids: None,
        source_tool_assistant_uuid: None,
        permission_mode: None,
        origin: None,
    })
}

/// Format command input with XML tags: `<command-message>name</command-message>\n<command-name>/name</command-name>`
pub fn format_command_input_tags(command_name: &str, args: &str) -> String {
    let mut parts = vec![
        format!("<{COMMAND_MESSAGE_TAG}>{command_name}</{COMMAND_MESSAGE_TAG}>"),
        format!("<{COMMAND_NAME_TAG}>/{command_name}</{COMMAND_NAME_TAG}>"),
    ];
    if !args.trim().is_empty() {
        parts.push(format!("<command-args>{args}</command-args>"));
    }
    parts.join("\n")
}

/// Check if a message is a system local command message
pub fn is_system_local_command_message(message: &Message) -> bool {
    match message {
        Message::System(sys) => sys.message.subtype.as_deref() == Some("local_command"),
        _ => false,
    }
}

/// Check if a message is a compact boundary message (used by /compact)
pub fn is_compact_boundary_message(message: &Message) -> bool {
    match message {
        Message::User(user) => user.is_compact_summary == Some(true),
        Message::System(sys) => sys.message.subtype.as_deref() == Some("compact"),
        _ => false,
    }
}

/// Extract tag from HTML-like content
pub fn extract_tag(html: &str, tag_name: &str) -> Option<String> {
    use regex::Regex;

    if html.trim().is_empty() || tag_name.trim().is_empty() {
        return None;
    }

    let escaped_tag = tag_name.replace(
        [
            '.', '*', '+', '?', '^', '$', '{', '}', '[', ']', '(', ')', '|', '\\',
        ],
        "\\$&",
    );

    let pattern = format!(
        r"<{}(?:\s+[^>]*)?>([\s\S]*?)</{}>",
        escaped_tag, escaped_tag
    );

    let re = Regex::new(&pattern).ok()?;

    let mut depth = 0i32;
    let mut last_index = 0;

    let opening_tag_re = Regex::new(&format!(r"<{}(?:\s+[^>]*)?>", escaped_tag)).ok()?;
    let closing_tag_re = Regex::new(&format!(r"</{}>", escaped_tag)).ok()?;

    for caps in re.captures_iter(html) {
        let content = caps.get(1)?.as_str();
        let start = caps.get(0)?.start();

        depth = 0;

        for _ in opening_tag_re.find_iter(&html[..start]) {
            depth += 1;
        }

        for _ in closing_tag_re.find_iter(&html[..start]) {
            depth -= 1;
        }

        if depth == 0 && !content.is_empty() {
            return Some(content.to_string());
        }

        last_index = start + caps.get(0)?.len();
    }

    None
}

/// Check if message is not empty
pub fn is_not_empty_message(message: &Message) -> bool {
    match message {
        Message::Progress(_) | Message::Attachment(_) | Message::System(_) => true,
        Message::User(user) => {
            match &user.message {
                MessageContent::String(s) => !s.trim().is_empty(),
                MessageContent::Blocks(blocks) => {
                    if blocks.is_empty() {
                        return false;
                    }
                    // Skip multi-block messages for now
                    if blocks.len() > 1 {
                        return true;
                    }
                    // Check first block
                    match &blocks[0] {
                        ContentBlock::Text { text } => {
                            !text.trim().is_empty()
                                && text != NO_RESPONSE_REQUESTED
                                && text != INTERRUPT_MESSAGE_FOR_TOOL_USE
                        }
                        _ => true,
                    }
                }
            }
        }
        Message::Assistant(assistant) => !assistant.message.content.is_empty(),
    }
}

/// Normalized message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NormalizedMessage {
    User(NormalizedUserMessage),
    Assistant(NormalizedAssistantMessage),
    Progress(ProgressMessage),
    Attachment(AttachmentMessage),
    System(SystemMessage),
}

/// Normalized user message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedUserMessage {
    pub message: MessageContent,
    #[serde(flatten)]
    pub extra: UserMessageExtra,
}

/// Normalized assistant message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedAssistantMessage {
    pub message: AssistantMessageContent,
    #[serde(flatten)]
    pub extra: AssistantMessageExtra,
}

/// Extra fields for normalized user message
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserMessageExtra {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_meta: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_visible_in_transcript_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_virtual: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_compact_summary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summarize_metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_meta: Option<serde_json::Value>,
    pub uuid: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_paste_ids: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_tool_assistant_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<MessageOrigin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_uuid: Option<String>,
}

/// Extra fields for normalized assistant message
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssistantMessageExtra {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_error: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_api_error_message: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_virtual: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_meta: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisor_model: Option<String>,
    pub uuid: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_uuid: Option<String>,
}

/// Derive UUID from parent UUID and index
pub fn derive_uuid(parent_uuid: &str, index: usize) -> String {
    let hex = format!("{:012x}", index);
    let parent_trimmed = parent_uuid.replace('-', "");
    let prefix = &parent_trimmed[..24.min(parent_trimmed.len())];
    format!("{}-{}-{}", &prefix[0..8], &prefix[8..12], hex)
}

/// Get tool use ID from a message
pub fn get_tool_use_id(message: &NormalizedMessage) -> Option<String> {
    match message {
        NormalizedMessage::Assistant(msg) => {
            if let Some(first) = msg.message.content.first() {
                if let Ok(block) = serde_json::from_value::<ContentBlock>(first.clone()) {
                    match block {
                        ContentBlock::ToolUse { id, .. } => Some(id),
                        _ => None,
                    }
                } else {
                    // Try to extract id from raw JSON
                    first.get("id").and_then(|v| v.as_str()).map(String::from)
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Message lookups for efficient O(1) access
#[derive(Debug, Default)]
pub struct MessageLookups {
    pub sibling_tool_use_ids: HashMap<String, HashSet<String>>,
    pub progress_messages_by_tool_use_id: HashMap<String, Vec<ProgressMessage>>,
    pub in_progress_hook_counts: HashMap<String, HashMap<String, u32>>,
    pub resolved_hook_counts: HashMap<String, HashMap<String, u32>>,
    pub tool_result_by_tool_use_id: HashMap<String, NormalizedMessage>,
    pub tool_use_by_tool_use_id: HashMap<String, serde_json::Value>,
    pub normalized_message_count: usize,
    pub resolved_tool_use_ids: HashSet<String>,
    pub errored_tool_use_ids: HashSet<String>,
}

/// Build message lookups from normalized messages
pub fn build_message_lookups(
    normalized_messages: &[NormalizedMessage],
    messages: &[Message],
) -> MessageLookups {
    let mut lookups = MessageLookups::default();

    // First pass: collect tool use IDs by message ID
    let mut tool_use_ids_by_message_id: HashMap<String, HashSet<String>> = HashMap::new();
    let mut tool_use_id_to_message_id: HashMap<String, String> = HashMap::new();
    let mut tool_use_by_tool_use_id: HashMap<String, serde_json::Value> = HashMap::new();

    for msg in messages {
        if let Message::Assistant(assistant) = msg {
            let id = &assistant.message.id;
            let mut tool_use_ids = HashSet::new();
            for content in &assistant.message.content {
                if let Ok(block) = serde_json::from_value::<ContentBlock>(content.clone()) {
                    if let ContentBlock::ToolUse { id: tool_id, .. } = block {
                        tool_use_ids.insert(tool_id.clone());
                        tool_use_id_to_message_id.insert(tool_id.clone(), id.clone());
                        tool_use_by_tool_use_id.insert(tool_id.clone(), content.clone());
                    }
                }
            }
            if !tool_use_ids.is_empty() {
                tool_use_ids_by_message_id.insert(id.clone(), tool_use_ids);
            }
        }
    }

    // Build sibling lookup
    for (tool_use_id, message_id) in &tool_use_id_to_message_id {
        if let Some(ids) = tool_use_ids_by_message_id.get(message_id) {
            lookups
                .sibling_tool_use_ids
                .insert(tool_use_id.clone(), ids.clone());
        }
    }

    // Second pass: build progress, hook, and tool result lookups
    for msg in normalized_messages {
        if let NormalizedMessage::Progress(progress) = msg {
            let tool_use_id = progress.parent_tool_use_id.clone().unwrap_or_default();
            if !tool_use_id.is_empty() {
                lookups
                    .progress_messages_by_tool_use_id
                    .entry(tool_use_id.clone())
                    .or_insert_with(Vec::new)
                    .push(progress.clone());
            }
        }

        // Tool result lookup
        if let NormalizedMessage::User(user) = msg {
            if let MessageContent::Blocks(blocks) = &user.message {
                for block in blocks {
                    if let ContentBlock::ToolResult {
                        tool_use_id,
                        is_error,
                        ..
                    } = block
                    {
                        lookups.resolved_tool_use_ids.insert(tool_use_id.clone());
                        if is_error == &Some(true) {
                            lookups.errored_tool_use_ids.insert(tool_use_id.clone());
                        }
                    }
                }
            }
        }

        // Server tool results
        if let NormalizedMessage::Assistant(assistant) = msg {
            for content in &assistant.message.content {
                // Check for server_tool_use, mcp_tool_use
                if let Some(tool_use_id) = content.get("id") {
                    if let Some(id_str) = tool_use_id.as_str() {
                        // Check if there's a corresponding result
                        let has_result = lookups.resolved_tool_use_ids.contains(id_str);
                        if !has_result {
                            // Mark as resolved but not errored
                            lookups.resolved_tool_use_ids.insert(id_str.to_string());
                        }
                    }
                }
            }
        }
    }

    lookups.tool_use_by_tool_use_id = tool_use_by_tool_use_id;
    lookups.normalized_message_count = normalized_messages.len();

    lookups
}

/// Get sibling tool use IDs from lookup
pub fn get_sibling_tool_use_ids_from_lookup(
    message: &NormalizedMessage,
    lookups: &MessageLookups,
) -> HashSet<String> {
    let tool_use_id = match get_tool_use_id(message) {
        Some(id) => id,
        None => return HashSet::new(),
    };
    lookups
        .sibling_tool_use_ids
        .get(&tool_use_id)
        .cloned()
        .unwrap_or_default()
}

/// Get progress messages from lookup
pub fn get_progress_messages_from_lookup(
    message: &NormalizedMessage,
    lookups: &MessageLookups,
) -> Vec<ProgressMessage> {
    let tool_use_id = match get_tool_use_id(message) {
        Some(id) => id,
        None => return Vec::new(),
    };
    lookups
        .progress_messages_by_tool_use_id
        .get(&tool_use_id)
        .cloned()
        .unwrap_or_default()
}

/// Get tool result IDs from normalized messages
pub fn get_tool_result_ids(normalized_messages: &[NormalizedMessage]) -> HashMap<String, bool> {
    let mut result = HashMap::new();

    for msg in normalized_messages {
        if let NormalizedMessage::User(user) = msg {
            if let MessageContent::Blocks(blocks) = &user.message {
                for block in blocks {
                    if let ContentBlock::ToolResult {
                        tool_use_id,
                        is_error,
                        ..
                    } = block
                    {
                        result.insert(tool_use_id.clone(), is_error.unwrap_or(false));
                    }
                }
            }
        }
    }

    result
}

/// Reorder attachments for API (bubble up until hitting tool result or assistant)
pub fn reorder_attachments_for_api(messages: Vec<Message>) -> Vec<Message> {
    let mut result = Vec::new();
    let mut pending_attachments: Vec<Message> = Vec::new();

    // Scan from bottom to top
    for i in (0..messages.len()).rev() {
        let message = messages[i].clone();

        if let Message::Attachment(_) = message {
            pending_attachments.push(message);
        } else {
            let is_stopping_point = matches!(
                message,
                Message::Assistant(_) | Message::User(_) if has_tool_result(&message)
            );

            if is_stopping_point && !pending_attachments.is_empty() {
                // Reverse pending attachments to maintain order
                for att in pending_attachments.drain(..).rev() {
                    result.push(att);
                }
                result.push(message);
            } else {
                result.push(message);
            }
        }
    }

    // Remaining attachments go to the top
    for att in pending_attachments.drain(..).rev() {
        result.push(att);
    }

    result.reverse();
    result
}

/// Check if message has tool result
fn has_tool_result(message: &Message) -> bool {
    if let Message::User(user) = message {
        if let MessageContent::Blocks(blocks) = &user.message {
            return blocks
                .iter()
                .any(|b| matches!(b, ContentBlock::ToolResult { .. }));
        }
    }
    false
}

/// Check if message is a tool use request
pub fn is_tool_use_request_message(message: &Message) -> bool {
    if let Message::Assistant(assistant) = message {
        assistant.message.content.iter().any(|c| {
            if let Ok(block) = serde_json::from_value::<ContentBlock>(c.clone()) {
                matches!(block, ContentBlock::ToolUse { .. })
            } else {
                c.get("type").and_then(|t| t.as_str()) == Some("tool_use")
            }
        })
    } else {
        false
    }
}

/// Check if message is a tool result message
pub fn is_tool_use_result_message(message: &Message) -> bool {
    if let Message::User(user) = message {
        if let MessageContent::Blocks(blocks) = &user.message {
            return blocks
                .iter()
                .any(|b| matches!(b, ContentBlock::ToolResult { .. }));
        }
    }
    false
}

/// Get last assistant message
pub fn get_last_assistant_message(messages: &[Message]) -> Option<&AssistantMessage> {
    messages.iter().rev().find_map(|m| {
        if let Message::Assistant(a) = m {
            Some(a)
        } else {
            None
        }
    })
}

/// Check if last assistant turn has tool calls
pub fn has_tool_calls_in_last_assistant_turn(messages: &[Message]) -> bool {
    for msg in messages.iter().rev() {
        if let Message::Assistant(assistant) = msg {
            return assistant.message.content.iter().any(|c| {
                if let Ok(block) = serde_json::from_value::<ContentBlock>(c.clone()) {
                    matches!(block, ContentBlock::ToolUse { .. })
                } else {
                    c.get("type").and_then(|t| t.as_str()) == Some("tool_use")
                }
            });
        }
    }
    false
}

/// Empty lookups for static rendering
pub fn empty_lookups() -> MessageLookups {
    MessageLookups::default()
}

/// Empty string set singleton
pub fn empty_string_set() -> HashSet<String> {
    HashSet::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_short_message_id() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let short_id = derive_short_message_id(uuid);
        assert_eq!(short_id.len(), 6);
    }

    #[test]
    fn test_build_yolo_rejection_message() {
        let msg = build_yolo_rejection_message("dangerous command");
        assert!(msg.contains("Permission for this action has been denied"));
        assert!(msg.contains("dangerous command"));
    }

    #[test]
    fn test_is_classifier_denial() {
        assert!(is_classifier_denial(
            "Permission for this action has been denied. Reason: testing"
        ));
        assert!(!is_classifier_denial("Just a regular message"));
    }

    #[test]
    fn test_extract_tag() {
        let html = "<test>Hello World</test>";
        let extracted = extract_tag(html, "test");
        assert_eq!(extracted, Some("Hello World".to_string()));
    }

    #[test]
    fn test_derive_uuid() {
        let parent = "550e8400-e29b-41d4-a716-446655440000";
        let derived = derive_uuid(parent, 0);
        assert!(!derived.is_empty());
    }

    #[test]
    fn test_get_tool_result_ids() {
        let messages = vec![NormalizedMessage::User(NormalizedUserMessage {
            message: MessageContent::Blocks(vec![ContentBlock::ToolResult {
                tool_use_id: "test-id".to_string(),
                content: None,
                is_error: Some(false),
            }]),
            extra: UserMessageExtra {
                uuid: "uuid".to_string(),
                timestamp: "timestamp".to_string(),
                ..Default::default()
            },
        })];

        let ids = get_tool_result_ids(&messages);
        assert!(ids.contains_key("test-id"));
    }
}
