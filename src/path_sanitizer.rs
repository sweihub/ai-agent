#![allow(dead_code)]

pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub fn is_safe_path(path: &str) -> bool {
    !path.contains("..") && !path.starts_with('/')
}
