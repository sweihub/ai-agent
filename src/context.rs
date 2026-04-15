// Source: /data/home/swei/claudecode/openclaudecode/src/context.ts
//! Context types.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Context {
    pub session_id: String,
    pub project_id: String,
    pub working_directory: String,
    pub env_vars: HashMap<String, String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Context {
    pub fn new(session_id: String, project_id: String, working_directory: String) -> Self {
        Self {
            session_id,
            project_id,
            working_directory,
            env_vars: std::env::vars().collect(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}
