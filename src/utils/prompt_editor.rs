//! Prompt editor utilities.

use serde::{Deserialize, Serialize};

/// Prompt editor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEditorConfig {
    pub auto_save: bool,
    pub syntax_highlighting: bool,
    pub line_numbers: bool,
    pub word_wrap: bool,
}

impl Default for PromptEditorConfig {
    fn default() -> Self {
        Self {
            auto_save: true,
            syntax_highlighting: true,
            line_numbers: true,
            word_wrap: false,
        }
    }
}

/// A prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    pub content: String,
    pub description: Option<String>,
}

impl PromptTemplate {
    pub fn new(name: &str, content: &str) -> Self {
        Self {
            name: name.to_string(),
            content: content.to_string(),
            description: None,
        }
    }
}
