use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPromptMessage {
    pub id: String,
    pub prompt: String,
    pub mode: PromptMode,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PromptMode {
    Bash,
    Prompt,
    OrphanedPermission,
    TaskNotification,
}

impl UserPromptMessage {
    pub fn new(prompt: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            prompt: prompt.to_string(),
            mode: PromptMode::Prompt,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn with_mode(mut self, mode: PromptMode) -> Self {
        self.mode = mode;
        self
    }
}
