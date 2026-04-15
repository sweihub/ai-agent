//! Message content types.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum MessageContent {
    Text(TextContent),
    Image(ImageContent),
    ToolUse(ToolUseContent),
    ToolResult(ToolResultContent),
}

#[derive(Debug, Clone)]
pub struct TextContent {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ImageContent {
    pub source: ImageSource,
    pub media_type: String,
}

#[derive(Debug, Clone)]
pub enum ImageSource {
    Base64(String),
    Url(String),
}

#[derive(Debug, Clone)]
pub struct ToolUseContent {
    pub id: String,
    pub name: String,
    pub input: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ToolResultContent {
    pub tool_use_id: String,
    pub content: String,
    pub is_error: bool,
}
