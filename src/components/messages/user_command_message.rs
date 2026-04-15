use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCommandMessage {
    pub id: String,
    pub command: String,
    pub args: Vec<String>,
    pub timestamp: i64,
}

impl UserCommandMessage {
    pub fn new(command: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            command: command.to_string(),
            args: Vec::new(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }
}
