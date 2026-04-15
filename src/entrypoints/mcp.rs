// Source: /data/home/swei/claudecode/openclaudecode/src/entrypoints/mcp.ts
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub servers: Vec<McpServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<std::collections::HashMap<String, String>>,
}

impl McpConfig {
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
        }
    }

    pub fn add_server(&mut self, server: McpServer) {
        self.servers.push(server);
    }

    pub fn get_server(&self, name: &str) -> Option<&McpServer> {
        self.servers.iter().find(|s| s.name == name)
    }

    pub fn remove_server(&mut self, name: &str) {
        self.servers.retain(|s| s.name != name);
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self::new()
    }
}
