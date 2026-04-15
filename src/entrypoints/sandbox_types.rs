use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enabled: bool,
    pub allowed_paths: Vec<String>,
    pub denied_paths: Vec<String>,
    pub max_memory_mb: Option<u64>,
    pub max_cpu_percent: Option<u32>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_paths: Vec::new(),
            denied_paths: Vec::new(),
            max_memory_mb: None,
            max_cpu_percent: None,
        }
    }
}

impl SandboxConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn allow_path(&mut self, path: String) {
        self.allowed_paths.push(path);
    }

    pub fn deny_path(&mut self, path: String) {
        self.denied_paths.push(path);
    }
}
