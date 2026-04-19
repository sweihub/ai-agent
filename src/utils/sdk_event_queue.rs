//! SDK event queue utilities.
//! Translated from ~/claudecode/openclaudecode/src/utils/sdkEventQueue.ts
//!
//! SDK events are emitted during headless/streaming sessions and drained
//! by the CLI output layer. In TUI mode they are silently dropped to avoid
//! queue buildup.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::bootstrap::state::{get_is_non_interactive_session, get_session_id};

/// Maximum number of SDK events to keep in the queue before dropping oldest.
const MAX_QUEUE_SIZE: usize = 1000;

/// A typed SDK event subtype.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SdkEventType {
    /// A background task has started.
    TaskStarted,
    /// A task is making progress (called periodically).
    TaskProgress,
    /// A task has reached a terminal state.
    TaskNotification,
    /// The session has changed state (idle/running/requires_action).
    SessionStateChanged,
}

/// A single SDK event with all possible fields (matching TS union type).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SdkEvent {
    /// Always "system" for internal SDK events.
    #[serde(rename = "type")]
    pub event_type: String,
    /// The event subtype discriminator.
    pub subtype: SdkEventType,
    /// The task ID this event relates to (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    /// The tool use ID that triggered the task (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    /// Human-readable description of the task (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The type of task (e.g. "local_agent", "local_bash") (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,
    /// The workflow name for workflow-based tasks (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_name: Option<String>,
    /// The prompt for dream tasks (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Usage statistics for task progress (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<SdkEventUsage>,
    /// Last tool name used during task progress (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_tool_name: Option<String>,
    /// Summary of progress (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Workflow progress data (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_progress: Option<Vec<serde_json::Value>>,
    /// For task_notification: terminal status of the task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// For task_notification: path to the task output file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_file: Option<String>,
    /// For session_state_changed: the new session state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

/// Usage statistics for task_progress events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SdkEventUsage {
    pub total_tokens: u64,
    pub tool_uses: u64,
    pub duration_ms: u64,
}

/// A drained SDK event with UUID and session ID appended.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrainedSdkEvent {
    pub uuid: String,
    pub session_id: String,
    #[serde(flatten)]
    pub event: SdkEvent,
}

impl SdkEvent {
    /// Create a new task_started event.
    pub fn task_started(
        task_id: String,
        tool_use_id: Option<String>,
        description: String,
        task_type: Option<String>,
        workflow_name: Option<String>,
        prompt: Option<String>,
    ) -> Self {
        Self {
            event_type: "system".to_string(),
            subtype: SdkEventType::TaskStarted,
            task_id: Some(task_id),
            tool_use_id,
            description: Some(description),
            task_type,
            workflow_name,
            prompt,
            usage: None,
            last_tool_name: None,
            summary: None,
            workflow_progress: None,
            status: None,
            output_file: None,
            state: None,
        }
    }

    /// Create a new task_progress event.
    pub fn task_progress(
        task_id: String,
        tool_use_id: Option<String>,
        description: String,
        usage: SdkEventUsage,
        last_tool_name: Option<String>,
        summary: Option<String>,
        workflow_progress: Option<Vec<serde_json::Value>>,
    ) -> Self {
        Self {
            event_type: "system".to_string(),
            subtype: SdkEventType::TaskProgress,
            task_id: Some(task_id),
            tool_use_id,
            description: Some(description),
            task_type: None,
            workflow_name: None,
            prompt: None,
            usage: Some(usage),
            last_tool_name,
            summary,
            workflow_progress,
            status: None,
            output_file: None,
            state: None,
        }
    }

