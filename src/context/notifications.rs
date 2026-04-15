// Source: /data/home/swei/claudecode/openclaudecode/src/context/notifications.tsx
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: i64,
    pub read: bool,
    pub dismissible: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

impl Notification {
    pub fn new(title: &str, message: &str, notification_type: NotificationType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            message: message.to_string(),
            notification_type,
            timestamp: chrono::Utc::now().timestamp_millis(),
            read: false,
            dismissible: true,
        }
    }

    pub fn mark_read(&mut self) {
        self.read = true;
    }
}
