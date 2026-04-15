//! Paste store utilities for clipboard history.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A pasted item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasteItem {
    pub content: String,
    pub timestamp: i64,
    pub source: Option<String>,
}

/// Store for pasted content
pub struct PasteStore {
    items: VecDeque<PasteItem>,
    max_size: usize,
}

impl PasteStore {
    pub fn new(max_size: usize) -> Self {
        Self {
            items: VecDeque::new(),
            max_size,
        }
    }

    pub fn add(&mut self, content: String, source: Option<String>) {
        let item = PasteItem {
            content,
            timestamp: chrono::Utc::now().timestamp(),
            source,
        };

        self.items.push_front(item);

        // Trim to max size
        while self.items.len() > self.max_size {
            self.items.pop_back();
        }
    }

    pub fn get(&self, index: usize) -> Option<&PasteItem> {
        self.items.get(index)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}
