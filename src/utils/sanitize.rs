#![allow(dead_code)]

pub fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 3
}

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

pub fn sanitize_path(path: &str) -> String {
    path.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '/' || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email() {
        assert!(is_valid_email("test@example.com"));
        assert!(!is_valid_email("invalid"));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("file.txt"), "file.txt");
        assert_eq!(sanitize_filename("file:name.txt"), "file_name.txt");
    }
}
