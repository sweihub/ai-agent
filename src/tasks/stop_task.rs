// Source: ~/claudecode/openclaudecode/src/tasks/stopTask.ts

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::tasks::guards::is_local_shell_task_from_value;

/// Error type for stop task operations.
#[derive(Debug, Error)]
pub enum StopTaskError {
    #[error("No task found with ID: {0}")]
    NotFound(String),
    #[error("Task {0} is not running (status: {1})")]
    NotRunning(String, String),
    #[error("Unsupported task type: {0}")]
    UnsupportedType(String),
}

/// Context for stop task operations.
pub struct StopTaskContext {
    pub get_app_state: Box<dyn Fn() -> serde_json::Value>,
    pub set_app_state: Box<dyn Fn(Box<dyn Fn(&serde_json::Value) -> serde_json::Value>)>,
}

/// Result of a successful stop task operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopTaskResult {
    pub task_id: String,
    #[serde(rename = "taskType")]
    pub task_type: String,
    pub command: Option<String>,
}

/// Look up a task by ID, validate it is running, kill it, and mark it as notified.
///
/// Throws `StopTaskError` when the task cannot be stopped (not found,
/// not running, or unsupported type). Callers can inspect the error variant
/// to distinguish the failure reason.
pub async fn stop_task(task_id: &str, context: &StopTaskContext) -> Result<StopTaskResult, StopTaskError> {
    let app_state = (context.get_app_state)();

    let task = app_state
        .get("tasks")
        .and_then(|t| t.get("tasks").or_else(|| t.get(task_id)))
        .or_else(|| app_state.get(task_id));

    let task = match task {
        Some(t) => t,
        None => return Err(StopTaskError::NotFound(task_id.to_string())),
    };

    let status = task
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();

    if status != "running" {
        return Err(StopTaskError::NotRunning(task_id.to_string(), status));
    }

    let task_type = task
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let task_impl = get_task_by_type(&task_type);
    if task_impl.is_none() {
        return Err(StopTaskError::UnsupportedType(task_type.clone()));
    }

    // Kill the task
    let task_impl = task_impl.unwrap();
    task_impl.kill(task_id, &context.set_app_state);

    // Bash: suppress the "exit code 137" notification (noise). Agent tasks: don't
    // suppress — the AbortError catch sends a notification carrying
    // extract_partial_result(agent_messages), which is the payload not noise.
    let is_shell_task = is_local_shell_task_from_value(task);
    if is_shell_task {
        let task_id_owned = task_id.to_string();
        let suppressed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let suppressed_clone = suppressed.clone();

        (context.set_app_state)(Box::new(move |prev: &serde_json::Value| {
            let prev_task = prev.get("tasks").and_then(|t| t.get(task_id_owned.as_str()));
            if let Some(prev_task) = prev_task {
                if prev_task.get("notified").and_then(|n| n.as_bool()) == Some(false) {
                    suppressed_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                    let mut new_prev = prev.clone();
                    if let Some(obj) = new_prev.as_object_mut() {
                        if let Some(tasks) = obj.get_mut("tasks") {
                            if let Some(tasks_obj) = tasks.as_object_mut() {
                                if let Some(task) = tasks_obj.get_mut(task_id_owned.as_str()) {
                                    if let Some(task_obj) = task.as_object_mut() {
                                        task_obj.insert("notified".to_string(), serde_json::json!(true));
                                    }
                                }
                            }
                        }
                    }
                    return new_prev;
                }
            }
            prev.clone()
        }));

        // Suppressing the XML notification also suppresses print.rs's parsed
        // task_notification SDK event — emit it directly so SDK consumers see
        // the task close.
        if suppressed.load(std::sync::atomic::Ordering::SeqCst) {
            let tool_use_id = task.get("toolUseId").and_then(|v| v.as_str()).map(|s| s.to_string());
            let summary = task.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
            emit_task_terminated_sdk(task_id, tool_use_id, summary);
        }
    }

    let command = if is_shell_task {
        task.get("command").and_then(|v| v.as_str()).map(|s| s.to_string())
    } else {
        task.get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    };

    Ok(StopTaskResult {
        task_id: task_id.to_string(),
        task_type,
        command,
    })
}

/// Trait representing a task implementation with kill capability.
pub trait Task: Send + Sync {
    fn name(&self) -> &str;
    fn task_type(&self) -> &str;
    fn kill(
        &self,
        task_id: &str,
        set_app_state: &dyn Fn(Box<dyn Fn(&serde_json::Value) -> serde_json::Value>),
    );
}

/// Get a task implementation by type string.
fn get_task_by_type(_task_type: &str) -> Option<Box<dyn Task>> {
    // Task dispatch would be implemented here based on the task type registry
    None
}

/// Emit a task terminated SDK event.
fn emit_task_terminated_sdk(task_id: &str, tool_use_id: Option<String>, summary: Option<String>) {
    // In a real implementation, this would enqueue an SDK event
    let _ = (task_id, tool_use_id, summary);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_task_error_not_found() {
        let error = StopTaskError::NotFound("test-id".to_string());
        assert!(error.to_string().contains("test-id"));
    }

    #[test]
    fn test_stop_task_error_not_running() {
        let error = StopTaskError::NotRunning("test-id".to_string(), "pending".to_string());
        assert!(error.to_string().contains("test-id"));
        assert!(error.to_string().contains("pending"));
    }

    #[test]
    fn test_stop_task_error_unsupported_type() {
        let error = StopTaskError::UnsupportedType("unknown".to_string());
        assert!(error.to_string().contains("unknown"));
    }
}
