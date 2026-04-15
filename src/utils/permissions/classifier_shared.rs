// Source: ~/claudecode/openclaudecode/src/utils/permissions/classifierShared.ts
#![allow(dead_code)]

//! Shared infrastructure for classifier-based permission systems.
//!
//! This module provides common types, schemas, and utilities used by both:
//! - bash_classifier (semantic Bash command matching)
//! - yolo_classifier (YOLO mode security classification)

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A content block from an API response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        name: String,
        input: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
}

/// Extract tool use block from message content by tool name.
pub fn extract_tool_use_block<'a>(content: &'a [ContentBlock], tool_name: &str) -> Option<&'a ContentBlock> {
    content
        .iter()
        .find(|b| matches!(b, ContentBlock::ToolUse { name, .. } if name == tool_name))
}

/// Parse and validate classifier response from tool use block.
/// Returns None if parsing fails.
pub fn parse_classifier_response<T: for<'de> Deserialize<'de>>(
    tool_use_block: &ContentBlock,
) -> Option<T> {
    if let ContentBlock::ToolUse { input, .. } = tool_use_block {
        serde_json::from_value(input.clone()).ok()
    } else {
        None
    }
}
