// Source: /data/home/swei/claudecode/openclaudecode/src/ink/events/event.ts
//! Event types.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Event {
    Message(MessageEvent),
    ToolUse(ToolUseEvent),
    ToolResult(ToolResultEvent),
    Error(ErrorEvent),
}

#[derive(Debug, Clone)]
pub struct MessageEvent {
    pub id: String,
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ToolUseEvent {
    pub id: String,
    pub name: String,
    pub input: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ToolResultEvent {
    pub tool_use_id: String,
    pub output: String,
    pub is_error: bool,
}

#[derive(Debug, Clone)]
pub struct ErrorEvent {
    pub code: String,
    pub message: String,
}
