use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadContext {
    pub task_id: String,
    pub session_id: String,
    pub parent_task_id: Option<String>,
    pub priority: TaskPriority,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl TaskPriority {
    pub fn as_u8(&self) -> u8 {
        match self {
            TaskPriority::Low => 0,
            TaskPriority::Normal => 1,
            TaskPriority::High => 2,
            TaskPriority::Critical => 3,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => TaskPriority::Low,
            1 => TaskPriority::Normal,
            2 => TaskPriority::High,
            _ => TaskPriority::Critical,
        }
    }
}

impl WorkloadContext {
    pub fn new(task_id: String, session_id: String) -> Self {
        Self {
            task_id,
            session_id,
            parent_task_id: None,
            priority: TaskPriority::Normal,
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_parent(mut self, parent_id: String) -> Self {
        self.parent_task_id = Some(parent_id);
        self
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workload_context() {
        let ctx = WorkloadContext::new("task1".to_string(), "session1".to_string());
        assert_eq!(ctx.task_id, "task1");
        assert_eq!(ctx.priority, TaskPriority::Normal);
    }
}
