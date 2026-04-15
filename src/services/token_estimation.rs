//! Token estimation for text.
//!
//! Provides token counting similar to claude code's token estimation.

use crate::types::Message;
use serde::{Deserialize, Serialize};

/// Estimated token count with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEstimate {
    pub tokens: usize,
    pub characters: usize,
    pub words: usize,
    pub method: EstimationMethod,
}

/// Method used for estimation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EstimationMethod {
    /// Fast estimation using character ratio
    CharacterRatio,
    /// Word-based estimation
    WordBased,
    /// Exact TikToken estimation (if available)
    TikToken,
}

// ============================================================================
// Translation of claude code's tokenEstimation.ts - strictly line by line
// ============================================================================

/// Rough token count estimation - matches original TypeScript:
/// `export function roughTokenCountEstimation(content: string, bytesPerToken: number = 4): number`
pub fn rough_token_count_estimation(content: &str, bytes_per_token: f64) -> usize {
    (content.len() as f64 / bytes_per_token).round() as usize
}

/// Returns bytes-per-token ratio for a given file extension
/// Matches original TypeScript:
/// `export function bytesPerTokenForFileType(fileExtension: string): number`
/// Dense JSON has many single-character tokens which makes ratio closer to 2
pub fn bytes_per_token_for_file_type(file_extension: &str) -> f64 {
    match file_extension {
        "json" | "jsonl" | "jsonc" => 2.0,
        _ => 4.0,
    }
}

/// Like roughTokenCountEstimation but uses more accurate bytes-per-token ratio
/// when file type is known - matches original TypeScript:
/// `export function roughTokenCountEstimationForFileType(content: string, fileExtension: string): number`
pub fn rough_token_count_estimation_for_file_type(content: &str, file_extension: &str) -> usize {
    rough_token_count_estimation(content, bytes_per_token_for_file_type(file_extension))
}

/// Estimate tokens for a single message - matches original TypeScript:
/// `export function roughTokenCountEstimationForMessage(message: {...}): number`
pub fn rough_token_count_estimation_for_message(message: &Message) -> usize {
    rough_token_count_estimation_for_content(&message.content)
}

/// Estimate tokens for message content (string or array) - matches original TypeScript:
/// `function roughTokenCountEstimationForContent(content: ...): number`
pub fn rough_token_count_estimation_for_content(content: &str) -> usize {
    if content.is_empty() {
        return 0;
    }
    rough_token_count_estimation(content, 4.0)
}

/// Estimate tokens for an array of messages - matches original TypeScript:
/// `export function roughTokenCountEstimationForMessages(messages: readonly {...}[]): number`
pub fn rough_token_count_estimation_for_messages(messages: &[Message]) -> usize {
    messages
        .iter()
        .map(|msg| rough_token_count_estimation_for_message(msg))
        .sum()
}

// ============================================================================
// Legacy estimation functions (kept for backward compatibility)
// ============================================================================

/// Estimate tokens using character ratio method (faster but less accurate)
/// Average ratio is ~4 characters per token for English
pub fn estimate_tokens_characters(text: &str) -> TokenEstimate {
    let characters = text.len();
    let words = text.split_whitespace().count();

    // Use 4:1 character to token ratio as baseline
    // Adjust based on text characteristics
    let ratio = if text.contains("```") {
        // Code blocks have more characters per token
        5.5
    } else if words > 0 {
        let avg_word_len = characters as f64 / words as f64;
        if avg_word_len > 8.0 {
            // Long words = more characters per token
            5.0
        } else if avg_word_len < 3.0 {
            // Short words = fewer characters per token
            3.5
        } else {
            4.0
        }
    } else {
        4.0
    };

    let tokens = (characters as f64 / ratio).ceil() as usize;

    TokenEstimate {
        tokens,
        characters,
        words,
        method: EstimationMethod::CharacterRatio,
    }
}

/// Estimate tokens using word-based method
pub fn estimate_tokens_words(text: &str) -> TokenEstimate {
    let words = text.split_whitespace().count();
    let characters = text.len();

    // Average ~1.3 words per token for English
    let tokens = (words as f64 / 1.3).ceil() as usize;

    TokenEstimate {
        tokens,
        characters,
        words,
        method: EstimationMethod::WordBased,
    }
}

/// Estimate tokens using combined method (best balance of speed and accuracy)
pub fn estimate_tokens(text: &str) -> TokenEstimate {
    let char_estimate = estimate_tokens_characters(text);
    let word_estimate = estimate_tokens_words(text);

    // Use the average of both methods for better accuracy
    let tokens = (char_estimate.tokens + word_estimate.tokens) / 2;

    TokenEstimate {
        tokens,
        characters: char_estimate.characters,
        words: char_estimate.words,
        method: EstimationMethod::CharacterRatio,
    }
}

