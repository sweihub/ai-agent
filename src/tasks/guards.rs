// Source: ~/claudecode/openclaudecode/src/tasks/LocalShellTask/guards.ts

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::task::TaskStateBase;
use crate::types::ids::AgentId;

/// Bash task kind - UI display variant.
/// 'monitor' shows description instead of command, 'Monitor details' dialog title,
/// distinct status bar pill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BashTaskKind {
    Bash,
    Monitor,
}

/// State for a local shell (bash) task.
/// Extracted from LocalShellTask.tsx so non-React consumers (stopTask.ts via
/// print.rs) don't pull React/ink into the module graph.
pub struct LocalShellTaskState {
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

    // LocalShellTask-specific fields
    /// Keep as 'local_bash' for backward compatibility with persisted session state
    pub r#type: String,
    pub command: String,
    pub result: Option<ShellCommandResult>,
    pub completion_status_sent_in_attachment: bool,
    pub shell_command: Option<Box<dyn ShellCommandTrait>>,
    pub unregister_cleanup: Option<Box<dyn FnOnce()>>,
    pub cleanup_timeout_id: Option<u64>,
    /// Track what we last reported for computing deltas (total lines from TaskOutput)
    pub last_reported_total_lines: usize,
    /// Whether the task has been backgrounded
    /// (false = foreground running, true = backgrounded)
    pub is_backgrounded: Option<bool>,
    /// Agent that spawned this task. Used to kill orphaned bash tasks when the
    /// agent exits (see kill_shell_tasks_for_agent). None = main thread.
    pub agent_id: Option<AgentId>,
    /// UI display variant. 'monitor' shows description instead of command,
    /// 'Monitor details' dialog title, distinct status bar pill.
    pub kind: Option<BashTaskKind>,
}

impl std::fmt::Debug for LocalShellTaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalShellTaskState")
            .field("id", &self.id)
            .field("task_type", &self.task_type)
            .field("status", &self.status)
            .field("description", &self.description)
            .field("tool_use_id", &self.tool_use_id)
            .field("start_time", &self.start_time)
            .field("end_time", &self.end_time)
            .field("total_paused_ms", &self.total_paused_ms)
            .field("output_file", &self.output_file)
            .field("output_offset", &self.output_offset)
            .field("notified", &self.notified)
            .field("r#type", &self.r#type)
            .field("command", &self.command)
            .field("result", &self.result)
            .field(
                "completion_status_sent_in_attachment",
                &self.completion_status_sent_in_attachment,
            )
            .field(
                "shell_command",
                &self
                    .shell_command
                    .as_ref()
                    .map(|_| "<dyn ShellCommandTrait>"),
            )
            .field(
                "unregister_cleanup",
                &self.unregister_cleanup.as_ref().map(|_| "<dyn FnOnce()>"),
            )
            .field("cleanup_timeout_id", &self.cleanup_timeout_id)
            .field("last_reported_total_lines", &self.last_reported_total_lines)
            .field("is_backgrounded", &self.is_backgrounded)
            .field("agent_id", &self.agent_id)
            .field("kind", &self.kind)
            .finish()
    }
}

/// Result of a shell command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellCommandResult {
    pub code: i32,
    pub interrupted: bool,
}

/// Trait for shell command operations (to avoid tying to a specific implementation).
pub trait ShellCommandTrait: Send + Sync {
    fn kill(&self);
    fn cleanup(&self);
}

/// Type guard: check if a task is a LocalShellTask.
pub fn is_local_shell_task(task: &dyn std::any::Any) -> bool {
    task.downcast_ref::<LocalShellTaskState>().is_some()
}

/// Type guard: check if a task value (as a generic reference) is a LocalShellTask.
pub fn is_local_shell_task_from_value(task: &serde_json::Value) -> bool {
    task.get("type").and_then(|v| v.as_str()) == Some("local_bash")
}
