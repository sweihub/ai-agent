use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMessageContext {
    queue: VecDeque<QueuedMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMessage {
    pub id: String,
    pub content: String,
    pub priority: MessagePriority,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for QueuedMessageContext {
    fn default() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

impl QueuedMessageContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enqueue(&mut self, content: String, priority: MessagePriority) {
        let message = QueuedMessage {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            priority,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        let insert_pos = self
            .queue
            .iter()
            .position(|m| m.priority < message.priority)
            .unwrap_or(self.queue.len());

        self.queue.insert(insert_pos, message);
    }

    pub fn dequeue(&mut self) -> Option<QueuedMessage> {
        self.queue.pop_front()
    }

    pub fn peek(&self) -> Option<&QueuedMessage> {
        self.queue.front()
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
