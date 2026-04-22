// Source: ~/claudecode/openclaudecode/src/utils/hooks/AsyncHookRegistry.ts
#![allow(dead_code)]
#![allow(static_mut_refs)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{Duration, interval};

/// Represents a hook event type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookEvent {
    SessionStart,
    Setup,
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    PermissionDenied,
    Notification,
    UserPromptSubmit,
    SessionEnd,
    Stop,
    StopFailure,
    SubagentStart,
    SubagentStop,
    PreCompact,
    PostCompact,
    PermissionRequest,
    TeammateIdle,
    TaskCreated,
    TaskCompleted,
    Elicitation,
    ElicitationResult,
    ConfigChange,
    InstructionsLoaded,
    WorktreeCreate,
    WorktreeRemove,
    CwdChanged,
    FileChanged,
    StatusLine,
    FileSuggestion,
    Custom(String),
}

impl HookEvent {
    pub fn as_str(&self) -> &str {
        match self {
            HookEvent::SessionStart => "SessionStart",
            HookEvent::Setup => "Setup",
            HookEvent::PreToolUse => "PreToolUse",
            HookEvent::PostToolUse => "PostToolUse",
            HookEvent::PostToolUseFailure => "PostToolUseFailure",
            HookEvent::PermissionDenied => "PermissionDenied",
            HookEvent::Notification => "Notification",
            HookEvent::UserPromptSubmit => "UserPromptSubmit",
            HookEvent::SessionEnd => "SessionEnd",
            HookEvent::Stop => "Stop",
            HookEvent::StopFailure => "StopFailure",
            HookEvent::SubagentStart => "SubagentStart",
            HookEvent::SubagentStop => "SubagentStop",
            HookEvent::PreCompact => "PreCompact",
            HookEvent::PostCompact => "PostCompact",
            HookEvent::PermissionRequest => "PermissionRequest",
            HookEvent::TeammateIdle => "TeammateIdle",
            HookEvent::TaskCreated => "TaskCreated",
            HookEvent::TaskCompleted => "TaskCompleted",
            HookEvent::Elicitation => "Elicitation",
            HookEvent::ElicitationResult => "ElicitationResult",
            HookEvent::ConfigChange => "ConfigChange",
            HookEvent::InstructionsLoaded => "InstructionsLoaded",
            HookEvent::WorktreeCreate => "WorktreeCreate",
            HookEvent::WorktreeRemove => "WorktreeRemove",
            HookEvent::CwdChanged => "CwdChanged",
            HookEvent::FileChanged => "FileChanged",
            HookEvent::StatusLine => "StatusLine",
            HookEvent::FileSuggestion => "FileSuggestion",
            HookEvent::Custom(s) => s,
        }
    }
}

/// JSON output from an async hook
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsyncHookJsonOutput {
    /// Timeout in seconds (0 means default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub async_timeout: Option<u64>,
}

