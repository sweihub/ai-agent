//! API response types.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ApiResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Choice {
    pub index: u32,
    pub message: ResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResponseMessage {
    pub role: String,
    pub content: Vec<ResponseContentBlock>,
}

#[derive(Debug, Clone)]
pub enum ResponseContentBlock {
    Text(TextBlock),
    ToolUse(ToolUseBlock),
}

#[derive(Debug, Clone)]
pub struct TextBlock {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ToolUseBlock {
    pub id: String,
    pub name: String,
    pub input: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}
