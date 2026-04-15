use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTextMessage {
    pub id: String,
    pub content: String,
    pub message_type: SystemMessageType,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SystemMessageType {
    Info,
    Warning,
    Error,
    Notice,
}

impl SystemTextMessage {
    pub fn new(content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.to_string(),
            message_type: SystemMessageType::Info,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn with_type(mut self, message_type: SystemMessageType) -> Self {
        self.message_type = message_type;
        self
    }
}
