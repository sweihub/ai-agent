// Source: /data/home/swei/claudecode/openclaudecode/src/utils/filePersistence/types.ts
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSyncContent {
    pub entries: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSyncData {
    pub user_id: String,
    pub version: u32,
    pub last_modified: String,
    pub checksum: String,
    pub content: UserSyncContent,
}

#[derive(Debug, Clone)]
pub struct SettingsSyncFetchResult {
    pub success: bool,
    pub data: Option<UserSyncData>,
    pub is_empty: Option<bool>,
    pub error: Option<String>,
    pub skip_retry: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct SettingsSyncUploadResult {
    pub success: bool,
    pub checksum: Option<String>,
    pub last_modified: Option<String>,
    pub error: Option<String>,
}

pub const SYNC_KEYS: SyncKeys = SyncKeys {
    user_settings: "~/.ai/settings.json",
    user_memory: "~/.ai/AI.md",
};

pub struct SyncKeys {
    pub user_settings: &'static str,
    pub user_memory: &'static str,
}

impl SyncKeys {
    pub fn project_settings(&self, project_id: &str) -> String {
        format!("projects/{}/.ai/settings.local.json", project_id)
    }

    pub fn project_memory(&self, project_id: &str) -> String {
        format!("projects/{}/CLAUDE.local.md", project_id)
    }
}
