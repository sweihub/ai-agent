use regex::Regex;

pub fn count_char_in_string(s: &str, ch: char) -> usize {
    s.chars().filter(|&c| c == ch).count()
}

pub fn is_empty_or_whitespace(s: &str) -> bool {
    s.trim().is_empty()
}

pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    format!("{}...", &s[..max_len])
}

pub fn extract_numbers(s: &str) -> Vec<u64> {
    let re = Regex::new(r"\d+").unwrap();
    re.find_iter(s)
        .filter_map(|m| m.as_str().parse().ok())
        .collect()
}

pub fn slugify(s: &str) -> String {
    let re = Regex::new(r"[^a-z0-9]+").unwrap();
    re.replace_all(&s.to_lowercase(), "-")
        .trim_matches('-')
        .to_string()
}
