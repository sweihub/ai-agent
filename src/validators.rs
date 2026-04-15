#![allow(dead_code)]

pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

pub fn validate_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}
