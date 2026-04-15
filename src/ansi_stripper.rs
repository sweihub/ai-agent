#![allow(dead_code)]

pub fn strip_ansi_codes(text: &str) -> String {
    regex::Regex::new(r"\x1b\[[0-9;]*m")
        .unwrap()
        .replace_all(text, "")
        .to_string()
}
