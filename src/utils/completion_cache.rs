//! Shell completion cache utilities
//!
//! Translated from openclaudecode/src/utils/completionCache.ts

use crate::constants::env::system;
use std::path::PathBuf;

/// Shell info for completion
#[derive(Debug, Clone)]
pub struct ShellInfo {
    pub name: String,
    pub rc_file: PathBuf,
    pub cache_file: PathBuf,
    pub completion_line: String,
    pub shell_flag: String,
}

/// Detect the current shell
pub fn detect_shell() -> Option<ShellInfo> {
    let shell = std::env::var(system::SHELL).unwrap_or_default();
    let home = std::env::var(system::HOME).ok()?;
    let claude_dir = PathBuf::from(&home).join(".ai");

    if shell.ends_with("/zsh") || shell.ends_with("/zsh.exe") {
        let cache_file = claude_dir.join("completion.zsh");
        let cache_file_str = cache_file.display().to_string();
        return Some(ShellInfo {
            name: "zsh".to_string(),
            rc_file: PathBuf::from(&home).join(".zshrc"),
            cache_file: cache_file.clone(),
            completion_line: format!(
                "[[ -f \"{}\" ]] && source \"{}\"",
                cache_file_str, cache_file_str
            ),
            shell_flag: "zsh".to_string(),
        });
    }

    if shell.ends_with("/bash") || shell.ends_with("/bash.exe") {
        let cache_file = claude_dir.join("completion.bash");
        let cache_file_str = cache_file.display().to_string();
        return Some(ShellInfo {
            name: "bash".to_string(),
            rc_file: PathBuf::from(&home).join(".bashrc"),
            cache_file: cache_file.clone(),
            completion_line: format!(
                "[ -f \"{}\" ] && source \"{}\"",
                cache_file_str, cache_file_str
            ),
            shell_flag: "bash".to_string(),
        });
    }

    if shell.ends_with("/fish") || shell.ends_with("/fish.exe") {
        let xdg_config = std::env::var(system::XDG_CONFIG_HOME)
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(&home).join(".config"));
        let cache_file = claude_dir.join("completion.fish");
        let cache_file_str = cache_file.display().to_string();
        return Some(ShellInfo {
            name: "fish".to_string(),
            rc_file: xdg_config.join("fish").join("config.fish"),
            cache_file: cache_file.clone(),
            completion_line: format!(
                "[ -f \"{}\" ] && source \"{}\"",
                cache_file_str, cache_file_str
            ),
            shell_flag: "fish".to_string(),
        });
    }

    None
}

/// Get the completion cache directory
pub fn get_completion_cache_dir() -> Option<PathBuf> {
    let home = std::env::var(system::HOME).ok()?;
    Some(PathBuf::from(home).join(".ai"))
}
