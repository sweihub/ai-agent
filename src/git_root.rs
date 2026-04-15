#![allow(dead_code)]

pub fn find_git_root(path: &str) -> Option<String> {
    let mut current = std::path::Path::new(path);
    loop {
        if current.join(".git").exists() {
            return Some(current.to_string_lossy().to_string());
        }
        match current.parent() {
            Some(p) => current = p,
            None => return None,
        }
    }
}
