#![allow(dead_code)]

pub fn escape_regex(text: &str) -> String {
    let special = [
        '\\', '.', '*', '+', '?', '(', ')', '[', ']', '{', '}', '|', '^', '$',
    ];
    let mut result = String::new();
    for ch in text.chars() {
        if special.contains(&ch) {
            result.push('\\');
        }
        result.push(ch);
    }
    result
}

pub fn create_regex(pattern: &str) -> Result<regex::Regex, regex::Error> {
    regex::Regex::new(pattern)
}
