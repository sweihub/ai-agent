//! Task framework utilities.
//! Contains task management, polling, and notification logic.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Standard polling interval for all tasks
pub const POLL_INTERVAL_MS: u64 = 1000;

/// Duration to display killed tasks before eviction
pub const STOPPED_DISPLAY_MS: u64 = 3_000;

/// Grace period for terminal local_agent tasks in the coordinator panel
pub const PANEL_GRACE_MS: u64 = 30_000;

/// Task types
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TaskType {
    LocalBash,
    LocalAgent,
    RemoteAgent,
    InProcessTeammate,
    LocalWorkflow,
    MonitorMcp,
    Dream,
}

impl TaskType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskType::LocalBash => "local_bash",
            TaskType::LocalAgent => "local_agent",
            TaskType::RemoteAgent => "remote_agent",
            TaskType::InProcessTeammate => "in_process_teammate",
            TaskType::LocalWorkflow => "local_workflow",
            TaskType::MonitorMcp => "monitor_mcp",
            TaskType::Dream => "dream",
        }
    }
}

/// Task status
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Killed,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
            TaskStatus::Killed => "killed",
        }
    }
}

/// True when a task is in a terminal state and will not transition further.
pub fn is_terminal_task_status(status: &TaskStatus) -> bool {
    matches!(
        status,
        TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Killed
    )
}

/// Base task state
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    // Optional fields for local agent
    pub retain: Option<bool>,
    pub evict_after: Option<u64>,
    pub messages: Option<String>,
    pub disk_loaded: Option<bool>,
    pub pending_messages: Option<Vec<String>>,
}

/// Attachment type for task status updates
#[derive(Debug, Clone)]
pub struct TaskAttachment {
    pub task_id: String,
    pub tool_use_id: Option<String>,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub description: String,
    pub delta_summary: Option<String>, // New output since last attachment
}

/// Type alias for app state
pub type AppState = HashMap<String, TaskStateBase>;

/// Type alias for set app state function
pub type SetAppState = Arc<dyn Fn(Arc<Mutex<AppState>>) -> Arc<Mutex<AppState>> + Send + Sync>;

/// Update a task's state in AppState.
/// Helper function for task implementations.
/// Generic to allow type-safe updates for specific task types.
pub fn update_task_state<T: TaskStateTrait>(
    task_id: &str,
    set_app_state: &SetAppState,
    updater: impl FnOnce(&T) -> T,
) {
    let app_state = Arc::new(Mutex::new(HashMap::new()));
    let new_state = set_app_state(app_state.clone());

    // Note: In a real implementation, this would update the actual app state
    // This is a simplified version showing the pattern
}

/// Task state trait for type-safe updates
pub trait TaskStateTrait {
    fn as_any(&self) -> &dyn std::any::Any;
    fn clone_box(&self) -> Box<dyn TaskStateTrait>;
}

impl<T: Clone + 'static> TaskStateTrait for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn TaskStateTrait> {
        Box::new(self.clone())
    }
}

/// Register a new task in AppState.
pub fn register_task(task: TaskStateBase, set_app_state: &SetAppState) -> bool {
    let mut is_replacement = false;
    let task_for_emit = task.clone();

    let _app_state = Arc::new(Mutex::new(HashMap::new()));
    let existing_state = set_app_state(_app_state.clone());
    let mut state = existing_state.blocking_lock();

    // Check if task exists (would be replacement)
    is_replacement = state.contains_key(&task_for_emit.id);

    if !is_replacement {
        state.insert(task_for_emit.id.clone(), task_for_emit.clone());
    }

    drop(state);

    // Replacement (resume) — not a new start. Skip to avoid double-emit.
    if is_replacement {
        return false;
    }

    // Emit task_started SDK event
    crate::utils::sdk_event_queue::emit_task_started(
        &task_for_emit.id,
        task_for_emit.tool_use_id.clone(),
        &task_for_emit.description,
        Some(task_for_emit.task_type.as_str().to_string()),
        None, // workflow_name — not in TaskStateBase
        None, // prompt — not in TaskStateBase
    );

    is_replacement
}

