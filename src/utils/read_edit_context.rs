//! Read and edit context utilities.

use serde::{Deserialize, Serialize};

/// Context for file read operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadContext {
    pub file_path: String,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

/// Context for file edit operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditContext {
    pub file_path: String,
    pub operation: EditOperation,
}

/// Edit operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditOperation {
    Replace {
        old_string: String,
        new_string: String,
    },
    Insert {
        position: InsertPosition,
        content: String,
    },
    Delete {
        start_line: usize,
        end_line: usize,
    },
}

/// Insert position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsertPosition {
    Before(usize),
    After(usize),
    At(usize),
}

impl ReadContext {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
            start_line: None,
            end_line: None,
            offset: None,
            limit: None,
        }
    }

    pub fn with_lines(mut self, start: usize, end: usize) -> Self {
        self.start_line = Some(start);
        self.end_line = Some(end);
        self
    }

    pub fn with_offset(mut self, offset: usize, limit: usize) -> Self {
        self.offset = Some(offset);
        self.limit = Some(limit);
        self
    }
}

impl EditContext {
    pub fn replace(file_path: &str, old: &str, new: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
            operation: EditOperation::Replace {
                old_string: old.to_string(),
                new_string: new.to_string(),
            },
        }
    }
}
