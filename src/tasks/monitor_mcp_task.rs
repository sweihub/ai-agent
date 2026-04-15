// Source: ~/claudecode/openclaudecode/src/tasks/MonitorMcpTask/MonitorMcpTask.ts

#![allow(dead_code)]

/// State for a Monitor MCP task.
#[derive(Debug, Clone)]
pub struct MonitorMcpTaskState {
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

/// Type guard: check if a value is a MonitorMcpTask.
pub fn is_monitor_mcp_task(_value: &serde_json::Value) -> bool {
    false
}
