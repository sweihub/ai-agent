//! Syntax highlighting utilities for terminal output

/// ANSI color codes for syntax highlighting
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const ITALIC: &str = "\x1b[3m";
    pub const UNDERLINE: &str = "\x1b[4m";

    // Foreground colors
    pub const BLACK: &str = "\x1b[30m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";

    // Bright foreground colors
    pub const BRIGHT_BLACK: &str = "\x1b[90m";
    pub const BRIGHT_RED: &str = "\x1b[91m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
    pub const BRIGHT_BLUE: &str = "\x1b[94m";
    pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
    pub const BRIGHT_WHITE: &str = "\x1b[97m";

    // Background colors
    pub const BG_BLACK: &str = "\x1b[40m";
    pub const BG_RED: &str = "\x1b[41m";
    pub const BG_GREEN: &str = "\x1b[42m";
    pub const BG_YELLOW: &str = "\x1b[43m";
    pub const BG_BLUE: &str = "\x1b[44m";
    pub const BG_MAGENTA: &str = "\x1b[45m";
    pub const BG_CYAN: &str = "\x1b[46m";
    pub const BG_WHITE: &str = "\x1b[47m";
}

/// Token types for syntax highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Keyword,
    String,
    Number,
    Comment,
    Function,
    Type,
    Variable,
    Operator,
    Punctuation,
    Plain,
}

impl TokenType {
    pub fn color(&self) -> &'static str {
        match self {
            TokenType::Keyword => colors::BRIGHT_MAGENTA,
            TokenType::String => colors::GREEN,
            TokenType::Number => colors::CYAN,
            TokenType::Comment => colors::DIM,
            TokenType::Function => colors::BLUE,
            TokenType::Type => colors::YELLOW,
            TokenType::Variable => colors::WHITE,
            TokenType::Operator => colors::BRIGHT_RED,
            TokenType::Punctuation => colors::WHITE,
            TokenType::Plain => colors::RESET,
        }
    }
}

/// A syntax token with position information
#[derive(Debug, Clone)]
pub struct SyntaxToken {
    pub token_type: TokenType,
    pub text: String,
    pub start: usize,
    pub end: usize,
}

impl SyntaxToken {
    pub fn new(token_type: TokenType, text: impl Into<String>, start: usize) -> Self {
        let text = text.into();
        let end = start + text.len();
        Self {
            token_type,
            text,
            start,
            end,
        }
    }

    /// Render the token with ANSI colors
    pub fn render(&self) -> String {
        format!("{}{}{}", self.token_type.color(), self.text, colors::RESET)
    }
}

/// Simple lexer for syntax highlighting
pub struct Lexer {
    input: String,
    position: usize,
}

impl Lexer {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            position: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<SyntaxToken> {
        let mut tokens = Vec::new();

        while self.position < self.input.len() {
            if let Some(token) = self.next_token() {
                tokens.push(token);
            }
        }

        tokens
    }

    fn next_token(&mut self) -> Option<SyntaxToken> {
        let start = self.position;
        let ch = self.current_char()?;

        // Skip whitespace
        if ch.is_whitespace() {
            let text = self.consume_while(|c| c.is_whitespace());
            return Some(SyntaxToken::new(TokenType::Plain, text, start));
        }

        // Comments
        if self.match_str("//") {
            let text = self.consume_while(|c| c != '\n');
            return Some(SyntaxToken::new(TokenType::Comment, text, start));
        }

        if self.match_str("/*") {
            let text = self.consume_until("*/");
            self.position += 2;
            return Some(SyntaxToken::new(
                TokenType::Comment,
                format!("*/{}", text),
                start,
            ));
        }

        // Strings
        if ch == '"' || ch == '\'' {
            let quote = ch;
            self.position += 1;
            let text = self.consume_while(|c| c != quote);
            if self.current_char() == Some(&quote) {
                self.position += 1;
            }
            return Some(SyntaxToken::new(
                TokenType::String,
                format!("{}{}{}", quote, text, quote),
                start,
            ));
        }

        // Numbers
        if ch.is_ascii_digit() {
            let text = self.consume_while(|c| c.is_ascii_digit() || *c == '.');
            return Some(SyntaxToken::new(TokenType::Number, text, start));
        }

        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            let text = self.consume_while(|c| c.is_alphanumeric() || *c == '_');
            let token_type = if is_keyword(&text) {
                TokenType::Keyword
            } else {
                TokenType::Variable
            };
            return Some(SyntaxToken::new(token_type, text, start));
        }

        // Operators and punctuation
        let text = ch.to_string();
        self.position += 1;
        let token_type = if "+-*/%=<>!&|^~?".contains(ch) {
            TokenType::Operator
        } else {
            TokenType::Punctuation
        };

        Some(SyntaxToken::new(token_type, text, start))
    }

    fn current_char(&self) -> Option<&char> {
        self.input.chars().nth(self.position)
    }

    fn match_str(&self, s: &str) -> bool {
        self.input[self.position..].starts_with(s)
    }

    fn consume_while<F>(&mut self, pred: F) -> String
    where
        F: Fn(&char) -> bool,
    {
        let start = self.position;
        while let Some(ch) = self.current_char() {
            if !pred(ch) {
                break;
            }
            self.position += 1;
        }
        self.input[start..self.position].to_string()
    }

    fn consume_until(&mut self, end: &str) -> String {
        let start = self.position;
        while let Some(ch) = self.current_char() {
            if self.match_str(end) {
                break;
            }
            self.position += 1;
        }
        self.input[start..self.position].to_string()
    }
}

/// Check if a string is a keyword
fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "fn" | "let"
            | "const"
            | "mut"
            | "if"
            | "else"
            | "match"
            | "for"
            | "while"
            | "loop"
            | "break"
            | "continue"
            | "return"
            | "struct"
            | "enum"
            | "impl"
            | "trait"
            | "type"
            | "pub"
            | "mod"
            | "use"
            | "crate"
            | "self"
            | "super"
            | "async"
            | "await"
            | "move"
            | "ref"
            | "where"
            | "unsafe"
            | "extern"
            | "static"
            | "dyn"
            | "true"
            | "false"
    )
}

/// Highlight code with ANSI colors
pub fn highlight_code(code: &str) -> String {
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize();
    tokens.iter().map(|t| t.render()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let mut lexer = Lexer::new("let x = 42;");
        let tokens = lexer.tokenize();

        assert!(tokens.len() > 0);
    }

    #[test]
    fn test_highlight() {
        let result = highlight_code("let x = 42;");
        assert!(!result.is_empty());
    }
}
