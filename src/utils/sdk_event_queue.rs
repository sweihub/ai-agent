//! SDK event queue utilities.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// An SDK event
#[derive(Debug, Clone)]
pub enum SdkEvent {
    Message(String),
    ToolUse(String),
    ToolResult { id: String, result: String },
    Error(String),
}

/// Event queue for SDK communication
pub struct SdkEventQueue {
    events: Arc<Mutex<VecDeque<SdkEvent>>>,
}

impl SdkEventQueue {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn push(&self, event: SdkEvent) {
        self.events.lock().unwrap().push_back(event);
    }

    pub fn pop(&self) -> Option<SdkEvent> {
        self.events.lock().unwrap().pop_front()
    }

    pub fn len(&self) -> usize {
        self.events.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }
}

impl Default for SdkEventQueue {
    fn default() -> Self {
        Self::new()
    }
}
