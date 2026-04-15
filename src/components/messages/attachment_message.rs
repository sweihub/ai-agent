use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMessage {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: u64,
    pub content: Option<String>,
    pub timestamp: i64,
}

impl AttachmentMessage {
    pub fn new(filename: &str, mime_type: &str, size: u64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            filename: filename.to_string(),
            mime_type: mime_type.to_string(),
            size,
            content: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }
}
