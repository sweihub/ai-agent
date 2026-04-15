use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTextMessage {
    pub id: String,
    pub content: String,
    pub timestamp: i64,
}

impl UserTextMessage {
    pub fn new(content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}
