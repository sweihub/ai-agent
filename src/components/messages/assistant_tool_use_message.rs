use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantToolUseMessage {
    pub id: String,
    pub tool_name: String,
    pub tool_input: HashMap<String, serde_json::Value>,
    pub status: ToolUseStatus,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ToolUseStatus {
    Pending,
    Running,
    Completed,
    Error,
}

impl AssistantToolUseMessage {
    pub fn new(tool_name: &str, tool_input: HashMap<String, serde_json::Value>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            tool_name: tool_name.to_string(),
            tool_input,
            status: ToolUseStatus::Pending,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn with_status(mut self, status: ToolUseStatus) -> Self {
        self.status = status;
        self
    }
}