    /// Create a new task_notification event for a terminal task.
    pub fn task_notification(
        task_id: String,
        tool_use_id: Option<String>,
        status: String,
        output_file: String,
        summary: String,
        usage: Option<SdkEventUsage>,
    ) -> Self {
        Self {
            event_type: "system".to_string(),
            subtype: SdkEventType::TaskNotification,
            task_id: Some(task_id),
            tool_use_id,
            description: None,
            task_type: None,
            workflow_name: None,
            prompt: None,
            usage,
            last_tool_name: None,
            summary: if summary.is_empty() {
                None
            } else {
                Some(summary)
            },
            workflow_progress: None,
            status: Some(status),
            output_file: if output_file.is_empty() {
                None
            } else {
                Some(output_file)
            },
            state: None,
        }
    }

    /// Create a new session_state_changed event.
    pub fn session_state_changed(state: String) -> Self {
        Self {
            event_type: "system".to_string(),
            subtype: SdkEventType::SessionStateChanged,
            task_id: None,
            tool_use_id: None,
            description: None,
            task_type: None,
            workflow_name: None,
            prompt: None,
            usage: None,
            last_tool_name: None,
            summary: None,
            workflow_progress: None,
            status: None,
            output_file: None,
            state: Some(state),
        }
    }
}

/// Thread-safe event queue for SDK communication.
pub struct SdkEventQueue {
    events: Arc<Mutex<VecDeque<SdkEvent>>>,
}

impl SdkEventQueue {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_QUEUE_SIZE))),
        }
    }

    /// Push an event into the queue. Drops oldest if over capacity.
    pub fn push(&self, event: SdkEvent) {
        let mut queue = self.events.lock().unwrap();
        if queue.len() >= MAX_QUEUE_SIZE {
            queue.pop_front();
        }
        queue.push_back(event);
    }

    /// Pop the next event from the queue.
    pub fn pop(&self) -> Option<SdkEvent> {
        self.events.lock().unwrap().pop_front()
    }

    /// Drain all events from the queue (returns all and clears).
    pub fn drain(&self) -> Vec<SdkEvent> {
        self.events.lock().unwrap().drain(..).collect()
    }

    pub fn len(&self) -> usize {
        self.events.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }
}

impl Default for SdkEventQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// The global SDK event queue singleton.
/// Only consumed (drained) in headless/streaming mode.
static GLOBAL_QUEUE: std::sync::LazyLock<SdkEventQueue> = std::sync::LazyLock::new(SdkEventQueue::new);

/// Enqueue an SDK event.
/// In TUI mode (interactive session), events are dropped to avoid queue buildup.
pub fn enqueue_sdk_event(event: SdkEvent) {
    if !get_is_non_interactive_session() {
        return;
    }
    GLOBAL_QUEUE.push(event);
}

/// Drain all pending SDK events and return them with uuid and session_id.
/// Call this from the output layer (print.ts equivalent) to deliver events
/// to SDK consumers.
pub fn drain_sdk_events() -> Vec<DrainedSdkEvent> {
    let events = GLOBAL_QUEUE.drain();
    if events.is_empty() {
        return Vec::new();
    }
    let session_id = get_session_id();
    events
        .into_iter()
        .map(|event| DrainedSdkEvent {
            uuid: Uuid::new_v4().to_string(),
            session_id: session_id.clone(),
            event,
        })
        .collect()
}

/// Emit a `task_started` SDK event.
/// Called by `register_task()` when a new task is registered.
pub fn emit_task_started(
    task_id: &str,
    tool_use_id: Option<String>,
    description: &str,
    task_type: Option<String>,
    workflow_name: Option<String>,
    prompt: Option<String>,
) {
    enqueue_sdk_event(SdkEvent::task_started(
        task_id.to_string(),
        tool_use_id,
        description.to_string(),
        task_type,
        workflow_name,
        prompt,
    ));
}

/// Emit a `task_progress` SDK event.
/// Shared by background agents (per tool_use) and workflows (per flush batch).
pub fn emit_task_progress(params: TaskProgressParams) {
    enqueue_sdk_event(SdkEvent::task_progress(
        params.task_id,
        params.tool_use_id,
        params.description,
        params.usage,
        params.last_tool_name,
        params.summary,
        params.workflow_progress,
    ));
}

