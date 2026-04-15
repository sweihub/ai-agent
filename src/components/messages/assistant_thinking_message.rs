use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantThinkingMessage {
    pub id: String,
    pub thinking: String,
    pub timestamp: i64,
}

impl AssistantThinkingMessage {
    pub fn new(thinking: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            thinking: thinking.to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}
