// Source: ~/claudecode/openclaudecode/src/utils/mailbox.ts

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::sync::Notify;
use uuid::Uuid;

/// Source of a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageSource {
    User,
    Teammate,
    System,
    Tick,
    Task,
}

/// A message in the mailbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub source: MessageSource,
    pub content: String,
    pub from: Option<String>,
    pub color: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl Message {
    /// Create a new message with a generated UUID and current timestamp.
    pub fn new(source: MessageSource, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source,
            content,
            from: None,
            color: None,
            timestamp: Utc::now(),
        }
    }
}

/// A predicate function for filtering messages.
pub type MessagePredicate = Box<dyn Fn(&Message) -> bool + Send>;

/// A waiter waiting for a message matching its predicate.
struct Waiter {
    predicate: MessagePredicate,
    sender: tokio::sync::oneshot::Sender<Message>,
}

/// A mailbox for sending and receiving messages asynchronously.
pub struct Mailbox {
    queue: VecDeque<Message>,
    waiters: Vec<Waiter>,
    notify: Notify,
    revision: u64,
}

impl Mailbox {
    /// Create a new empty mailbox.
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            waiters: Vec::new(),
            notify: Notify::new(),
            revision: 0,
        }
    }

    /// Get the number of messages in the queue.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if the mailbox is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get the current revision number.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Send a message to the mailbox.
    pub fn send(&mut self, msg: Message) {
        self.revision += 1;

        // Check if any waiter matches this message
        if let Some(idx) = self.waiters.iter().position(|w| (w.predicate)(&msg)) {
            let waiter = self.waiters.remove(idx);
            let _ = waiter.sender.send(msg);
            self.notify.notify_waiters();
            return;
        }

        self.queue.push_back(msg);
        self.notify.notify_waiters();
    }

    /// Poll for a message matching the predicate.
    pub fn poll<F>(&mut self, predicate: F) -> Option<Message>
    where
        F: Fn(&Message) -> bool,
    {
        if let Some(idx) = self.queue.iter().position(|m| predicate(m)) {
            let msg = self.queue.remove(idx).unwrap();
            self.notify.notify_waiters();
            Some(msg)
        } else {
            None
        }
    }

    /// Wait asynchronously for a message matching the predicate.
    pub async fn receive<F>(&mut self, predicate: F) -> Message
    where
        F: Fn(&Message) -> bool + Send + 'static,
    {
        // First check the queue
        if let Some(idx) = self.queue.iter().position(|m| predicate(m)) {
            let msg = self.queue.remove(idx).unwrap();
            self.notify.notify_waiters();
            return msg;
        }

        // Add a waiter
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.waiters.push(Waiter {
            predicate: Box::new(predicate),
            sender,
        });

        // Wait for notification
        self.notify.notified().await;

        // Receiver will get the message from the waiter
        receiver.await.expect("Mailbox receiver cancelled")
    }

    /// Subscribe to changes in the mailbox.
    pub async fn subscribe(&self) {
        self.notify.notified().await;
    }

    /// Notify all waiters and subscribers of a change.
    fn notify(&self) {
        self.notify.notify_waiters();
    }
}

impl Default for Mailbox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mailbox_send_poll() {
        let mut mailbox = Mailbox::new();
        let msg = Message::new(MessageSource::User, "hello".to_string());
        let id = msg.id.clone();

        mailbox.send(msg);
        assert_eq!(mailbox.len(), 1);

        let received = mailbox.poll(|m| m.id == id);
        assert!(received.is_some());
        assert_eq!(received.unwrap().content, "hello");
        assert_eq!(mailbox.len(), 0);
    }

    #[tokio::test]
    async fn test_mailbox_receive() {
        let mut mailbox = Mailbox::new();

        let handle = tokio::spawn(async move {
            mailbox
                .receive(|m| m.source == MessageSource::User)
                .await
        });

        // Give the task time to start waiting
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let msg = Message::new(MessageSource::User, "test".to_string());
        mailbox.send(msg);

        let received = handle.await.unwrap();
        assert_eq!(received.content, "test");
    }
}
