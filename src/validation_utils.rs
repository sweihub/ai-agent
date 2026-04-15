use regex::Regex;

pub fn is_valid_email(email: &str) -> bool {
    let email_re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_re.is_match(email)
}

pub fn is_valid_url(url: &str) -> bool {
    let url_re = Regex::new(r"^https?://[^\s]+$").unwrap();
    url_re.is_match(url)
}

pub fn is_valid_json(s: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(s).is_ok()
}

pub fn is_valid_yaml(s: &str) -> bool {
    serde_yaml::from_str::<serde_yaml::Value>(s).is_ok()
}

pub fn is_numeric(s: &str) -> bool {
    s.parse::<f64>().is_ok()
}

pub fn is_alpha(s: &str) -> bool {
    s.chars().all(|c| c.is_alphabetic())
}

pub fn is_alphanumeric(s: &str) -> bool {
    s.chars().all(|c| c.is_alphanumeric())
}

pub fn is_hex_string(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn validate_length(s: &str, min: usize, max: usize) -> bool {
    let len = s.len();
    len >= min && len <= max
}
