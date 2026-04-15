// Source: /data/home/swei/claudecode/openclaudecode/src/utils/teammate.ts
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Teammate {
    pub id: String,
    pub name: String,
    pub model: String,
    pub status: TeammateStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TeammateStatus {
    Starting,
    Running,
    Stopped,
    Error,
}

impl Teammate {
    pub fn new(id: String, name: String, model: String) -> Self {
        Self {
            id,
            name,
            model,
            status: TeammateStatus::Starting,
        }
    }

    pub fn is_running(&self) -> bool {
        self.status == TeammateStatus::Running
    }

    pub fn start(&mut self) {
        self.status = TeammateStatus::Running;
    }

    pub fn stop(&mut self) {
        self.status = TeammateStatus::Stopped;
    }

    pub fn error(&mut self) {
        self.status = TeammateStatus::Error;
    }
}

pub struct TeammateManager {
    teammates: Vec<Teammate>,
}

impl TeammateManager {
    pub fn new() -> Self {
        Self {
            teammates: Vec::new(),
        }
    }

    pub fn add(&mut self, teammate: Teammate) {
        self.teammates.push(teammate);
    }

    pub fn remove(&mut self, id: &str) -> Option<Teammate> {
        if let Some(pos) = self.teammates.iter().position(|t| t.id == id) {
            Some(self.teammates.remove(pos))
        } else {
            None
        }
    }

    pub fn get(&self, id: &str) -> Option<&Teammate> {
        self.teammates.iter().find(|t| t.id == id)
    }

    pub fn list_running(&self) -> Vec<&Teammate> {
        self.teammates.iter().filter(|t| t.is_running()).collect()
    }
}

impl Default for TeammateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teammate() {
        let mut teammate =
            Teammate::new("1".to_string(), "helper".to_string(), "claude".to_string());
        assert_eq!(teammate.status, TeammateStatus::Starting);

        teammate.start();
        assert!(teammate.is_running());
    }
}