/// JSON output from a sync hook
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncHookJsonOutput {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Output from a shell command task
#[derive(Clone)]
pub struct TaskOutput {
    stdout: Arc<Mutex<String>>,
    stderr: Arc<Mutex<String>>,
}

impl TaskOutput {
    pub fn new() -> Self {
        Self {
            stdout: Arc::new(Mutex::new(String::new())),
            stderr: Arc::new(Mutex::new(String::new())),
        }
    }

    pub async fn get_stdout(&self) -> String {
        self.stdout.lock().unwrap().clone()
    }

    pub fn get_stderr(&self) -> String {
        self.stderr.lock().unwrap().clone()
    }

    pub fn append_stdout(&self, data: &str) {
        self.stdout.lock().unwrap().push_str(data);
    }

    pub fn append_stderr(&self, data: &str) {
        self.stderr.lock().unwrap().push_str(data);
    }
}

/// A shell command being executed
pub struct ShellCommand {
    pub status: ShellCommandStatus,
    pub task_output: TaskOutput,
    pub pid: Option<u32>,
}

impl ShellCommand {
    pub fn cleanup(&self) {
        if let Some(pid) = self.pid {
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
        }
    }

    pub fn kill(&mut self) {
        self.status = ShellCommandStatus::Killed;
        self.cleanup();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShellCommandStatus {
    Running,
    Completed,
    Killed,
}

/// A pending async hook in the registry
pub struct PendingAsyncHook {
    pub process_id: String,
    pub hook_id: String,
    pub hook_name: String,
    pub hook_event: HookEvent,
    pub tool_name: Option<String>,
    pub plugin_id: Option<String>,
    pub start_time: std::time::SystemTime,
    pub timeout_ms: u64,
    pub command: String,
    pub response_attachment_sent: bool,
    pub shell_command: Option<ShellCommand>,
    pub progress_task_id: Option<u64>, // Simple ID for tracking (simplified)
}

/// Global registry state for pending async hooks
struct AsyncHookRegistryState {
    pending_hooks: HashMap<String, PendingAsyncHook>,
}

lazy_static::lazy_static! {
    static ref ASYNC_HOOK_REGISTRY: Arc<Mutex<AsyncHookRegistryState>> = Arc::new(Mutex::new(
        AsyncHookRegistryState {
            pending_hooks: HashMap::new(),
        }
    ));
}

/// Parameters for registering a pending async hook
pub struct RegisterPendingAsyncHookParams {
    pub process_id: String,
    pub hook_id: String,
    pub async_response: AsyncHookJsonOutput,
    pub hook_name: String,
    pub hook_event: HookEvent,
    pub command: String,
    pub shell_command: ShellCommand,
    pub tool_name: Option<String>,
    pub plugin_id: Option<String>,
}

/// Register a pending async hook
pub fn register_pending_async_hook(params: RegisterPendingAsyncHookParams) {
    let timeout = params.async_response.async_timeout.unwrap_or(15) * 1000; // Default 15s, convert to ms

    log_for_debugging(&format!(
        "Hooks: Registering async hook {} ({}) with timeout {}ms",
        params.process_id, params.hook_name, timeout
    ));

    let hook_id = params.hook_id.clone();
    let hook_name = params.hook_name.clone();
    let hook_event = params.hook_event.clone();
    let process_id = params.process_id.clone();
    let shell_task_output = params.shell_command.task_output.clone();

    // Create progress interval that polls shell command output
    let _progress_handle = start_hook_progress_interval(HookProgressParams {
        hook_id: params.hook_id.clone(),
        hook_name: params.hook_name.clone(),
        hook_event: params.hook_event.clone(),
        get_output: Arc::new(move || {
            let task_output = shell_task_output.clone();
            Box::pin(async move {
                let stdout = task_output.get_stdout().await;
                let stderr = task_output.get_stderr();
                let output = format!("{}{}", stdout, stderr);
                HookOutput {
                    stdout,
                    stderr,
                    output,
                }
            })
        }),
        interval_ms: None,
    });

    let pending_hook = PendingAsyncHook {
        process_id: params.process_id.clone(),
        hook_id: params.hook_id,
        hook_name: params.hook_name,
        hook_event: params.hook_event,
        tool_name: params.tool_name,
        plugin_id: params.plugin_id,
        start_time: std::time::SystemTime::now(),
        timeout_ms: timeout,
        command: params.command,
        response_attachment_sent: false,
        shell_command: Some(params.shell_command),
        progress_task_id: None,
    };

    let mut registry = ASYNC_HOOK_REGISTRY.lock().unwrap();
    registry
        .pending_hooks
        .insert(params.process_id, pending_hook);
}

/// Get all pending async hooks that haven't sent their response attachment
pub fn get_pending_async_hooks() -> Vec<Arc<Mutex<PendingAsyncHook>>> {
    let registry = ASYNC_HOOK_REGISTRY.lock().unwrap();
    registry
        .pending_hooks
        .values()
        .filter(|hook| !hook.response_attachment_sent)
        .map(|hook| {
            Arc::new(Mutex::new(PendingAsyncHook {
                process_id: hook.process_id.clone(),
                hook_id: hook.hook_id.clone(),
                hook_name: hook.hook_name.clone(),
                hook_event: hook.hook_event.clone(),
                tool_name: hook.tool_name.clone(),
                plugin_id: hook.plugin_id.clone(),
                start_time: hook.start_time,
                timeout_ms: hook.timeout_ms,
                command: hook.command.clone(),
                response_attachment_sent: hook.response_attachment_sent,
                shell_command: None, // Can't clone shell command
                progress_task_id: None,
            }))
        })
        .collect()
}

pub struct HookProgressParams {
    pub hook_id: String,
    pub hook_name: String,
    pub hook_event: HookEvent,
    pub get_output: Arc<
        dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = HookOutput> + Send>>
            + Send
            + Sync,
    >,
    pub interval_ms: Option<u64>,
}

pub struct HookOutput {
    pub stdout: String,
    pub stderr: String,
    pub output: String,
}

const MAX_PENDING_EVENTS: usize = 100;

static mut EVENT_HANDLER: Option<Box<dyn Fn(HookExecutionEvent) + Send + Sync>> = None;
static mut PENDING_EVENTS: Vec<HookExecutionEvent> = Vec::new();
static mut ALL_HOOK_EVENTS_ENABLED: bool = false;

// Always emitted hook events regardless of includeHookEvents option
const ALWAYS_EMITTED_HOOK_EVENTS: [&str; 2] = ["SessionStart", "Setup"];

#[derive(Debug, Clone)]
pub enum HookExecutionEvent {
    Started {
        hook_id: String,
        hook_name: String,
        hook_event: String,
    },
    Progress {
        hook_id: String,
        hook_name: String,
        hook_event: String,
        stdout: String,
        stderr: String,
        output: String,
    },
    Response {
        hook_id: String,
        hook_name: String,
        hook_event: String,
        output: String,
        stdout: String,
        stderr: String,
        exit_code: Option<i32>,
        outcome: HookOutcome,
    },
}

#[derive(Debug, Clone)]
pub enum HookOutcome {
    Success,
    Error,
    Cancelled,
}

fn emit_hook_event(event: HookExecutionEvent) {
    unsafe {
        if let Some(ref handler) = EVENT_HANDLER {
            handler(event);
        } else {
            PENDING_EVENTS.push(event);
            if PENDING_EVENTS.len() > MAX_PENDING_EVENTS {
                PENDING_EVENTS.remove(0);
            }
        }
    }
}

fn should_emit(hook_event: &str) -> bool {
    if ALWAYS_EMITTED_HOOK_EVENTS.contains(&hook_event) {
        return true;
    }
    unsafe { ALL_HOOK_EVENTS_ENABLED }
}

/// Register a handler for hook execution events
pub fn register_hook_event_handler(handler: Option<Box<dyn Fn(HookExecutionEvent) + Send + Sync>>) {
    unsafe {
        let old_handler = EVENT_HANDLER.take();
        EVENT_HANDLER = handler;

        // If we have a new handler and pending events, deliver them
        if let Some(ref handler) = EVENT_HANDLER {
            let events: Vec<HookExecutionEvent> = PENDING_EVENTS.drain(..).collect();
            for event in events {
                handler(event);
            }
        } else {
            // Restore old handler if any
            if let Some(old) = old_handler {
                EVENT_HANDLER = Some(old);
            }
        }
    }
}

/// Emit hook started event
pub fn emit_hook_started(hook_id: &str, hook_name: &str, hook_event: &str) {
    if !should_emit(hook_event) {
        return;
    }
    emit_hook_event(HookExecutionEvent::Started {
        hook_id: hook_id.to_string(),
        hook_name: hook_name.to_string(),
        hook_event: hook_event.to_string(),
    });
}

/// Emit hook progress event
pub fn emit_hook_progress(params: HookProgressParams) {
    if !should_emit(params.hook_event.as_str()) {
        return;
    }
    emit_hook_event(HookExecutionEvent::Progress {
        hook_id: params.hook_id,
        hook_name: params.hook_name,
        hook_event: params.hook_event.as_str().to_string(),
        stdout: String::new(),
        stderr: String::new(),
        output: String::new(),
    });
}

/// Start a progress interval that periodically emits hook progress events.
/// Returns a JoinHandle that can be aborted to stop the interval.
pub fn start_hook_progress_interval(params: HookProgressParams) -> tokio::task::JoinHandle<()> {
    if !should_emit(params.hook_event.as_str()) {
        return tokio::spawn(async {});
    }

    let interval_ms = params.interval_ms.unwrap_or(1000);
    let hook_id = params.hook_id.clone();
    let hook_name = params.hook_name.clone();
    let hook_event = params.hook_event.clone();
    let get_output = params.get_output;

    // Spawn tokio task for progress polling
    tokio::spawn(async move {
        let mut last_emitted_output = String::new();
        let mut interval = interval(Duration::from_millis(interval_ms));

        loop {
            interval.tick().await;
            let output = get_output().await;
            if output.output == last_emitted_output {
                continue;
            }
            last_emitted_output = output.output.clone();

            emit_hook_event(HookExecutionEvent::Progress {
                hook_id: hook_id.clone(),
                hook_name: hook_name.clone(),
                hook_event: hook_event.as_str().to_string(),
                stdout: output.stdout,
                stderr: output.stderr,
                output: output.output,
            });
        }
    })
}

/// Emit hook response event
pub fn emit_hook_response(data: HookResponseData) {
    // Always log full hook output to debug log for verbose mode debugging
    let output_to_log =
        if !data.stdout.is_empty() || !data.stderr.is_empty() || !data.output.is_empty() {
            if !data.stdout.is_empty() {
                Some(&data.stdout)
            } else if !data.stderr.is_empty() {
                Some(&data.stderr)
            } else {
                Some(&data.output)
            }
        } else {
            None
        };

    if let Some(output) = output_to_log {
        log_for_debugging(&format!(
            "Hook {} ({}) {:?}:\n{}",
            data.hook_name, data.hook_event, data.outcome, output
        ));
    }

    if !should_emit(&data.hook_event) {
        return;
    }

    emit_hook_event(HookExecutionEvent::Response {
        hook_id: data.hook_id,
        hook_name: data.hook_name,
        hook_event: data.hook_event,
        output: data.output,
        stdout: data.stdout,
        stderr: data.stderr,
        exit_code: data.exit_code,
        outcome: data.outcome,
    });
}

pub struct HookResponseData {
    pub hook_id: String,
    pub hook_name: String,
    pub hook_event: String,
    pub output: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub outcome: HookOutcome,
}

/// Enable emission of all hook event types (beyond SessionStart and Setup)
pub fn set_all_hook_events_enabled(enabled: bool) {
    unsafe {
        ALL_HOOK_EVENTS_ENABLED = enabled;
    }
}

/// Clear hook event state
pub fn clear_hook_event_state() {
    unsafe {
        EVENT_HANDLER = None;
        PENDING_EVENTS.clear();
        ALL_HOOK_EVENTS_ENABLED = false;
    }
}

/// Finalize a hook after completion
async fn finalize_hook(_hook: &PendingAsyncHook, exit_code: i32, outcome: HookOutcome) {
    // Note: progress_task_id cannot be called through a shared reference
    // since it's a FnOnce. In practice, the caller would have already stopped it.

    let stdout = if let Some(shell_cmd) = &_hook.shell_command {
        shell_cmd.task_output.get_stdout().await
    } else {
        String::new()
    };
    let stderr = _hook
        .shell_command
        .as_ref()
        .map_or(String::new(), |s| s.task_output.get_stderr());

    if let Some(shell_cmd) = &_hook.shell_command {
        shell_cmd.cleanup();
    }

    emit_hook_response(HookResponseData {
        hook_id: _hook.hook_id.clone(),
        hook_name: _hook.hook_name.clone(),
        hook_event: _hook.hook_event.as_str().to_string(),
        output: format!("{}{}", stdout, stderr),
        stdout,
        stderr,
        exit_code: Some(exit_code),
        outcome,
    });
}

/// Response from check_for_async_hook_responses
pub struct AsyncHookResponse {
    pub process_id: String,
    pub response: SyncHookJsonOutput,
    pub hook_name: String,
    pub hook_event: HookEvent,
    pub tool_name: Option<String>,
    pub plugin_id: Option<String>,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

/// Check for completed async hook responses
pub async fn check_for_async_hook_responses() -> Vec<AsyncHookResponse> {
    let mut responses: Vec<AsyncHookResponse> = Vec::new();

    let pending_count;
    let hooks_snapshot;
    {
        let registry = ASYNC_HOOK_REGISTRY.lock().unwrap();
        pending_count = registry.pending_hooks.len();
        hooks_snapshot = registry
            .pending_hooks
            .values()
            .map(|h| h.process_id.clone())
            .collect::<Vec<_>>();
    }

    log_for_debugging(&format!(
        "Hooks: Found {} total hooks in registry",
        pending_count
    ));

    let mut process_ids_to_remove: Vec<String> = Vec::new();
    let mut session_start_completed = false;

    for process_id in hooks_snapshot {
        let hook_result = {
            let mut registry = ASYNC_HOOK_REGISTRY.lock().unwrap();
            let hook = match registry.pending_hooks.get_mut(&process_id) {
                Some(h) => h,
                None => continue,
            };

            if !hook.shell_command.is_some() {
                log_for_debugging(&format!(
                    "Hooks: Hook {} has no shell command, removing from registry",
                    process_id
                ));
                hook.progress_task_id = None;

                process_ids_to_remove.push(process_id.clone());
                continue;
            }

            let shell_cmd = hook.shell_command.as_ref().unwrap();
            if shell_cmd.status == ShellCommandStatus::Killed {
                log_for_debugging(&format!(
                    "Hooks: Hook {} is killed, removing from registry",
                    process_id
                ));
                hook.progress_task_id = None;

                shell_cmd.cleanup();
                process_ids_to_remove.push(process_id.clone());
                continue;
            }

            if shell_cmd.status != ShellCommandStatus::Completed {
                continue;
            }

            if hook.response_attachment_sent {
                log_for_debugging(&format!(
                    "Hooks: Skipping hook {} - already delivered/sent",
                    process_id
                ));
                hook.progress_task_id = None;

                process_ids_to_remove.push(process_id.clone());
                continue;
            }

            let stdout = shell_cmd.task_output.get_stdout().await;
            if stdout.trim().is_empty() {
                log_for_debugging(&format!("Hooks: Skipping hook {} - no stdout", process_id));
                hook.progress_task_id = None;

                process_ids_to_remove.push(process_id.clone());
                continue;
            }

            let lines: Vec<&str> = stdout.lines().collect();
            log_for_debugging(&format!(
                "Hooks: Processing {} lines of stdout for {}",
                lines.len(),
                process_id
            ));

            let exit_code = 0; // Would come from shell command result

            let mut response = SyncHookJsonOutput {
                extra: HashMap::new(),
            };
            for line in &lines {
                if line.trim().starts_with('{') {
                    log_for_debugging(&format!(
                        "Hooks: Found JSON line: {}...",
                        &line.trim().chars().take(100).collect::<String>()
                    ));
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line.trim()) {
                        if !parsed
                            .as_object()
                            .map_or(false, |obj| obj.contains_key("async"))
                        {
                            log_for_debugging(&format!(
                                "Hooks: Found sync response from {}: {}",
                                process_id,
                                serde_json::to_string(&parsed).unwrap_or_default()
                            ));
                            if let Some(obj) = parsed.as_object() {
                                response.extra =
                                    obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                            }
                            break;
                        }
                    }
                }
            }

            hook.response_attachment_sent = true;
            let is_session_start = hook.hook_event == HookEvent::SessionStart;

            // Finalize (spawn to avoid blocking)
            let hook_clone = PendingAsyncHook {
                process_id: hook.process_id.clone(),
                hook_id: hook.hook_id.clone(),
                hook_name: hook.hook_name.clone(),
                hook_event: hook.hook_event.clone(),
                tool_name: hook.tool_name.clone(),
                plugin_id: hook.plugin_id.clone(),
                start_time: hook.start_time,
                timeout_ms: hook.timeout_ms,
                command: hook.command.clone(),
                response_attachment_sent: true,
                shell_command: None,
                progress_task_id: None,
            };
            tokio::spawn(async move {
                finalize_hook(&hook_clone, exit_code, HookOutcome::Success).await;
            });

            process_ids_to_remove.push(process_id.clone());
            session_start_completed = session_start_completed || is_session_start;

            Some((
                process_id.clone(),
                response,
                hook.hook_name.clone(),
                hook.hook_event.clone(),
                hook.tool_name.clone(),
                hook.plugin_id.clone(),
                stdout,
                shell_cmd.task_output.get_stderr(),
                Some(exit_code),
            ))
        };

        if let Some((
            process_id,
            response,
            hook_name,
            hook_event,
            tool_name,
            plugin_id,
            stdout,
            stderr,
            exit_code,
        )) = hook_result
        {
            responses.push(AsyncHookResponse {
                process_id,
                response,
                hook_name,
                hook_event,
                tool_name,
                plugin_id,
                stdout,
                stderr,
                exit_code,
            });
        }
    }

    // Remove processed hooks
    {
        let mut registry = ASYNC_HOOK_REGISTRY.lock().unwrap();
        for process_id in process_ids_to_remove {
            registry.pending_hooks.remove(&process_id);
        }
    }

    if session_start_completed {
        log_for_debugging("Invalidating session env cache after SessionStart hook completed");
        invalidate_session_env_cache();
    }

    log_for_debugging(&format!(
        "Hooks: checkForNewResponses returning {} responses",
        responses.len()
    ));

    responses
}

/// Remove delivered async hooks from the registry
pub fn remove_delivered_async_hooks(process_ids: &[String]) {
    let mut registry = ASYNC_HOOK_REGISTRY.lock().unwrap();
    for process_id in process_ids {
        if let Some(hook) = registry.pending_hooks.get(process_id) {
            if hook.response_attachment_sent {
                log_for_debugging(&format!("Hooks: Removing delivered hook {}", process_id));
                // Note: can't call progress_task_id on borrowed ref
            }
        }
        registry.pending_hooks.remove(process_id);
    }
}

/// Finalize all pending async hooks (e.g., on shutdown)
pub async fn finalize_pending_async_hooks() {
    let hooks_snapshot;
    {
        let registry = ASYNC_HOOK_REGISTRY.lock().unwrap();
        hooks_snapshot = registry
            .pending_hooks
            .values()
            .map(|h| h.process_id.clone())
            .collect::<Vec<_>>();
    }

    let mut futures = Vec::new();
    for process_id in hooks_snapshot {
        let mut registry = ASYNC_HOOK_REGISTRY.lock().unwrap();
        if let Some(hook) = registry.pending_hooks.remove(&process_id) {
            let exit_code;
            let outcome;

            if let Some(ref shell_cmd) = hook.shell_command {
                if shell_cmd.status == ShellCommandStatus::Completed {
                    exit_code = 0;
                    outcome = HookOutcome::Success;
                } else {
                    if shell_cmd.status != ShellCommandStatus::Killed {
                        // Can't mutate through ref
                    }
                    exit_code = 1;
                    outcome = HookOutcome::Cancelled;
                }
            } else {
                exit_code = 1;
                outcome = HookOutcome::Cancelled;
            }

            futures.push(tokio::spawn(async move {
                finalize_hook(&hook, exit_code, outcome).await;
            }));
        }
    }

    // Wait for all finalize tasks
    for f in futures {
        let _ = f.await;
    }
}

/// Clear all async hooks (test utility)
pub fn clear_all_async_hooks() {
    let mut registry = ASYNC_HOOK_REGISTRY.lock().unwrap();
    for hook in registry.pending_hooks.values() {
        // Can't call progress_task_id through &ref
    }
    registry.pending_hooks.clear();
}

/// Log for debugging (simplified)
fn log_for_debugging(msg: &str) {
    log::debug!("{}", msg);
}

/// Invalidate session env cache (simplified)
fn invalidate_session_env_cache() {
    log::debug!("Invalidating session env cache");
}

/// JSON parse helper
fn json_parse(s: &str) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::from_str(s)
}

/// JSON stringify helper
fn json_stringify(value: &serde_json::Value) -> String {
    serde_json::to_string(value).unwrap_or_default()
}
