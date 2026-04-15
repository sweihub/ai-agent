use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookProgressMessage {
    pub id: String,
    pub hook_event: String,
    pub hook_name: String,
    pub command: String,
    pub progress: f32,
    pub status_message: Option<String>,
    pub timestamp: i64,
}

impl HookProgressMessage {
    pub fn new(hook_event: &str, hook_name: &str, command: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            hook_event: hook_event.to_string(),
            hook_name: hook_name.to_string(),
            command: command.to_string(),
            progress: 0.0,
            status_message: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn with_progress(mut self, progress: f32) -> Self {
        self.progress = progress;
        self
    }

    pub fn with_status_message(mut self, message: String) -> Self {
        self.status_message = Some(message);
        self
    }
}
