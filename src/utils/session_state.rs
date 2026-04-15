//! Session state management.

use serde::{Deserialize, Serialize};

/// Session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub status: SessionStatus,
    pub messages_count: u32,
    pub model: Option<String>,
}

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Paused,
    Completed,
    Error,
}

impl Default for SessionStatus {
    fn default() -> Self {
        SessionStatus::Active
    }
}

impl SessionState {
    pub fn new(id: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();

        Self {
            id,
            created_at: now.clone(),
            updated_at: now,
            status: SessionStatus::Active,
            messages_count: 0,
            model: None,
        }
    }

    pub fn update(&mut self) {
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn increment_messages(&mut self) {
        self.messages_count += 1;
        self.update();
    }

    pub fn set_status(&mut self, status: SessionStatus) {
        self.status = status;
        self.update();
    }

    pub fn set_model(&mut self, model: String) {
        self.model = Some(model);
        self.update();
    }
}
