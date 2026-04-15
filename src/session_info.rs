//! Session info types.

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub cwd: String,
    pub project_name: Option<String>,
}

impl SessionInfo {
    pub fn new(id: String, cwd: String) -> Self {
        Self {
            id,
            started_at: Utc::now(),
            cwd,
            project_name: None,
        }
    }
}