/// Estimate tokens in messages (handles role/content format)
pub fn estimate_message_tokens<T: MessageContent>(messages: &[T]) -> usize {
    messages
        .iter()
        .map(|m| {
            let content = m.content();
            // Add overhead for role annotation
            let role_overhead = 4;
            estimate_tokens(content).tokens + role_overhead
        })
        .sum()
}

/// Estimate tokens in a conversation string
pub fn estimate_conversation(conversation: &str) -> TokenEstimate {
    // Count turns by looking for common patterns
    let turns = conversation
        .matches("User:")
        .count()
        .max(conversation.matches("Assistant:").count());

    // Each turn has overhead for role prefix
    let turn_overhead = turns * 10;

    let base = estimate_tokens(conversation);
    TokenEstimate {
        tokens: base.tokens + turn_overhead,
        characters: base.characters,
        words: base.words,
        method: base.method,
    }
}

/// Estimate tokens for tool definitions
pub fn estimate_tool_definitions(tools: &[ToolDefinition]) -> usize {
    tools
        .iter()
        .map(|t| {
            let name_tokens = estimate_tokens(&t.name).tokens;
            let desc_tokens = t
                .description
                .as_ref()
                .map(|d| estimate_tokens(d).tokens)
                .unwrap_or(0);
            let params_tokens = estimate_tokens(&t.input_schema).tokens;
            name_tokens + desc_tokens + params_tokens + 20 // overhead
        })
        .sum()
}

/// Simple message content for estimation
pub trait MessageContent {
    fn content(&self) -> &str;
}

impl MessageContent for String {
    fn content(&self) -> &str {
        self.as_str()
    }
}

impl MessageContent for &str {
    fn content(&self) -> &str {
        self
    }
}

/// Message with role
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl MessageContent for ChatMessage {
    fn content(&self) -> &str {
        &self.content
    }
}

/// Tool definition for estimation
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: String,
}

/// Calculate padding needed for context window
/// Returns the amount of extra input tokens that could fit given the output token budget
pub fn calculate_padding(input_tokens: usize, max_tokens: usize, context_limit: usize) -> usize {
    // Calculate how much room is left for input given the output budget
    let available_for_input = context_limit.saturating_sub(max_tokens);
    if input_tokens < available_for_input {
        available_for_input.saturating_sub(input_tokens)
    } else {
        0
    }
}

/// Estimate if content fits in context
pub fn fits_in_context(content_tokens: usize, max_tokens: usize, context_limit: usize) -> bool {
    content_tokens + max_tokens <= context_limit
}

/// Token encoding utilities
pub mod encoding {
    /// Common tokenization patterns
    pub const CHARS_PER_TOKEN_EN: f64 = 4.0;
    pub const CHARS_PER_TOKEN_CODE: f64 = 5.5;
    pub const CHARS_PER_TOKEN_CJK: f64 = 2.0; // Chinese, Japanese, Korean

    /// Detect if text is primarily code
    pub fn is_code(text: &str) -> bool {
        let code_indicators = [
            "```", "function", "class ", "def ", "const ", "let ", "var ", "import ",
        ];
        code_indicators.iter().any(|i| text.contains(i))
    }

    /// Detect if text is primarily CJK
    pub fn is_cjk(text: &str) -> bool {
        text.chars().any(|c| {
            (c >= '\u{4E00}' && c <= '\u{9FFF}') ||  // CJK Unified Ideographs
            (c >= '\u{3040}' && c <= '\u{309F}') ||  // Hiragana
            (c >= '\u{30A0}' && c <= '\u{30FF}') ||  // Katakana
            (c >= '\u{AC00}' && c <= '\u{D7AF}') // Korean
        })
    }

