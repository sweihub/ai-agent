//! Query guard utilities for protecting against malicious queries.

use std::collections::HashSet;

/// A guard that validates and sanitizes queries
pub struct QueryGuard {
    blocked_patterns: HashSet<String>,
    max_length: usize,
}

impl QueryGuard {
    pub fn new() -> Self {
        let blocked_patterns = vec![
            "rm -rf /".to_string(),
            "format c:".to_string(),
            "del /f /s /q".to_string(),
        ]
        .into_iter()
        .collect();

        Self {
            blocked_patterns,
            max_length: 10000,
        }
    }

    /// Validate a query
    pub fn validate(&self, query: &str) -> Result<(), QueryGuardError> {
        // Check length
        if query.len() > self.max_length {
            return Err(QueryGuardError::TooLong(query.len()));
        }

        // Check for blocked patterns
        for pattern in &self.blocked_patterns {
            if query.contains(pattern) {
                return Err(QueryGuardError::BlockedPattern(pattern.clone()));
            }
        }

        Ok(())
    }

    /// Sanitize a query
    pub fn sanitize(&self, query: &str) -> String {
        // Remove null bytes
        let sanitized = query.replace('\0', "");

        // Trim whitespace
        sanitized.trim().to_string()
    }
}

impl Default for QueryGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// Query guard errors
#[derive(Debug, Clone)]
pub enum QueryGuardError {
    TooLong(usize),
    BlockedPattern(String),
}

impl std::fmt::Display for QueryGuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryGuardError::TooLong(len) => write!(f, "Query too long: {} characters", len),
            QueryGuardError::BlockedPattern(pattern) => {
                write!(f, "Query contains blocked pattern: {}", pattern)
            }
        }
    }
}
