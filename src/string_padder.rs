#![allow(dead_code)]

pub fn pad_string(s: &str, width: usize, pad_char: char) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{}{}", s, pad_char.to_string().repeat(width - s.len()))
    }
}

pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
