// Source: /data/home/swei/claudecode/openclaudecode/src/utils/filePersistence/types.ts
pub type TeamMemorySyncStatus = String;

#[derive(Debug, Clone)]
pub struct TeamMemoryEntry {
    pub path: String,
    pub content: String,
}
