// Source: /data/home/swei/claudecode/openclaudecode/src/utils/sanitization.ts
//! Sanitization utilities for cleaning user input and file paths.

use std::path::Path;

/// Sanitize a filename by removing dangerous characters
pub fn sanitize_filename(name: &str) -> String {
    let mut result = String::new();

    for c in name.chars() {
        match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => {
                result.push('_');
            }
            '\0' => {
                // Skip null bytes
            }
            _ => {
                result.push(c);
            }
        }
    }

    // Remove leading/trailing whitespace and dots
    result.trim().trim_matches('.').to_string()
}

/// Sanitize a path by removing dangerous components
pub fn sanitize_path(path: &str) -> String {
    path.replace("..", "").replace('\0', "")
}

/// Escape HTML entities
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Unescape HTML entities
pub fn unescape_html(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

/// Escape a string for use in a shell command
pub fn escape_shell_arg(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Truncate a string to a maximum length, adding ellipsis if needed
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
    }
}

/// Normalize whitespace (replace multiple spaces with single space)
pub fn normalize_whitespace(s: &str) -> String {
    let mut result = String::new();
    let mut last_was_space = false;

    for c in s.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(c);
            last_was_space = false;
        }
    }

    result.trim().to_string()
}

/// Check if a string contains only ASCII characters
pub fn is_ascii(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii())
}