    /// Get appropriate chars per token ratio
    pub fn chars_per_token(text: &str) -> f64 {
        if is_code(text) {
            super::encoding::CHARS_PER_TOKEN_CODE
        } else if is_cjk(text) {
            super::encoding::CHARS_PER_TOKEN_CJK
        } else {
            super::encoding::CHARS_PER_TOKEN_EN
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MessageRole;

    // ============================================================================
    // Tests for the translated TypeScript functions
    // ============================================================================

    #[test]
    fn test_rough_token_count_estimation() {
        // "Hello world" = 11 chars, 11/4 = 2.75 rounds to 3
        assert_eq!(rough_token_count_estimation("Hello world", 4.0), 3);
        // 100 chars / 4 = 25 tokens
        assert_eq!(rough_token_count_estimation(&"a".repeat(100), 4.0), 25);
    }

    #[test]
    fn test_bytes_per_token_for_file_type() {
        assert_eq!(bytes_per_token_for_file_type("json"), 2.0);
        assert_eq!(bytes_per_token_for_file_type("jsonl"), 2.0);
        assert_eq!(bytes_per_token_for_file_type("rs"), 4.0);
        assert_eq!(bytes_per_token_for_file_type("txt"), 4.0);
    }

    #[test]
    fn test_rough_token_count_estimation_for_file_type() {
        // JSON: 100 chars / 2 = 50 tokens
        assert_eq!(
            rough_token_count_estimation_for_file_type(&"a".repeat(100), "json"),
            50
        );
        // Rust: 100 chars / 4 = 25 tokens
        assert_eq!(
            rough_token_count_estimation_for_file_type(&"a".repeat(100), "rs"),
            25
        );
    }

    #[test]
    fn test_rough_token_count_estimation_for_content() {
        assert_eq!(rough_token_count_estimation_for_content(""), 0);
        // "Hello" = 5 chars, 5/4 = 1.25 rounds to 1
        assert_eq!(rough_token_count_estimation_for_content("Hello"), 1);
    }

    #[test]
    fn test_rough_token_count_estimation_for_message() {
        let msg = crate::types::Message {
            role: MessageRole::User,
            content: "Hello world".to_string(),
            ..Default::default()
        };
        // "Hello world" = 11 chars, 11/4 = 2.75 rounds to 3
        assert_eq!(rough_token_count_estimation_for_message(&msg), 3);
    }

    #[test]
    fn test_rough_token_count_estimation_for_messages() {
        let messages = vec![
            crate::types::Message {
                role: MessageRole::User,
                content: "Hello".to_string(),
                ..Default::default()
            },
            crate::types::Message {
                role: MessageRole::Assistant,
                content: "Hi there".to_string(),
                ..Default::default()
            },
        ];
        // "Hello" = 5 chars / 4 = 1.25 -> 1 token
        // "Hi there" = 8 chars / 4 = 2 tokens
        // Total = 3 tokens
        assert_eq!(rough_token_count_estimation_for_messages(&messages), 3);
    }

    // ============================================================================
    // Tests for legacy estimation functions
    // ============================================================================

    #[test]
    fn test_estimate_tokens_characters() {
        let result = estimate_tokens_characters("Hello, world!");
        assert!(result.tokens >= 3);
        assert_eq!(result.characters, 13);
    }

    #[test]
    fn test_estimate_tokens_words() {
        let result = estimate_tokens_words("Hello world this is a test");
        assert!(result.tokens > 0);
        assert_eq!(result.words, 6);
    }

    #[test]
    fn test_estimate_tokens() {
        let result = estimate_tokens("The quick brown fox jumps over the lazy dog");
        assert!(result.tokens > 0);
    }

    #[test]
    fn test_estimate_conversation() {
        let conv = "User: Hello\nAssistant: Hi there!\nUser: How are you?";
        let result = estimate_conversation(conv);
        assert!(result.tokens > 0);
    }

    #[test]
    fn test_estimate_tool_definitions() {
        let tools = vec![ToolDefinition {
            name: "Read".to_string(),
            description: Some("Read a file".to_string()),
            input_schema: r#"{"type":"object","properties":{"path":{"type":"string"}}}"#
                .to_string(),
        }];
        let tokens = estimate_tool_definitions(&tools);
        assert!(tokens > 0);
    }

    #[test]
    fn test_calculate_padding() {
        assert_eq!(calculate_padding(1000, 500, 2000), 500);
        assert_eq!(calculate_padding(1500, 500, 2000), 0);
    }

    #[test]
    fn test_fits_in_context() {
        assert!(fits_in_context(1000, 500, 2000));
        assert!(!fits_in_context(1600, 500, 2000));
    }

    #[test]
    fn test_encoding_chars_per_token() {
        assert_eq!(
            encoding::chars_per_token("Hello world"),
            encoding::CHARS_PER_TOKEN_EN
        );
        assert_eq!(
            encoding::chars_per_token("function test() {}"),
            encoding::CHARS_PER_TOKEN_CODE
        );
    }

    #[test]
    fn test_is_code() {
        assert!(encoding::is_code("function foo() { return 1; }"));
        assert!(!encoding::is_code("Hello world"));
    }

    #[test]
    fn test_is_cjk() {
        assert!(encoding::is_cjk("你好世界"));
        assert!(!encoding::is_cjk("Hello world"));
    }

    #[test]
    fn test_message_content_trait() {
        let msg = ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };
        assert_eq!(msg.content(), "Hello");
    }
}
