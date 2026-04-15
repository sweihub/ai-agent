// Task management module

pub mod guards;
pub mod kill_shell_tasks;
pub mod local_workflow_task;
pub mod monitor_mcp_task;
pub mod pill_label;
pub mod stop_task;
pub mod types;

pub use guards::{is_local_shell_task, is_local_shell_task_from_value, BashTaskKind, LocalShellTaskState};
pub use kill_shell_tasks::{kill_shell_tasks_for_agent, kill_task};
pub use local_workflow_task::{is_local_workflow_task, LocalWorkflowTaskState};
pub use monitor_mcp_task::{is_monitor_mcp_task, MonitorMcpTaskState};
pub use pill_label::{get_pill_label, pill_needs_cta};
pub use stop_task::{stop_task, StopTaskContext, StopTaskError, StopTaskResult};
pub use types::{is_background_task, BackgroundTaskState, TaskState};
