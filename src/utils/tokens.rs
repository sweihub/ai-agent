// Source: /data/home/swei/claudecode/openclaudecode/src/utils/tokens.ts
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(default)]
    pub cache_creation_input_tokens: Option<u32>,
    #[serde(default)]
    pub cache_read_input_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub msg_type: String,
    pub message: InnerMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnerMessage {
    pub content: Vec<ContentBlock>,
    pub usage: Option<TokenUsage>,
    pub id: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "thinking")]
    Thinking { thinking: String },
    #[serde(rename = "redacted_thinking")]
    RedactedThinking { data: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        input: serde_json::Value,
        name: Option<String>,
    },
}

const SYNTHETIC_MODEL: &str = "synthetic";

pub fn get_token_usage(message: &Message) -> Option<&TokenUsage> {
    if message.msg_type != "assistant" {
        return None;
    }

    let usage = message.message.usage.as_ref()?;

    if message.message.model.as_deref() == Some(SYNTHETIC_MODEL) {
        return None;
    }

    if let Some(ContentBlock::Text { text }) = message.message.content.first() {
        if text.contains("SYNTHETIC") {
            return None;
        }
    }

    Some(usage)
}

pub fn get_token_count_from_usage(usage: &TokenUsage) -> u32 {
    let cache_creation = usage.cache_creation_input_tokens.unwrap_or(0);
    let cache_read = usage.cache_read_input_tokens.unwrap_or(0);
    usage.input_tokens + cache_creation + cache_read + usage.output_tokens
}

pub fn token_count_from_last_api_response(messages: &[Message]) -> u32 {
    for message in messages.iter().rev() {
        if let Some(usage) = get_token_usage(message) {
            return get_token_count_from_usage(usage);
        }
    }
    0
}

pub fn get_current_usage(messages: &[Message]) -> Option<TokenUsage> {
    for message in messages.iter().rev() {
        if let Some(usage) = get_token_usage(message) {
            return Some(TokenUsage {
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
                cache_creation_input_tokens: usage.cache_creation_input_tokens,
                cache_read_input_tokens: usage.cache_read_input_tokens,
            });
        }
    }
    None
}

pub fn does_most_recent_assistant_message_exceed_200k(messages: &[Message]) -> bool {
    const THRESHOLD: u32 = 200_000;

    let last_asst = messages.iter().rev().find(|m| m.msg_type == "assistant");
    let last_asst = match last_asst {
        Some(m) => m,
        None => return false,
    };

    match get_token_usage(last_asst) {
        Some(usage) => get_token_count_from_usage(usage) > THRESHOLD,
        None => false,
    }
}

pub fn get_assistant_message_content_length(message: &Message) -> usize {
    let mut content_length = 0;

    for block in &message.message.content {
        match block {
            ContentBlock::Text { text } => content_length += text.len(),
            ContentBlock::Thinking { thinking } => content_length += thinking.len(),
            ContentBlock::RedactedThinking { data } => content_length += data.len(),
            ContentBlock::ToolUse { input, .. } => {
                content_length += serde_json::to_string(input).map(|s| s.len()).unwrap_or(0);
            }
        }
    }

    content_length
}

pub fn token_count_with_estimation(messages: &[Message]) -> u32 {
    for message in messages.iter().rev() {
        if let Some(usage) = get_token_usage(message) {
            let base_count = get_token_count_from_usage(usage);
            return base_count;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_count() {
        let usage = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_creation_input_tokens: Some(20),
            cache_read_input_tokens: Some(30),
        };
        assert_eq!(get_token_count_from_usage(&usage), 200);
    }
}
