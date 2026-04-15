use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub timestamp: u64,
    pub tags: Vec<String>,
}

pub struct TeamMemory {
    entries: HashMap<String, MemoryEntry>,
}

impl TeamMemory {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn store(&mut self, key: String, value: String, tags: Vec<String>) {
        let entry = MemoryEntry {
            key: key.clone(),
            value,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tags,
        };
        self.entries.insert(key, entry);
    }

    pub fn retrieve(&self, key: &str) -> Option<&MemoryEntry> {
        self.entries.get(key)
    }

    pub fn search_by_tag(&self, tag: &str) -> Vec<&MemoryEntry> {
        self.entries
            .values()
            .filter(|e| e.tags.contains(&tag.to_string()))
            .collect()
    }

    pub fn delete(&mut self, key: &str) -> Option<MemoryEntry> {
        self.entries.remove(key)
    }

    pub fn list_keys(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }
}

impl Default for TeamMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_memory() {
        let mut memory = TeamMemory::new();

        memory.store(
            "project_info".to_string(),
            "AI coding assistant".to_string(),
            vec!["project".to_string()],
        );

        let entry = memory.retrieve("project_info").unwrap();
        assert_eq!(entry.value, "AI coding assistant");
    }
}
