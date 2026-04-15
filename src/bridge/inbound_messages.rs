//! Process inbound user messages from the bridge.
//!
//! Translated from openclaudecode/src/bridge/inboundMessages.ts

use serde::{Deserialize, Serialize};

// =============================================================================
// TYPES
// =============================================================================

/// SDK message type for bridge communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SDKMessage {
    User {
        message: Option<UserMessageContent>,
        uuid: Option<String>,
    },
    Assistant {
        message: Option<AssistantMessageContent>,
        uuid: Option<String>,
    },
    ToolUse {
        message: Option<ToolUseMessageContent>,
        uuid: Option<String>,
    },
    ToolResult {
        message: Option<ToolResultMessageContent>,
        uuid: Option<String>,
    },
    System {
        message: Option<SystemMessageContent>,
        uuid: Option<String>,
    },
}

impl SDKMessage {
    /// Create a placeholder user message with session_id (for bridge internal use).
    pub fn user_message_with_session(_session_id: String) -> Self {
        SDKMessage::User {
            message: None,
            uuid: None,
        }
    }
}

/// User message content (can be string or content blocks).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserMessageContent {
    String(String),
    Blocks(Vec<ContentBlock>),
}

/// Assistant message content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessageContent {
    pub content: Option<serde_json::Value>,
}

/// Tool use message content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseMessageContent {
    pub content: Option<serde_json::Value>,
}

/// Tool result message content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultMessageContent {
    pub content: Option<serde_json::Value>,
}

/// System message content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessageContent {
    pub content: Option<serde_json::Value>,
}

/// Content block parameter (supports text, image, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "source", rename_all = "snake_case")]
pub enum ContentBlock {
    Text {
        text: String,
    },
    Image {
        #[serde(rename = "media_type")]
        media_type: Option<String>,
        data: String,
    },
    // Add other variants as needed
    #[serde(other)]
    Other,
}

/// Image block parameter with source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBlock {
    #[serde(rename = "media_type")]
    pub media_type: Option<String>,
    pub r#type: String,
    pub data: String,
}

// =============================================================================
// MESSAGE EXTRACTION
// =============================================================================

/// Result of extracting inbound message fields.
#[derive(Debug, Clone)]
pub struct InboundMessageFields {
    pub content: String,
    pub uuid: Option<String>,
}

/// Process an inbound user message from the bridge, extracting content
/// and UUID for enqueueing. Supports both string content and
/// ContentBlockParam[] (e.g. messages containing images).
///
/// Returns the extracted fields, or None if the message should be
/// skipped (non-user type, missing/empty content).
pub fn extract_inbound_message_fields(msg: &SDKMessage) -> Option<InboundMessageFields> {
    let SDKMessage::User { message, uuid } = msg else {
        return None;
    };

    let content = match message {
        Some(UserMessageContent::String(s)) => {
            if s.is_empty() {
                return None;
            }
            s.clone()
        }
        Some(UserMessageContent::Blocks(blocks)) => {
            if blocks.is_empty() {
                return None;
            }
            // Normalize and extract text from blocks
            let normalized = normalize_image_blocks(blocks);
            extract_text_from_blocks(&normalized)
        }
        None => return None,
    };

    Some(InboundMessageFields {
        content,
        uuid: uuid.clone(),
    })
}

// =============================================================================
// IMAGE BLOCK NORMALIZATION
// =============================================================================

/// Normalize image content blocks from bridge clients.
/// iOS/web clients may send `mediaType` (camelCase) instead of `media_type` (snake_case),
/// or omit the field entirely.
pub fn normalize_image_blocks(blocks: &[ContentBlock]) -> Vec<ContentBlock> {
    if !blocks.iter().any(|b| is_malformed_base64_image(b)) {
        return blocks.to_vec();
    }

    blocks
        .iter()
        .map(|block| {
            if !is_malformed_base64_image(block) {
                return block.clone();
            }
            // This is a malformed image block - we need to fix it
            // Extract mediaType or detect from base64
            let media_type = detect_image_format(block);
            ContentBlock::Image {
                media_type: Some(media_type),
                data: get_image_data(block),
            }
        })
        .collect()
}

fn is_malformed_base64_image(block: &ContentBlock) -> bool {
    match block {
        ContentBlock::Image { media_type, .. } => media_type.is_none(),
        _ => false,
    }
}

fn detect_image_format(_block: &ContentBlock) -> String {
    // Try to get mediaType from the raw block
    // In production, would use detectImageFormatFromBase64
    // For now, default to a common format
    "image/png".to_string()
}

fn get_image_data(block: &ContentBlock) -> String {
    match block {
        ContentBlock::Image { data, .. } => data.clone(),
        _ => String::new(),
    }
}

fn extract_text_from_blocks(blocks: &[ContentBlock]) -> String {
    blocks
        .iter()
        .filter_map(|block| {
            if let ContentBlock::Text { text } = block {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// =============================================================================
// ALTERNATIVE: Direct JSON value processing
// =============================================================================

/// Alternative: Process a JSON value directly (for loosely-typed inbound messages).
pub fn extract_inbound_message_fields_from_json(
    msg: &serde_json::Value,
) -> Option<InboundMessageFields> {
    // Check if it's a user message
    let msg_type = msg.get("type")?.as_str()?;
    if msg_type != "user" {
        return None;
    }

    let message = msg.get("message")?;
    let content = if let Some(s) = message.as_str() {
        if s.is_empty() {
            return None;
        }
        s.to_string()
    } else if let Some(arr) = message.as_array() {
        if arr.is_empty() {
            return None;
        }
        // Process content blocks
        let normalized: Vec<ContentBlock> = arr
            .iter()
            .filter_map(|b| serde_json::from_value(b.clone()).ok())
            .collect();
        extract_text_from_blocks(&normalized)
    } else {
        return None;
    };

    let uuid = msg.get("uuid").and_then(|v| v.as_str()).map(String::from);

    Some(InboundMessageFields { content, uuid })
}
