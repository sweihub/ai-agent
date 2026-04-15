use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIndex {
    entries: HashMap<String, FileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub modified: i64,
    pub hash: String,
}

impl FileIndex {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, entry: FileEntry) {
        self.entries.insert(entry.path.clone(), entry);
    }

    pub fn get_entry(&self, path: &str) -> Option<&FileEntry> {
        self.entries.get(path)
    }

    pub fn remove_entry(&mut self, path: &str) -> Option<FileEntry> {
        self.entries.remove(path)
    }

    pub fn list_entries(&self) -> Vec<&FileEntry> {
        self.entries.values().collect()
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }
}

impl Default for FileIndex {
    fn default() -> Self {
        Self::new()
    }
}
