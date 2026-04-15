//! Task types and utilities translated from TypeScript Task.ts

use std::collections::HashMap;

/// Types of tasks that can be created
#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum TaskType {
    local_bash,
    local_agent,
    remote_agent,
    in_process_teammate,
    local_workflow,
    monitor_mcp,
    dream,
}

impl TaskType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskType::local_bash => "local_bash",
            TaskType::local_agent => "local_agent",
            TaskType::remote_agent => "remote_agent",
            TaskType::in_process_teammate => "in_process_teammate",
            TaskType::local_workflow => "local_workflow",
            TaskType::monitor_mcp => "monitor_mcp",
            TaskType::dream => "dream",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "local_bash" => Some(TaskType::local_bash),
            "local_agent" => Some(TaskType::local_agent),
            "remote_agent" => Some(TaskType::remote_agent),
            "in_process_teammate" => Some(TaskType::in_process_teammate),
            "local_workflow" => Some(TaskType::local_workflow),
            "monitor_mcp" => Some(TaskType::monitor_mcp),
            "dream" => Some(TaskType::dream),
            _ => None,
        }
    }
}

/// Status of a task
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(non_camel_case_types)]
pub enum TaskStatus {
    pending,
    running,
    completed,
    failed,
    killed,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::pending => "pending",
            TaskStatus::running => "running",
            TaskStatus::completed => "completed",
            TaskStatus::failed => "failed",
            TaskStatus::killed => "killed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(TaskStatus::pending),
            "running" => Some(TaskStatus::running),
            "completed" => Some(TaskStatus::completed),
            "failed" => Some(TaskStatus::failed),
            "killed" => Some(TaskStatus::killed),
            _ => None,
        }
    }
}

/// True when a task is in a terminal state and will not transition further.
/// Used to guard against injecting messages into dead teammates, evicting
/// finished tasks from AppState, and orphan-cleanup paths.
pub fn is_terminal_task_status(status: &TaskStatus) -> bool {
    matches!(
        status,
        TaskStatus::completed | TaskStatus::failed | TaskStatus::killed
    )
}

/// Handle to a task, including its ID and optional cleanup callback
pub struct TaskHandle {
    pub task_id: String,
    pub cleanup: Option<Box<dyn Fn() + Send>>,
}

impl Clone for TaskHandle {
    fn clone(&self) -> Self {
        // Note: cleanup cannot be cloned, so we set it to None
        Self {
            task_id: self.task_id.clone(),
            cleanup: None,
        }
    }
}

/// Function type for updating application state
pub type SetAppState = Box<dyn Fn(Box<dyn Fn() -> Box<dyn AppState>>) + Send + Sync>;

/// Trait for application state
pub trait AppState: Send + Sync {
    // Basic trait for state management
}

/// Context passed to tasks containing abort controller and state access
pub struct TaskContext {
    pub abort_controller: AbortController,
    pub get_app_state: Box<dyn Fn() -> Box<dyn AppState> + Send + Sync>,
    pub set_app_state: SetAppState,
}

/// Abort controller for cancelling operations
#[derive(Clone)]
pub struct AbortController {
    signal: Option<AbortSignal>,
}

impl AbortController {
    pub fn new() -> Self {
        Self { signal: None }
    }

    pub fn with_signal(signal: AbortSignal) -> Self {
        Self {
            signal: Some(signal),
        }
    }

    pub fn signal(&self) -> Option<&AbortSignal> {
        self.signal.as_ref()
    }

