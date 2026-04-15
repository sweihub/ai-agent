// Source: /data/home/swei/claudecode/openclaudecode/src/utils/path.ts
#![allow(dead_code)]

use std::path::Path;

pub fn expand_path(path: &str) -> String {
    if path.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            return path.replace('~', &home.to_string_lossy());
        }
    }
    path.to_string()
}

pub fn normalize_path(path: &str) -> String {
    Path::new(path)
        .canonicalize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string())
}

pub fn is_absolute(path: &str) -> bool {
    Path::new(path).is_absolute()
}

pub fn join_path(a: &str, b: &str) -> String {
    Path::new(a).join(b).to_string_lossy().to_string()
}

pub fn parent_path(path: &str) -> Option<String> {
    Path::new(path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
}

pub fn file_name(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_absolute() {
        #[cfg(unix)]
        assert!(is_absolute("/usr/bin"));
        #[cfg(windows)]
        assert!(is_absolute("C:\\Windows"));
    }
}
