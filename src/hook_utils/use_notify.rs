use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Notification {
    pub key: String,
    pub text: String,
    pub priority: NotificationPriority,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPriority {
    Immediate,
    Normal,
    Low,
}

pub struct NotificationManager {
    notifications: Arc<Mutex<HashMap<String, Notification>>>,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add(&self, notification: Notification) {
        let mut notifications = self.notifications.lock().await;
        notifications.insert(notification.key.clone(), notification);
    }

    pub async fn remove(&self, key: &str) -> Option<Notification> {
        let mut notifications = self.notifications.lock().await;
        notifications.remove(key)
    }

    pub async fn get(&self, key: &str) -> Option<Notification> {
        let notifications = self.notifications.lock().await;
        notifications.get(key).cloned()
    }

    pub async fn clear(&self) {
        let mut notifications = self.notifications.lock().await;
        notifications.clear();
    }

    pub async fn list(&self) -> Vec<Notification> {
        let notifications = self.notifications.lock().await;
        notifications.values().cloned().collect()
    }

    pub async fn len(&self) -> usize {
        let notifications = self.notifications.lock().await;
        notifications.len()
    }

    pub async fn is_empty(&self) -> bool {
        let notifications = self.notifications.lock().await;
        notifications.is_empty()
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_notification(
    key: &str,
    text: &str,
    priority: NotificationPriority,
    timeout_ms: Option<u64>,
) -> Notification {
    Notification {
        key: key.to_string(),
        text: text.to_string(),
        priority,
        timeout_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_notification_manager() {
        let manager = NotificationManager::new();

        let notif = create_notification("test", "Hello", NotificationPriority::Normal, None);
        manager.add(notif).await;

        assert_eq!(manager.len().await, 1);

        let retrieved = manager.get("test").await;
        assert!(retrieved.is_some());
    }
}
