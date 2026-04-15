// Source: ~/claudecode/openclaudecode/src/tasks/types.ts

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Union of all concrete task state types.
/// Use this for components that need to work with any task type.
pub enum TaskState {
    LocalShell(crate::tasks::guards::LocalShellTaskState),
    LocalAgent(std::collections::HashMap<String, serde_json::Value>),
    RemoteAgent(std::collections::HashMap<String, serde_json::Value>),
    InProcessTeammate(std::collections::HashMap<String, serde_json::Value>),
    LocalWorkflow(crate::tasks::local_workflow_task::LocalWorkflowTaskState),
    MonitorMcp(crate::tasks::monitor_mcp_task::MonitorMcpTaskState),
    Dream(std::collections::HashMap<String, serde_json::Value>),
}

/// Task types that can appear in the background tasks indicator.
pub enum BackgroundTaskState {
    LocalShell(crate::tasks::guards::LocalShellTaskState),
    LocalAgent(std::collections::HashMap<String, serde_json::Value>),
    RemoteAgent(std::collections::HashMap<String, serde_json::Value>),
    InProcessTeammate(std::collections::HashMap<String, serde_json::Value>),
    LocalWorkflow(crate::tasks::local_workflow_task::LocalWorkflowTaskState),
    MonitorMcp(crate::tasks::monitor_mcp_task::MonitorMcpTaskState),
    Dream(std::collections::HashMap<String, serde_json::Value>),
}

impl BackgroundTaskState {
    /// Returns the task type string.
    pub fn task_type(&self) -> &'static str {
        match self {
            BackgroundTaskState::LocalShell(_) => "local_bash",
            BackgroundTaskState::LocalAgent(_) => "local_agent",
            BackgroundTaskState::RemoteAgent(_) => "remote_agent",
            BackgroundTaskState::InProcessTeammate(_) => "in_process_teammate",
            BackgroundTaskState::LocalWorkflow(_) => "local_workflow",
            BackgroundTaskState::MonitorMcp(_) => "monitor_mcp",
            BackgroundTaskState::Dream(_) => "dream",
        }
    }

    /// Returns the task status.
    pub fn status(&self) -> crate::task::TaskStatus {
        match self {
            BackgroundTaskState::LocalShell(t) => t.status.clone(),
            BackgroundTaskState::LocalAgent(t) => {
                t.get("status")
                    .and_then(|v| v.as_str())
                    .and_then(|s| match s {
                        "pending" => Some(crate::task::TaskStatus::pending),
                        "running" => Some(crate::task::TaskStatus::running),
                        "completed" => Some(crate::task::TaskStatus::completed),
                        "failed" => Some(crate::task::TaskStatus::failed),
                        "killed" => Some(crate::task::TaskStatus::killed),
                        _ => None,
                    })
                    .unwrap_or(crate::task::TaskStatus::pending)
            }
            BackgroundTaskState::RemoteAgent(t) => {
                t.get("status")
                    .and_then(|v| v.as_str())
                    .and_then(|s| match s {
                        "pending" => Some(crate::task::TaskStatus::pending),
                        "running" => Some(crate::task::TaskStatus::running),
                        "completed" => Some(crate::task::TaskStatus::completed),
                        "failed" => Some(crate::task::TaskStatus::failed),
                        "killed" => Some(crate::task::TaskStatus::killed),
                        _ => None,
                    })
                    .unwrap_or(crate::task::TaskStatus::pending)
            }
            BackgroundTaskState::InProcessTeammate(t) => {
                t.get("status")
                    .and_then(|v| v.as_str())
                    .and_then(|s| match s {
                        "pending" => Some(crate::task::TaskStatus::pending),
                        "running" => Some(crate::task::TaskStatus::running),
                        "completed" => Some(crate::task::TaskStatus::completed),
                        "failed" => Some(crate::task::TaskStatus::failed),
                        "killed" => Some(crate::task::TaskStatus::killed),
                        _ => None,
                    })
                    .unwrap_or(crate::task::TaskStatus::pending)
            }
            BackgroundTaskState::LocalWorkflow(t) => t.status.clone(),
            BackgroundTaskState::MonitorMcp(t) => t.status.clone(),
            BackgroundTaskState::Dream(t) => {
                t.get("status")
                    .and_then(|v| v.as_str())
                    .and_then(|s| match s {
                        "pending" => Some(crate::task::TaskStatus::pending),
                        "running" => Some(crate::task::TaskStatus::running),
                        "completed" => Some(crate::task::TaskStatus::completed),
                        "failed" => Some(crate::task::TaskStatus::failed),
                        "killed" => Some(crate::task::TaskStatus::killed),
                        _ => None,
                    })
                    .unwrap_or(crate::task::TaskStatus::pending)
            }
        }
    }
}

/// Check if a task should be shown in the background tasks indicator.
/// A task is considered a background task if:
/// 1. It is running or pending
/// 2. It has been explicitly backgrounded (not a foreground task)
pub fn is_background_task(task: &BackgroundTaskState) -> bool {
    let status = task.status();
    if status != crate::task::TaskStatus::running
        && status != crate::task::TaskStatus::pending
    {
        return false;
    }

    // Foreground tasks (is_backgrounded === false) are not yet "background tasks"
    if let BackgroundTaskState::LocalShell(t) = task {
        if t.is_backgrounded == Some(false) {
            return false;
        }
    }

    true
}
