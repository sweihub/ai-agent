// Source: ~/claudecode/openclaudecode/src/tasks/LocalShellTask/killShellTasks.ts

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::tasks::guards::{LocalShellTaskState, is_local_shell_task_from_value};
use crate::types::ids::AgentId;

/// Type alias for the app state updater function.
type SetAppStateFn = Box<dyn Fn(Box<dyn Fn(&serde_json::Value) -> serde_json::Value>)>;

/// Get the app state (caller-provided).
type GetAppStateFn = Box<dyn Fn() -> serde_json::Value>;

/// Kill a running shell task by ID.
pub fn kill_task(task_id: &str, set_app_state: &SetAppStateFn) {
    let task_id_owned = task_id.to_string();
    set_app_state(Box::new(move |prev: &serde_json::Value| {
        let mut prev = prev.clone();

        let tasks = prev.get("tasks").and_then(|t| t.as_object());
        if tasks.is_none() {
            return prev;
        }

        let tasks = tasks.unwrap();
        let task = tasks.get(task_id_owned.as_str());
        if task.is_none() {
            return prev;
        }

        let task = task.unwrap();

        // Check if the task is a local shell task and is running
        if !is_local_shell_task_from_value(task) {
            return prev;
        }

        let status = task.get("status").and_then(|s| s.as_str()).unwrap_or("");
        if status != "running" {
            return prev;
        }

        // Log the kill request
        log_for_debugging(&format!("LocalShellTask {} kill requested", task_id_owned));

        // Kill and cleanup the shell command (if present)
        // Note: shell_command is a non-serializable field, handled via the state object
        if let Some(shell_cmd) = task.get("shellCommand") {
            if !shell_cmd.is_null() {
                // The actual kill/cleanup would be done by the shell command implementation
                // Here we just mark the task as killed
            }
        }

        // Unregister cleanup and clear timeout
        // These are non-serializable and handled by the caller

        // Update the task state
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let mut updated_task = task.clone();
        if let Some(obj) = updated_task.as_object_mut() {
            obj.insert("status".to_string(), serde_json::json!("killed"));
            obj.insert("notified".to_string(), serde_json::json!(true));
            obj.insert("shellCommand".to_string(), serde_json::json!(null));
            obj.insert("endTime".to_string(), serde_json::json!(now));
        }

        let mut new_tasks = tasks.clone();
        new_tasks.insert(task_id_owned.clone(), updated_task);

        if let Some(obj) = prev.as_object_mut() {
            obj.insert("tasks".to_string(), serde_json::json!(new_tasks));
        }

        prev
    }));

    // Evict task output (async, fire and forget)
    let task_id = task_id.to_string();
    tokio::spawn(async move {
        let _ = evict_task_output(&task_id).await;
    });
}

/// Kill all running bash tasks spawned by a given agent.
/// Called from run_agent.rs finally block so background processes don't outlive
/// the agent that started them (prevents 10-day fake-logs.sh zombies).
pub fn kill_shell_tasks_for_agent(
    agent_id: &AgentId,
    get_app_state: &GetAppStateFn,
    set_app_state: &SetAppStateFn,
) {
    let app_state = get_app_state();
    let tasks = app_state.get("tasks").and_then(|t| t.as_object());

    if let Some(tasks) = tasks {
        for (task_id, task) in tasks {
            if is_local_shell_task_from_value(task) {
                let task_agent_id = task
                    .get("agentId")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let status = task.get("status").and_then(|s| s.as_str()).unwrap_or("");

                if task_agent_id.as_deref() == Some(agent_id.as_str()) && status == "running" {
                    log_for_debugging(&format!(
                        "kill_shell_tasks_for_agent: killing orphaned shell task {} (agent {} exiting)",
                        task_id, agent_id
                    ));
                    kill_task(task_id, set_app_state);
                }
            }
        }
    }

    // Purge any queued notifications addressed to this agent — its query loop
    // has exited and won't drain them. kill_task fires 'killed' notifications
    // asynchronously; drop the ones already queued and any that land later sit
    // harmlessly (no consumer matches a dead agent_id).
    dequeue_all_matching(|cmd: &serde_json::Value| {
        cmd.get("agentId")
            .and_then(|v| v.as_str())
            .map(|s| s == agent_id.as_str())
            .unwrap_or(false)
    });
}

/// Log a debug message.
fn log_for_debugging(msg: &str) {
    // In a real implementation, this would use the debug logging infrastructure
    // For now, use eprintln
    eprintln!("[DEBUG] {}", msg);
}

/// Evict task output from disk.
async fn evict_task_output(_task_id: &str) -> std::io::Result<()> {
    // In a real implementation, this would remove the task's output file
    Ok(())
}

/// Dequeue all messages matching a filter from the message queue.
fn dequeue_all_matching<F>(_filter: F)
where
    F: Fn(&serde_json::Value) -> bool,
{
    // In a real implementation, this would use the message queue manager
    // to remove all queued commands matching the filter
}