/// Parameters for task_progress event emission.
pub struct TaskProgressParams {
    pub task_id: String,
    pub tool_use_id: Option<String>,
    pub description: String,
    pub usage: SdkEventUsage,
    pub last_tool_name: Option<String>,
    pub summary: Option<String>,
    pub workflow_progress: Option<Vec<serde_json::Value>>,
}

/// Emit a `task_notification` SDK event for a task reaching a terminal state.
///
/// This is the closing bookend to `emit_task_started` (always called via
/// `register_task`). Call this from any exit path that sets a task terminal
/// WITHOUT going through enqueuePendingNotification (print.rs parses the XML
/// into the same SDK event, so paths that do both would double-emit).
/// Paths that suppress the XML notification (notified: true pre-set, kill
/// paths, abort branches) must call this directly so SDK consumers
/// (Scuttle's bg-task dot, VS Code subagent panel) see the task close.
pub fn emit_task_terminated_sdk(
    task_id: &str,
    tool_use_id: Option<String>,
    status: &str,
    summary: Option<String>,
    output_file: Option<String>,
    usage: Option<SdkEventUsage>,
) {
    enqueue_sdk_event(SdkEvent::task_notification(
        task_id.to_string(),
        tool_use_id,
        status.to_string(),
        output_file.unwrap_or_default(),
        summary.unwrap_or_default(),
        usage,
    ));
}

/// Emit a `session_state_changed` SDK event.
/// Mirrors `notifySessionStateChanged` — the 'idle' transition fires after
/// heldBackResult flushes and the bg-agent do-while loop exits, serving as
/// the authoritative "turn is over" signal for SDK consumers.
pub fn emit_session_state_changed(state: &str) {
    enqueue_sdk_event(SdkEvent::session_state_changed(state.to_string()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdk_event_task_started() {
        let event = SdkEvent::task_started(
            "task-1".to_string(),
            Some("tool-1".to_string()),
            "Test task".to_string(),
            Some("local_agent".to_string()),
            None,
            None,
        );
        assert_eq!(event.subtype, SdkEventType::TaskStarted);
        assert_eq!(event.task_id, Some("task-1".to_string()));
        assert_eq!(event.event_type, "system");
    }

    #[test]
    fn test_sdk_event_task_notification() {
        let event = SdkEvent::task_notification(
            "task-1".to_string(),
            None,
            "completed".to_string(),
            "/tmp/task_output.txt".to_string(),
            "Task completed successfully".to_string(),
            None,
        );
        assert_eq!(event.subtype, SdkEventType::TaskNotification);
        assert_eq!(event.status, Some("completed".to_string()));
    }

    #[test]
    fn test_sdk_event_session_state_changed() {
        let event = SdkEvent::session_state_changed("idle".to_string());
        assert_eq!(event.subtype, SdkEventType::SessionStateChanged);
        assert_eq!(event.state, Some("idle".to_string()));
    }

    #[test]
    fn test_sdk_event_queue() {
        let queue = SdkEventQueue::new();
        let event = SdkEvent::task_started(
            "task-1".to_string(),
            None,
            "Test".to_string(),
            None,
            None,
            None,
        );
        queue.push(event.clone());
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop(), Some(event));
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_sdk_event_queue_capacity() {
        let queue = SdkEventQueue::new();
        // Push more than MAX_QUEUE_SIZE to test drop-oldest behavior
        for i in 0..1000 {
            queue.push(SdkEvent::task_started(
                format!("task-{i}"),
                None,
                "Test".to_string(),
                None,
                None,
                None,
            ));
        }
        assert_eq!(queue.len(), 1000);
    }

    #[test]
    fn test_sdk_event_usage() {
        let usage = SdkEventUsage {
            total_tokens: 1500,
            tool_uses: 5,
            duration_ms: 3000,
        };
        assert_eq!(usage.total_tokens, 1500);
        assert_eq!(usage.tool_uses, 5);
        assert_eq!(usage.duration_ms, 3000);
    }
}
