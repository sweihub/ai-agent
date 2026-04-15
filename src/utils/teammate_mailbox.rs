use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

pub struct TeammateMailbox {
    messages: Arc<RwLock<VecDeque<MailboxMessage>>>,
}

#[derive(Debug, Clone)]
pub struct MailboxMessage {
    pub from: String,
    pub to: String,
    pub content: String,
    pub timestamp: u64,
}

impl TeammateMailbox {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    pub fn send(&self, from: String, to: String, content: String) -> Result<(), String> {
        let msg = MailboxMessage {
            from,
            to,
            content,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let mut queue = self.messages.write().map_err(|e| e.to_string())?;
        queue.push_back(msg);
        Ok(())
    }

    pub fn receive(&self, recipient: &str) -> Result<Option<MailboxMessage>, String> {
        let mut queue = self.messages.write().map_err(|e| e.to_string())?;

        if let Some(pos) = queue.iter().position(|m| m.to == recipient) {
            Ok(Some(queue.remove(pos).unwrap()))
        } else {
            Ok(None)
        }
    }

    pub fn peek(&self, recipient: &str) -> Result<Option<MailboxMessage>, String> {
        let queue = self.messages.read().map_err(|e| e.to_string())?;

        Ok(queue.iter().find(|m| m.to == recipient).cloned())
    }

    pub fn has_messages(&self, recipient: &str) -> Result<bool, String> {
        let queue = self.messages.read().map_err(|e| e.to_string())?;
        Ok(queue.iter().any(|m| m.to == recipient))
    }
}

impl Default for TeammateMailbox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mailbox() {
        let mailbox = TeammateMailbox::new();

        mailbox
            .send("a".to_string(), "b".to_string(), "hello".to_string())
            .unwrap();

        assert!(mailbox.has_messages("b").unwrap());
        let msg = mailbox.receive("b").unwrap().unwrap();
        assert_eq!(msg.content, "hello");
    }
}