/// Eagerly evict a terminal task from AppState.
/// The task must be in a terminal state (completed/failed/killed) with notified=true.
pub fn evict_terminal_task(task_id: &str, set_app_state: &SetAppState) {
    let _ = (task_id, set_app_state);
    // Note: In real implementation, this would:
    // 1. Check if task exists and is terminal
    // 2. Check if task has been notified
    // 3. Check retain/evict_after for grace period
    // 4. Remove task from app state
}

/// Get all running tasks.
pub fn get_running_tasks(state: &AppState) -> Vec<&TaskStateBase> {
    state
        .values()
        .filter(|task| task.status == TaskStatus::Running)
        .collect()
}

/// Generate attachments for tasks with new output or status changes.
/// Called by the framework to create push notifications.
pub async fn generate_task_attachments(
    state: Arc<Mutex<AppState>>,
) -> (
    Vec<TaskAttachment>,
    HashMap<String, u64>, // updated task offsets
    Vec<String>,          // evicted task ids
) {
    let mut attachments = Vec::new();
    let mut updated_task_offsets: HashMap<String, u64> = HashMap::new();
    let mut evicted_task_ids: Vec<String> = Vec::new();

    let tasks = state.lock().await;

    for (task_id, task_state) in tasks.iter() {
        if task_state.notified {
            match task_state.status {
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Killed => {
                    // Evict terminal tasks
                    evicted_task_ids.push(task_id.clone());
                    continue;
                }
                TaskStatus::Pending => {
                    // Keep in map — hasn't run yet
                    continue;
                }
                TaskStatus::Running => {
                    // Fall through to running logic
                }
            }
        }

        if task_state.status == TaskStatus::Running {
            // Get delta output
            // In real implementation: get_task_output_delta(task_state.id, task_state.output_offset)
            // For now, just update offset (simplified)
            updated_task_offsets.insert(task_id.clone(), task_state.output_offset);
        }
    }

    (attachments, updated_task_offsets, evicted_task_ids)
}

/// Apply the outputOffset patches and evictions from generate_task_attachments.
/// Merges patches against fresh state.
pub fn apply_task_offsets_and_evictions(
    set_app_state: &SetAppState,
    updated_task_offsets: HashMap<String, u64>,
    evicted_task_ids: Vec<String>,
) {
    if updated_task_offsets.is_empty() && evicted_task_ids.is_empty() {
        return;
    }

    let _ = (set_app_state, updated_task_offsets, evicted_task_ids);
    // Note: In real implementation, this would update app state atomically
}

/// Poll all running tasks and check for updates.
/// This is the main polling loop called by the framework.
pub async fn poll_tasks(
    get_app_state: impl Fn() -> Arc<Mutex<AppState>>,
    set_app_state: &SetAppState,
) {
    let state = get_app_state();
    let (attachments, updated_task_offsets, evicted_task_ids) =
        generate_task_attachments(state).await;

    apply_task_offsets_and_evictions(set_app_state, updated_task_offsets, evicted_task_ids);

    // Send notifications for completed tasks
    for attachment in attachments {
        enqueue_task_notification(attachment);
    }
}

/// Enqueue a task notification to the message queue.
fn enqueue_task_notification(attachment: TaskAttachment) {
    let status_text = get_status_text(&attachment.status);

    // Note: In real implementation, this would create XML notification
    // using the constants from xml.ts
    let _ = (attachment, status_text);
    // <task-notification>...message...</task-notification>
}

/// Get human-readable status text.
fn get_status_text(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Completed => "completed successfully",
        TaskStatus::Failed => "failed",
        TaskStatus::Killed => "was stopped",
        TaskStatus::Running => "is running",
        TaskStatus::Pending => "is pending",
    }
}

