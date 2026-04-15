//! Process info types.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub platform: String,
    pub arch: String,
    pub version: String,
    pub env: HashMap<String, String>,
}

impl Default for ProcessInfo {
    fn default() -> Self {
        Self {
            pid: std::process::id(),
            ppid: None,
            platform: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            version: String::new(),
            env: std::env::vars().collect(),
        }
    }
}
