#![allow(dead_code)]

use once_cell::sync::Lazy;
use regex::Regex;

static REGEX_SPECIAL_CHARS: Lazy<Regex> = Lazy::new(|| Regex::new(r"[.*+?^${}()|[\]\\]").unwrap());

pub fn escape_regex(str: &str) -> String {
    REGEX_SPECIAL_CHARS.replace_all(str, r"\$&").to_string()
}

pub fn capitalize(str: &str) -> String {
    let mut chars = str.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub fn plural(n: u32, word: &str) -> String {
    if n == 1 {
        word.to_string()
    } else {
        format!("{}s", word)
    }
}

pub fn plural_with_custom(n: u32, word: &str, plural_word: &str) -> String {
    if n == 1 {
        word.to_string()
    } else {
        plural_word.to_string()
    }
}

pub fn first_line_of(s: &str) -> &str {
    match s.find('\n') {
        None => s,
        Some(idx) => &s[..idx],
    }
}

pub fn count_char(str: &str, c: char) -> usize {
    str.chars().filter(|&x| x == c).count()
}

pub fn is_empty_or_whitespace(s: &str) -> bool {
    s.trim().is_empty()
}

pub fn trim_empty(s: &str) -> &str {
    s.trim()
}

pub fn split_at_any(s: &str, delimiters: &[char]) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    for c in s.chars() {
        if delimiters.contains(&c) {
            if !current.is_empty() {
                result.push(current);
                current = String::new();
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

/// Extract content from XML-style tags.
/// Handles:
/// - Self-closing tags
/// - Tags with attributes
/// - Nested tags of the same type
/// - Multiline content
pub fn extract_tag(html: &str, tag_name: &str) -> Option<String> {
    if html.trim().is_empty() || tag_name.trim().is_empty() {
        return None;
    }

    let escaped_tag = escape_regex(tag_name);

    // Create regex pattern
    let pattern = format!(
        r"<{}(?:\s+[^>]*)?>([\s\S]*?)</{}>",
        escaped_tag, escaped_tag
    );

    let re = Regex::new(&pattern).ok()?;

    let mut depth = 0i32;
    let mut last_index = 0;

    // Create separate patterns for opening and closing tags to handle nesting
    let opening_tag_re = Regex::new(&format!(r"<{}(?:\s+[^>]*)?>", escaped_tag)).ok()?;
    let closing_tag_re = Regex::new(&format!(r"</{}>", escaped_tag)).ok()?;

    for caps in re.captures_iter(html) {
        let content = caps.get(1)?.as_str();
        let start = caps.get(0)?.start();

        // Reset depth counter
        depth = 0;

        // Count opening tags before this match
        for _ in opening_tag_re.find_iter(&html[..start]) {
            depth += 1;
        }

        // Count closing tags before this match
        for _ in closing_tag_re.find_iter(&html[..start]) {
            depth -= 1;
        }

        // Only return content if we're at the correct nesting level
        if depth == 0 && !content.is_empty() {
            return Some(content.to_string());
        }

        last_index = start + caps.get(0)?.len();
    }

    None
}

/// Normalize full-width (zenkaku) digits to half-width digits.
/// Useful for accepting input from Japanese/CJK IMEs.
pub fn normalize_full_width_digits(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if ('０'..='９').contains(&c) {
                char::from_u32(c as u32 - 0xfee0).unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}

/// Normalize full-width (zenkaku) space to half-width space.
/// Useful for accepting input from Japanese/CJK IMEs (U+3000 → U+0020).
pub fn normalize_full_width_space(input: &str) -> String {
    input.replace('\u{3000}', " ")
}

// Keep in-memory accumulation modest to avoid blowing up RSS.
// Overflow beyond this limit is spilled to disk by ShellCommand.
const MAX_STRING_LENGTH: usize = 2_usize.pow(25);

/// Safely joins an array of strings with a delimiter, truncating if the result exceeds maxSize.
pub fn safe_join_lines(lines: &[String], delimiter: &str, max_size: usize) -> String {
    let truncation_marker = "...[truncated]";
    let mut result = String::new();

    for line in lines {
        let delimiter_to_add = if result.is_empty() { "" } else { delimiter };
        let full_addition = format!("{}{}", delimiter_to_add, line);

        if result.len() + full_addition.len() <= max_size {
            result.push_str(&full_addition);
        } else {
            let remaining_space =
                max_size - result.len() - delimiter_to_add.len() - truncation_marker.len();

            if remaining_space > 0 {
                result.push_str(delimiter_to_add);
                result.push_str(&line[..remaining_space]);
                result.push_str(truncation_marker);
            } else {
                result.push_str(truncation_marker);
            }
            return result;
        }
    }
    result
}

/// A string accumulator that safely handles large outputs by truncating from the end
/// when a size limit is exceeded.
pub struct EndTruncatingAccumulator {
    content: String,
    is_truncated: bool,
    total_bytes_received: usize,
    max_size: usize,
}

impl EndTruncatingAccumulator {
    pub fn new(max_size: usize) -> Self {
        Self {
            content: String::new(),
            is_truncated: false,
            total_bytes_received: 0,
            max_size,
        }
    }

    pub fn append(&mut self, data: &str) {
        self.total_bytes_received += data.len();

        if self.is_truncated && self.content.len() >= self.max_size {
            return;
        }

        if self.content.len() + data.len() > self.max_size {
            let remaining_space = self.max_size - self.content.len();
            if remaining_space > 0 {
                self.content.push_str(&data[..remaining_space]);
            }
            self.is_truncated = true;
        } else {
            self.content.push_str(data);
        }
    }

    pub fn to_string(&self) -> String {
        if !self.is_truncated {
            return self.content.clone();
        }

        let truncated_bytes = self.total_bytes_received - self.max_size;
        let truncated_kb = (truncated_bytes / 1024).round() as u32;
        format!(
            "{}\n... [output truncated - {}KB removed]",
            self.content, truncated_kb
        )
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.is_truncated = false;
        self.total_bytes_received = 0;
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn truncated(&self) -> bool {
        self.is_truncated
    }

    pub fn total_bytes(&self) -> usize {
        self.total_bytes_received
    }
}

/// Truncates text to a maximum number of lines, adding an ellipsis if truncated.
pub fn truncate_to_lines(text: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= max_lines {
        return text.to_string();
    }
    let truncated: String = lines[..max_lines].join("\n");
    format!("{}…", truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_regex() {
        assert_eq!(escape_regex("a.b"), r"a\.b");
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("hello"), "Hello");
    }

    #[test]
    fn test_plural() {
        assert_eq!(plural(1, "file"), "file");
        assert_eq!(plural(3, "file"), "files");
    }

    #[test]
    fn test_first_line() {
        assert_eq!(first_line_of("hello\nworld"), "hello");
    }

    #[test]
    fn test_extract_tag_basic() {
        assert_eq!(
            extract_tag("<bash-input>ls -la</bash-input>", "bash-input"),
            Some("ls -la".to_string())
        );
    }

    #[test]
    fn test_extract_tag_not_found() {
        assert_eq!(extract_tag("<other>content</other>", "bash-input"), None);
    }

    #[test]
    fn test_extract_tag_empty() {
        assert_eq!(extract_tag("", "bash-input"), None);
        assert_eq!(extract_tag("<bash-input></bash-input>", "bash-input"), None);
    }

    #[test]
    fn test_extract_tag_multiline() {
        let input = "<bash-input>ls\n-la\ntest</bash-input>";
        assert_eq!(
            extract_tag(input, "bash-input"),
            Some("ls\n-la\ntest".to_string())
        );
    }

    #[test]
    fn test_extract_tag_with_attributes() {
        let input = r#"<bash-input attr="value">content</bash-input>"#;
        assert_eq!(
            extract_tag(input, "bash-input"),
            Some("content".to_string())
        );
    }

    #[test]
    fn test_normalize_full_width_digits() {
        assert_eq!(normalize_full_width_digits("１２３"), "123");
    }

    #[test]
    fn test_normalize_full_width_space() {
        assert_eq!(normalize_full_width_space("hello　world"), "hello world");
    }

    #[test]
    fn test_safe_join_lines() {
        let lines = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(safe_join_lines(&lines, ",", 100), "a,b,c");
    }

    #[test]
    fn test_truncate_to_lines() {
        assert_eq!(truncate_to_lines("a\nb\nc\nd", 2), "a\nb…");
    }
}