// XML tag constants
pub const TASK_NOTIFICATION_TAG: &str = "task-notification";
pub const TASK_ID_TAG: &str = "task-id";
pub const TOOL_USE_ID_TAG: &str = "tool-use-id";
pub const TASK_TYPE_TAG: &str = "task-type";
pub const OUTPUT_FILE_TAG: &str = "output-file";
pub const STATUS_TAG: &str = "status";
pub const SUMMARY_TAG: &str = "summary";

/// Helper to format task notification XML
pub fn format_task_notification(
    task_id: &str,
    tool_use_id: Option<&str>,
    task_type: &TaskType,
    output_file: &str,
    status: &TaskStatus,
    description: &str,
) -> String {
    let tool_use_id_line = tool_use_id
        .map(|id| format!("\n<{}>{}</{}>", TOOL_USE_ID_TAG, id, TOOL_USE_ID_TAG))
        .unwrap_or_default();

    let status_text = get_status_text(status);

    format!(
        "<{}>\
<{}>{}</{}>{}\
<{}>{}</{}>\
<{}>{}</{}>\
<{}>{}</{}>\
<{}>Task \"{}\" {}</{}>\
</{}>",
        TASK_NOTIFICATION_TAG,
        TASK_ID_TAG,
        task_id,
        TASK_ID_TAG,
        tool_use_id_line,
        TASK_TYPE_TAG,
        task_type.as_str(),
        TASK_TYPE_TAG,
        OUTPUT_FILE_TAG,
        output_file,
        OUTPUT_FILE_TAG,
        STATUS_TAG,
        status.as_str(),
        STATUS_TAG,
        SUMMARY_TAG,
        description,
        status_text,
        SUMMARY_TAG,
        TASK_NOTIFICATION_TAG
    )
}

/// Task output structure
#[derive(Debug, Clone)]
pub struct TaskOutput {
    pub task_id: String,
    pub content: String,
    pub timestamp: i64,
}

/// Maximum task output size in bytes
pub const MAX_TASK_OUTPUT_BYTES: usize = 100_000;

/// Maximum task output size for display
pub const MAX_TASK_OUTPUT_BYTES_DISPLAY: &str = "100KB";

/// Initialize task output
#[allow(dead_code)]
pub fn init_task_output(_task_id: &str) -> std::path::PathBuf {
    // Stub - would create task output file
    std::env::temp_dir().join("task_output.txt")
}

/// Initialize task output as symlink
#[allow(dead_code)]
pub fn init_task_output_as_symlink(
    _task_id: &str,
    _target: &std::path::Path,
) -> std::io::Result<()> {
    // Stub - would create symlink
    Ok(())
}

/// Get task output path
#[allow(dead_code)]
pub fn get_task_output_path(_task_id: &str) -> std::path::PathBuf {
    std::env::temp_dir().join("task_output.txt")
}

/// Get task output size
#[allow(dead_code)]
pub fn get_task_output_size(_task_id: &str) -> usize {
    0
}

/// Get task output
#[allow(dead_code)]
pub fn get_task_output(_task_id: &str) -> Option<TaskOutput> {
    None
}

/// Get task output delta (for streaming)
#[allow(dead_code)]
pub fn get_task_output_delta(_task_id: &str, _from_byte: usize) -> Option<String> {
    None
}

/// Append to task output
#[allow(dead_code)]
pub fn append_task_output(_task_id: &str, _content: &str) -> std::io::Result<usize> {
    Ok(0)
}

/// Cleanup task output
#[allow(dead_code)]
pub fn cleanup_task_output(_task_id: &str) -> std::io::Result<()> {
    Ok(())
}

/// Evict task output (remove old outputs)
#[allow(dead_code)]
pub fn evict_task_output(_task_id: &str) -> std::io::Result<()> {
    Ok(())
}

/// Flush task output (ensure written to disk)
#[allow(dead_code)]
pub fn flush_task_output(_task_id: &str) -> std::io::Result<()> {
    Ok(())
}
