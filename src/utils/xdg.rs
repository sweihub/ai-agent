// Source: /data/home/swei/claudecode/openclaudecode/src/utils/xdg.ts
#![allow(dead_code)]

use std::env;
use std::path::PathBuf;

pub fn get_xdg_state_home() -> String {
    env::var("XDG_STATE_HOME").ok().unwrap_or_else(|| {
        dirs::home_dir()
            .map(|h| h.join(".local").join("state"))
            .unwrap_or_else(|| PathBuf::from(".local/state"))
            .to_string_lossy()
            .to_string()
    })
}

pub fn get_xdg_cache_home() -> String {
    env::var("XDG_CACHE_HOME").ok().unwrap_or_else(|| {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .to_string_lossy()
            .to_string()
    })
}

pub fn get_xdg_data_home() -> String {
    env::var("XDG_DATA_HOME").ok().unwrap_or_else(|| {
        dirs::data_dir()
            .map(|h| h.join(".local/share"))
            .unwrap_or_else(|| PathBuf::from(".local/share"))
            .to_string_lossy()
            .to_string()
    })
}

pub fn get_user_bin_dir() -> String {
    dirs::home_dir()
        .map(|h| h.join(".local/bin"))
        .unwrap_or_else(|| PathBuf::from(".local/bin"))
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xdg_state_home() {
        let home = get_xdg_state_home();
        assert!(home.contains(".local/state") || home.contains("XDG_STATE_HOME"));
    }

    #[test]
    fn test_xdg_cache_home() {
        let home = get_xdg_cache_home();
        assert!(!home.is_empty());
    }

    #[test]
    fn test_user_bin_dir() {
        let bin = get_user_bin_dir();
        assert!(bin.contains(".local/bin"));
    }
}
