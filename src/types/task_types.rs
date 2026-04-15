// Task types module

use serde::{Deserialize, Serialize};

/// Task status enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Task context with lifecycle tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub id: String,
    pub status: TaskStatus,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    #[serde(rename = "updatedAt")]
    pub updated_at: u64,
}

impl TaskContext {
    pub fn new(id: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            id: id.to_string(),
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_running(&self) -> bool {
        self.status == TaskStatus::Running
    }

    pub fn is_completed(&self) -> bool {
        self.status == TaskStatus::Completed
    }
}
