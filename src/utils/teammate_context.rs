use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeammateContext {
    pub teammate_id: String,
    pub task: String,
    pub shared_files: Vec<String>,
    pub allowed_tools: Vec<String>,
}

impl TeammateContext {
    pub fn new(teammate_id: String, task: String) -> Self {
        Self {
            teammate_id,
            task,
            shared_files: Vec::new(),
            allowed_tools: Vec::new(),
        }
    }

    pub fn share_file(&mut self, path: String) {
        if !self.shared_files.contains(&path) {
            self.shared_files.push(path);
        }
    }

    pub fn allow_tool(&mut self, tool: String) {
        if !self.allowed_tools.contains(&tool) {
            self.allowed_tools.push(tool);
        }
    }

    pub fn can_access_file(&self, path: &str) -> bool {
        self.shared_files.is_empty() || self.shared_files.contains(&path.to_string())
    }

    pub fn can_use_tool(&self, tool: &str) -> bool {
        self.allowed_tools.is_empty() || self.allowed_tools.contains(&tool.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teammate_context() {
        let mut ctx = TeammateContext::new("tm1".to_string(), "help".to_string());

        ctx.share_file("src/main.rs".to_string());
        assert!(ctx.can_access_file("src/main.rs"));
        assert!(!ctx.can_access_file("other.txt"));
    }
}