    pub fn abort(&self) {
        if let Some(signal) = &self.signal {
            signal
                .aborted
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    pub fn is_aborted(&self) -> bool {
        self.signal
            .as_ref()
            .map(|s| s.aborted.load(std::sync::atomic::Ordering::SeqCst))
            .unwrap_or(false)
    }
}

impl Default for AbortController {
    fn default() -> Self {
        Self::new()
    }
}

/// Abort signal for cancellation
pub struct AbortSignal {
    aborted: std::sync::atomic::AtomicBool,
}

impl AbortSignal {
    pub fn new() -> Self {
        Self {
            aborted: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn aborted(&self) -> bool {
        self.aborted.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Clone for AbortSignal {
    fn clone(&self) -> Self {
        // AtomicBool doesn't implement Clone, but we can create a new one with the same value
        Self {
            aborted: std::sync::atomic::AtomicBool::new(self.aborted()),
        }
    }
}

impl Default for AbortSignal {
    fn default() -> Self {
        Self::new()
    }
}

/// Base fields shared by all task states
#[derive(Debug, Clone)]
pub struct TaskStateBase {
    pub id: String,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub description: String,
    pub tool_use_id: Option<String>,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub total_paused_ms: Option<u64>,
    pub output_file: String,
    pub output_offset: u64,
    pub notified: bool,
}

/// Input for spawning a local shell task
#[derive(Debug, Clone)]
pub struct LocalShellSpawnInput {
    pub command: String,
    pub description: String,
    pub timeout: Option<u64>,
    pub tool_use_id: Option<String>,
    pub agent_id: Option<String>,
    /// UI display variant: description-as-label, dialog title, status bar pill.
    pub kind: Option<ShellKind>,
}

/// Shell kind for UI display
#[derive(Debug, Clone, PartialEq)]
pub enum ShellKind {
    bash,
    monitor,
}

impl ShellKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ShellKind::bash => "bash",
            ShellKind::monitor => "monitor",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "bash" => Some(ShellKind::bash),
            "monitor" => Some(ShellKind::monitor),
            _ => None,
        }
    }
}

/// Task trait for kill operations
pub trait Task: Send + Sync {
    fn name(&self) -> &str;
    fn task_type(&self) -> TaskType;
    fn kill(
        &self,
        task_id: &str,
        set_app_state: SetAppState,
    ) -> impl std::future::Future<Output = ()> + Send;
}

/// Task ID prefixes for backward compatibility
pub const TASK_ID_PREFIXES: &[(&str, &str)] = &[
    ("local_bash", "b"),
    ("local_agent", "a"),
    ("remote_agent", "r"),
    ("in_process_teammate", "t"),
    ("local_workflow", "w"),
    ("monitor_mcp", "m"),
    ("dream", "d"),
];

/// Get task ID prefix for a task type
pub fn get_task_id_prefix(task_type: &TaskType) -> &'static str {
    TASK_ID_PREFIXES
        .iter()
        .find(|(t, _)| *t == task_type.as_str())
        .map(|(_, p)| *p)
        .unwrap_or("x")
}

/// Case-insensitive-safe alphabet (digits + lowercase) for task IDs.
/// 36^8 ≈ 2.8 trillion combinations, sufficient to resist brute-force symlink attacks.
pub const TASK_ID_ALPHABET: &str = "0123456789abcdefghijklmnopqrstuvwxyz";

/// Generate a unique task ID for a given task type
pub fn generate_task_id(task_type: &TaskType) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let prefix = get_task_id_prefix(task_type);
    let mut rng_seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let mut id = prefix.to_string();
    for i in 0..8 {
        // Simple pseudo-random based on seed
        rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let alphabet_idx = (rng_seed >> (i * 3)) as usize % TASK_ID_ALPHABET.len();
        id.push(TASK_ID_ALPHABET.chars().nth(alphabet_idx).unwrap());
    }
    id
}

/// Get the output file path for a task
pub fn get_task_output_path(task_id: &str) -> String {
    // This would typically use a proper path, using a simple placeholder
    format!("/tmp/task_output_{}.txt", task_id)
}

/// Create a base task state
pub fn create_task_state_base(
    id: String,
    task_type: TaskType,
    description: String,
    tool_use_id: Option<String>,
) -> TaskStateBase {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    TaskStateBase {
        id,
        task_type,
        status: TaskStatus::pending,
        description,
        tool_use_id,
        start_time: now,
        end_time: None,
        total_paused_ms: None,
        output_file: String::new(),
        output_offset: 0,
        notified: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_type_strings() {
        assert_eq!(TaskType::local_bash.as_str(), "local_bash");
        assert_eq!(TaskType::remote_agent.as_str(), "remote_agent");
    }

    #[test]
    fn test_task_status_strings() {
        assert_eq!(TaskStatus::pending.as_str(), "pending");
        assert_eq!(TaskStatus::completed.as_str(), "completed");
    }

    #[test]
    fn test_is_terminal_task_status() {
        assert!(!is_terminal_task_status(&TaskStatus::pending));
        assert!(!is_terminal_task_status(&TaskStatus::running));
        assert!(is_terminal_task_status(&TaskStatus::completed));
        assert!(is_terminal_task_status(&TaskStatus::failed));
        assert!(is_terminal_task_status(&TaskStatus::killed));
    }

    #[test]
    fn test_shell_kind_strings() {
        assert_eq!(ShellKind::bash.as_str(), "bash");
        assert_eq!(ShellKind::monitor.as_str(), "monitor");
    }

    #[test]
    fn test_generate_task_id() {
        let id = generate_task_id(&TaskType::local_bash);
        assert!(id.starts_with('b'));
        assert_eq!(id.len(), 9); // 1 prefix + 8 chars
    }

    #[test]
    fn test_task_id_prefix() {
        assert_eq!(get_task_id_prefix(&TaskType::local_bash), "b");
        assert_eq!(get_task_id_prefix(&TaskType::local_agent), "a");
        assert_eq!(get_task_id_prefix(&TaskType::remote_agent), "r");
        assert_eq!(get_task_id_prefix(&TaskType::in_process_teammate), "t");
        assert_eq!(get_task_id_prefix(&TaskType::local_workflow), "w");
        assert_eq!(get_task_id_prefix(&TaskType::monitor_mcp), "m");
        assert_eq!(get_task_id_prefix(&TaskType::dream), "d");
    }
}
