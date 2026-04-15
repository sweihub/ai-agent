#![allow(dead_code)]

#[cfg(target_os = "windows")]
use crate::constants::env::ai;

#[cfg(target_os = "windows")]
use std::path::Path;

#[cfg(target_os = "windows")]
pub fn set_shell_if_windows() {
    if let Ok(git_bash_path) = find_git_bash_path() {
        std::env::set_var("SHELL", git_bash_path);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn set_shell_if_windows() {}

#[cfg(target_os = "windows")]
pub fn find_git_bash_path() -> Result<String, String> {
    if let Ok(env_path) = std::env::var(ai::CODE_GIT_BASH_PATH) {
        if Path::new(&env_path).exists() {
            return Ok(env_path);
        }
        return Err(format!("{} not found: {}", ai::CODE_GIT_BASH_PATH, env_path));
    }

    // Check common Git installation locations
    let default_locations = vec![
        "C:\\Program Files\\Git\\bin\\bash.exe",
        "C:\\Program Files (x86)\\Git\\bin\\bash.exe",
    ];

    for location in default_locations {
        if Path::new(location).exists() {
            return Ok(location.to_string());
        }
    }

    Err("Git Bash not found".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn find_git_bash_path() -> Result<String, String> {
    Ok("/bin/bash".to_string())
}

pub fn windows_path_to_posix_path(windows_path: &str) -> String {
    // Handle UNC paths: \\server\share -> //server/share
    if windows_path.starts_with("\\\\") {
        return windows_path.replace('\\', "/");
    }

    // Handle drive letter paths: C:\Users\foo -> /c/Users/foo
    if let Some(letter) = windows_path.chars().next() {
        if letter.is_ascii_alphabetic() && windows_path.len() >= 3 {
            let third = windows_path.chars().nth(2);
            if third == Some(':') || third == Some('\\') || third == Some('/') {
                let drive = letter.to_lowercase().next().unwrap();
                let rest = &windows_path[3..];
                return format!("/{}{}", drive, rest.replace('\\', "/"));
            }
        }
    }

    windows_path.replace('\\', "/")
}

pub fn posix_path_to_windows_path(posix_path: &str) -> String {
    // Handle UNC paths: //server/share -> \\server\share
    if posix_path.starts_with("//") {
        return posix_path.replace('/', "\\");
    }

    // Handle /c/... format
    if posix_path.starts_with('/') && posix_path.len() >= 2 {
        let second = posix_path.chars().nth(1);
        if let Some(letter) = second {
            if letter.is_ascii_alphabetic() {
                let drive = letter.to_uppercase().next().unwrap();
                let rest = &posix_path[2..];
                return format!("{}:{}", drive, rest.replace('/', "\\"));
            }
        }
    }

    posix_path.replace('/', "\\")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_to_posix() {
        assert_eq!(
            windows_path_to_posix_path("C:\\Users\\test"),
            "/c/Users/test"
        );
        assert_eq!(
            windows_path_to_posix_path("\\\\server\\share"),
            "//server/share"
        );
    }

    #[test]
    fn test_posix_to_windows() {
        assert_eq!(
            posix_path_to_windows_path("/c/Users/test"),
            "C:\\Users\\test"
        );
    }
}
