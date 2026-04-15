// Source: ~/claudecode/openclaudecode/src/utils/hooks/hookEvents.ts
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use tokio::time::{interval, Duration};

/// Hook events that are always emitted regardless of the includeHookEvents option
const ALWAYS_EMITTED_HOOK_EVENTS: [&str; 2] = ["SessionStart", "Setup"];

const MAX_PENDING_EVENTS: usize = 100;

/// Hook started event
#[derive(Debug, Clone)]
pub struct HookStartedEvent {
    pub hook_id: String,
    pub hook_name: String,
    pub hook_event: String,
}

/// Hook progress event
#[derive(Debug, Clone)]
pub struct HookProgressEvent {
    pub hook_id: String,
    pub hook_name: String,
    pub hook_event: String,
    pub stdout: String,
    pub stderr: String,
    pub output: String,
}

/// Hook response event
#[derive(Debug, Clone)]
pub struct HookResponseEvent {
    pub hook_id: String,
    pub hook_name: String,
    pub hook_event: String,
    pub output: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub outcome: HookOutcome,
}

/// Hook execution event
#[derive(Debug, Clone)]
pub enum HookExecutionEvent {
    Started(HookStartedEvent),
    Progress(HookProgressEvent),
    Response(HookResponseEvent),
}

/// Hook outcome
#[derive(Debug, Clone)]
pub enum HookOutcome {
    Success,
    Error,
    Cancelled,
}

/// Hook event handler type
pub type HookEventHandler = Box<dyn Fn(HookExecutionEvent) + Send + Sync>;

/// Parameters for progress output
pub struct ProgressOutput {
    pub stdout: String,
    pub stderr: String,
    pub output: String,
}

/// Parameters for starting a progress interval
pub struct StartHookProgressParams {
    pub hook_id: String,
    pub hook_name: String,
    pub hook_event: String,
    pub get_output: Arc<
        dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ProgressOutput> + Send>>
            + Send
            + Sync,
    >,
    pub interval_ms: Option<u64>,
}

/// Internal state for hook events
struct HookEventState {
    pending_events: Vec<HookExecutionEvent>,
    event_handler: Option<HookEventHandler>,
    all_hook_events_enabled: bool,
}

lazy_static::lazy_static! {
    static ref HOOK_EVENT_STATE: Arc<Mutex<HookEventState>> = Arc::new(Mutex::new(
        HookEventState {
            pending_events: Vec::new(),
            event_handler: None,
            all_hook_events_enabled: false,
        }
    ));
}

/// Register a handler for hook execution events
pub fn register_hook_event_handler(handler: Option<HookEventHandler>) {
    let mut state = HOOK_EVENT_STATE.lock().unwrap();

    // Take pending events first
    let events: Vec<HookExecutionEvent> = state.pending_events.drain(..).collect();
    state.event_handler = handler;

    // Deliver pending events to the new handler
    if let Some(ref handler) = state.event_handler {
        for event in events {
            handler(event);
        }
    }
}

/// Emit a hook event
fn emit(event: HookExecutionEvent) {
    let mut state = HOOK_EVENT_STATE.lock().unwrap();
    if let Some(ref handler) = state.event_handler {
        handler(event);
    } else {
        state.pending_events.push(event);
        if state.pending_events.len() > MAX_PENDING_EVENTS {
            state.pending_events.remove(0);
        }
    }
}

/// Check if a hook event should be emitted
fn should_emit(hook_event: &str) -> bool {
    if ALWAYS_EMITTED_HOOK_EVENTS.contains(&hook_event) {
        return true;
    }
    let state = HOOK_EVENT_STATE.lock().unwrap();
    state.all_hook_events_enabled
}

/// Emit hook started event
pub fn emit_hook_started(hook_id: &str, hook_name: &str, hook_event: &str) {
    if !should_emit(hook_event) {
        return;
    }

    emit(HookExecutionEvent::Started(HookStartedEvent {
        hook_id: hook_id.to_string(),
        hook_name: hook_name.to_string(),
        hook_event: hook_event.to_string(),
    }));
}

/// Emit hook progress event
pub fn emit_hook_progress(
    hook_id: &str,
    hook_name: &str,
    hook_event: &str,
    stdout: &str,
    stderr: &str,
    output: &str,
) {
    if !should_emit(hook_event) {
        return;
    }

    emit(HookExecutionEvent::Progress(HookProgressEvent {
        hook_id: hook_id.to_string(),
        hook_name: hook_name.to_string(),
        hook_event: hook_event.to_string(),
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
        output: output.to_string(),
    }));
}

/// Start a progress interval that periodically emits hook progress events.
/// Returns a function that stops the interval.
pub fn start_hook_progress_interval(
    params: StartHookProgressParams,
) -> Arc<dyn Fn() + Send + Sync> {
    if !should_emit(&params.hook_event) {
        return Arc::new(|| {});
    }

    let interval_ms = params.interval_ms.unwrap_or(1000);
    let hook_id = params.hook_id.clone();
    let hook_name = params.hook_name.clone();
    let hook_event = params.hook_event.clone();
    let get_output = params.get_output.clone();

    let stopped = Arc::new(Mutex::new(false));
    let stopped_clone = stopped.clone();

    // Spawn tokio task for progress polling
    let handle = tokio::spawn(async move {
        let mut last_emitted_output = String::new();
        let mut interval = interval(Duration::from_millis(interval_ms));

        loop {
            interval.tick().await;

            // Check if stopped
            if *stopped_clone.lock().unwrap() {
                break;
            }

            let output = get_output().await;
            if output.output == last_emitted_output {
                continue;
            }
            last_emitted_output = output.output.clone();

            emit_hook_progress(
                &hook_id,
                &hook_name,
                &hook_event,
                &output.stdout,
                &output.stderr,
                &output.output,
            );
        }
    });

    // Return closure that stops the task
    Arc::new(move || {
        let mut stopped = stopped.lock().unwrap();
        *stopped = true;
        handle.abort();
    })
}

/// Parameters for emitting a hook response
pub struct EmitHookResponseParams {
    pub hook_id: String,
    pub hook_name: String,
    pub hook_event: String,
    pub output: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub outcome: HookOutcome,
}

/// Emit hook response event
pub fn emit_hook_response(data: EmitHookResponseParams) {
    // Always log full hook output to debug log for verbose mode debugging
    let output_to_log = if !data.stdout.is_empty()
        || !data.stderr.is_empty()
        || !data.output.is_empty()
    {
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

    emit(HookExecutionEvent::Response(HookResponseEvent {
        hook_id: data.hook_id,
        hook_name: data.hook_name,
        hook_event: data.hook_event,
        output: data.output,
        stdout: data.stdout,
        stderr: data.stderr,
        exit_code: data.exit_code,
        outcome: data.outcome,
    }));
}

/// Enable emission of all hook event types (beyond SessionStart and Setup)
pub fn set_all_hook_events_enabled(enabled: bool) {
    let mut state = HOOK_EVENT_STATE.lock().unwrap();
    state.all_hook_events_enabled = enabled;
}

/// Clear hook event state
pub fn clear_hook_event_state() {
    let mut state = HOOK_EVENT_STATE.lock().unwrap();
    state.event_handler = None;
    state.pending_events.clear();
    state.all_hook_events_enabled = false;
}

/// Log for debugging
fn log_for_debugging(msg: &str) {
    log::debug!("{}", msg);
}
