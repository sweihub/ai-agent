use crate::types::message::MessageContent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantTextMessage {
    pub id: String,
    pub content: String,
    pub timestamp: i64,
}

impl AssistantTextMessage {
    pub fn new(content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn from_message_content(content: &MessageContent) -> Option<Self> {
        match content {
            MessageContent::Text(text) => Some(Self::new(text)),
            _ => None,
        }
    }
}
