//! Empty usage tracking - translated from emptyUsage.ts

use serde::{Deserialize, Serialize};

/// Token usage for an API call
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    #[serde(rename = "input_tokens")]
    pub input_tokens: i64,
    #[serde(rename = "output_tokens")]
    pub output_tokens: i64,
    #[serde(rename = "cache_creation_input_tokens")]
    pub cache_creation_input_tokens: i64,
    #[serde(rename = "cache_hit_input_tokens")]
    pub cache_hit_input_tokens: i64,
}

impl Usage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_input_tokens(mut self, tokens: i64) -> Self {
        self.input_tokens = tokens;
        self
    }

    pub fn with_output_tokens(mut self, tokens: i64) -> Self {
        self.output_tokens = tokens;
        self
    }

    pub fn total(&self) -> i64 {
        self.input_tokens + self.output_tokens
    }
}

/// Create an empty usage struct (for errors, etc.)
pub fn empty_usage() -> Usage {
    Usage::default()
}

/// Create a usage struct with just input tokens
pub fn input_only_usage(tokens: i64) -> Usage {
    Usage::new().with_input_tokens(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_default() {
        let usage = Usage::default();
        assert_eq!(usage.input_tokens, 0);
        assert_eq!(usage.output_tokens, 0);
        assert_eq!(usage.total(), 0);
    }

    #[test]
    fn test_usage_with_tokens() {
        let usage = Usage::new().with_input_tokens(100).with_output_tokens(50);

        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.total(), 150);
    }
}
