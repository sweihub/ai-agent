//! Task utilities module.

pub mod task_framework;

// Re-exports from task_framework
pub use crate::utils::circular_buffer::CircularBuffer;
pub use task_framework::{
    AppState, MAX_TASK_OUTPUT_BYTES, MAX_TASK_OUTPUT_BYTES_DISPLAY, OUTPUT_FILE_TAG,
    PANEL_GRACE_MS, POLL_INTERVAL_MS, STATUS_TAG, STOPPED_DISPLAY_MS, SUMMARY_TAG, SetAppState,
    TASK_ID_TAG, TASK_NOTIFICATION_TAG, TASK_TYPE_TAG, TOOL_USE_ID_TAG, TaskAttachment, TaskOutput,
    TaskStateBase, TaskStatus, TaskType, append_task_output, apply_task_offsets_and_evictions,
    cleanup_task_output, evict_task_output, evict_terminal_task, flush_task_output,
    format_task_notification, generate_task_attachments, get_running_tasks, get_task_output,
    get_task_output_delta, get_task_output_path, get_task_output_size, init_task_output,
    init_task_output_as_symlink, is_terminal_task_status, poll_tasks, register_task,
};
