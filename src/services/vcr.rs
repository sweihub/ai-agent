// Source: /data/home/swei/claudecode/openclaudecode/src/services/vcr.ts
//! VCR (Video Cassette Recorder) - fixture management for testing.
//!
////! Translates vcr.ts from claude code.

use crate::constants::env::{ai, ai_code, system};
use std::collections::HashMap;

pub fn should_use_vcr() -> bool {
    std::env::var(system::NODE_ENV)
        .map(|v| v == "test")
        .unwrap_or(false)
}

pub fn get_vcr_record() -> bool {
    std::env::var(system::VCR_RECORD)
        .map(|v| v == "1")
        .unwrap_or(false)
}

pub fn get_fixtures_root() -> String {
    std::env::var(ai_code::TEST_FIXTURES_ROOT)
        .ok()
        .unwrap_or_else(|| {
            std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        })
}

pub fn is_ci() -> bool {
    std::env::var(system::CI)
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

fn dehydrate_value(s: &str) -> String {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let config_home = std::env::var(ai::CLAUDE_CONFIG_HOME).unwrap_or_else(|_| {
        dirs::config_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    });

    let mut result = s.to_string();

    result = result.replace("num_files=\"[NUM]\"", "num_files=\"[NUM]\"");
    result = result.replace("duration_ms=\"[DURATION]\"", "duration_ms=\"[DURATION]\"");
    result = result.replace("cost_usd=\"[COST]\"", "cost_usd=\"[COST]\"");
    result = result.replace(&config_home, "[CONFIG_HOME]");
    result = result.replace(&cwd, "[CWD]");
    result = result.replace(
        "Available commands: [COMMANDS]",
        "Available commands: [COMMANDS]",
    );

    if result.contains("Files modified by user:") {
        return "Files modified by user: [FILES]".to_string();
    }

    result
}

fn hydrate_value(s: &str) -> String {
    let config_home = std::env::var(ai::CLAUDE_CONFIG_HOME).unwrap_or_else(|_| {
        dirs::config_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    });

    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut result = s.to_string();
    result = result.replace("[NUM]", "1");
    result = result.replace("[DURATION]", "100");
    result = result.replace("[CONFIG_HOME]", &config_home);
    result = result.replace("[CWD]", &cwd);

    result
}

pub fn normalize_path_for_vcr(path: &str) -> String {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let config_home = std::env::var(ai::CLAUDE_CONFIG_HOME).unwrap_or_else(|_| {
        dirs::config_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    });

    path.replace(&cwd, "[CWD]")
        .replace(&config_home, "[CONFIG_HOME]")
}

pub fn denormalize_path(path: &str) -> String {
    let config_home = std::env::var(ai::CLAUDE_CONFIG_HOME).unwrap_or_else(|_| {
        dirs::config_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    });

    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    path.replace("[CONFIG_HOME]", &config_home)
        .replace("[CWD]", &cwd)
}

pub fn hash_input(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_use_vcr() {
        unsafe { std::env::set_var("NODE_ENV", "test") };
        assert!(should_use_vcr());
        unsafe { std::env::remove_var("NODE_ENV") };
    }

    #[test]
    fn test_hash_input() {
        let hash = hash_input("test input");
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_normalize_denormalize_path() {
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let normalized = normalize_path_for_vcr(&format!("{}/src/main.rs", cwd));
        assert!(normalized.contains("[CWD]"));

        let denormalized = denormalize_path(&normalized);
        assert!(denormalized.contains("main.rs"));
    }
}
