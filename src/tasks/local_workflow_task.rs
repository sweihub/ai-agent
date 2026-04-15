// Source: ~/claudecode/openclaudecode/src/tasks/LocalWorkflowTask/LocalWorkflowTask.ts

#![allow(dead_code)]

/// State for a Local Workflow task.
#[derive(Debug, Clone)]
pub struct LocalWorkflowTaskState {
    // Inherited from TaskStateBase
    pub id: String,
    pub task_type: String,
    pub status: crate::task::TaskStatus,
    pub description: String,
    pub tool_use_id: Option<String>,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub total_paused_ms: Option<u64>,
    pub output_file: String,
    pub output_offset: u64,
    pub notified: bool,
}

/// Type guard: check if a value is a LocalWorkflowTask.
pub fn is_local_workflow_task(_value: &serde_json::Value) -> bool {
    false
}
