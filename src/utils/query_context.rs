//! Query context utilities.

use serde::{Deserialize, Serialize};

/// Context for a query operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryContext {
    pub query: String,
    pub search_path: Option<String>,
    pub file_pattern: Option<String>,
    pub case_sensitive: bool,
    pub regex: bool,
}

impl QueryContext {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            search_path: None,
            file_pattern: None,
            case_sensitive: false,
            regex: false,
        }
    }

    pub fn with_path(mut self, path: &str) -> Self {
        self.search_path = Some(path.to_string());
        self
    }

    pub fn with_pattern(mut self, pattern: &str) -> Self {
        self.file_pattern = Some(pattern.to_string());
        self
    }

    pub fn case_sensitive(mut self) -> Self {
        self.case_sensitive = true;
        self
    }

    pub fn is_regex(mut self) -> Self {
        self.regex = true;
        self
    }
}

/// Query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub matches: Vec<QueryMatch>,
    pub total_count: usize,
}

/// A single query match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMatch {
    pub file_path: String,
    pub line_number: usize,
    pub line_content: String,
    pub match_start: usize,
    pub match_end: usize,
}
