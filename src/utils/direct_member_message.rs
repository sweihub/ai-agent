//! Direct team member messaging utilities
//!
//! Parse `@agent-name message` syntax for direct team member messaging.

use serde::{Deserialize, Serialize};

/// Result of parsing a direct member message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDirectMemberMessage {
    pub recipient_name: String,
    pub message: String,
}

/// Result of sending a direct message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "success")]
pub enum DirectMessageResult {
    /// Message sent successfully
    #[serde(rename = "true")]
    Ok {
        /// Name of the recipient
        recipient_name: String,
    },
    /// Failed to send message
    #[serde(rename = "false")]
    Err {
        /// Error type
        error: DirectMessageError,
        /// Name of the intended recipient (if applicable)
        recipient_name: Option<String>,
    },
}

/// Error types for direct messaging
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectMessageError {
    /// No team context available
    NoTeamContext,
    /// Recipient not found
    UnknownRecipient,
}

/// Parse `@agent-name message` syntax for direct team member messaging.
///
/// # Arguments
/// * `input` - The input string to parse
///
/// # Returns
/// Some(ParsedDirectMemberMessage) if valid, None otherwise
pub fn parse_direct_member_message(input: &str) -> Option<ParsedDirectMemberMessage> {
    // Match @agent-name message pattern
    let regex = regex::Regex::new(r"^@([\w-]+)\s+(.+)$").ok()?;
    let caps = regex.captures(input)?;
    
    let recipient_name = caps.get(1)?.as_str().to_string();
    let message = caps.get(2)?.as_str().trim().to_string();
    
    if recipient_name.is_empty() || message.is_empty() {
        return None;
    }
    
    Some(ParsedDirectMemberMessage {
        recipient_name,
        message,
    })
}
