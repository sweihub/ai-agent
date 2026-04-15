// Source: /data/home/swei/claudecode/openclaudecode/src/utils/mailbox.ts
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mailbox<T> {
    messages: VecDeque<MailboxMessage<T>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxMessage<T> {
    pub id: String,
    pub sender: String,
    pub content: T,
    pub timestamp: i64,
    pub read: bool,
}

impl<T> Mailbox<T> {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }

    pub fn push(&mut self, sender: &str, content: T) {
        let message = MailboxMessage {
            id: uuid::Uuid::new_v4().to_string(),
            sender: sender.to_string(),
            content,
            timestamp: chrono::Utc::now().timestamp_millis(),
            read: false,
        };
        self.messages.push_back(message);
    }

    pub fn pop(&mut self) -> Option<MailboxMessage<T>> {
        self.messages.pop_front()
    }

    pub fn peek(&self) -> Option<&MailboxMessage<T>> {
        self.messages.front()
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn mark_read(&mut self, id: &str) {
        if let Some(msg) = self.messages.iter_mut().find(|m| m.id == id) {
            msg.read = true;
        }
    }

    pub fn unread_count(&self) -> usize {
        self.messages.iter().filter(|m| !m.read).count()
    }
}

impl<T> Default for Mailbox<T> {
    fn default() -> Self {
        Self::new()
    }
}
