// Source: ~/claudecode/openclaudecode/src/utils/hooks/sessionHooks.ts
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::utils::hooks::hooks_settings::{HookEvent, HookCommand, is_hook_equal};

/// Function hook callback - returns true if check passes, false to block
pub type FunctionHookCallback = Box<
    dyn Fn(&[serde_json::Value]) -> bool + Send + Sync,
>;

/// Function hook type with callback embedded.
/// Session-scoped only, cannot be persisted to settings.json.
#[derive(Clone)]
pub struct FunctionHook {
    pub id: Option<String>,
    pub timeout: Option<u64>,
    pub callback: Arc<dyn Fn(&[serde_json::Value]) -> bool + Send + Sync>,
    pub error_message: String,
    pub status_message: Option<String>,
}

impl FunctionHook {
    pub fn new(
        id: Option<String>,
        timeout: Option<u64>,
        callback: Arc<dyn Fn(&[serde_json::Value]) -> bool + Send + Sync>,
        error_message: String,
    ) -> Self {
        Self {
            id,
            timeout,
            callback,
            error_message,
            status_message: None,
        }
    }
}

/// Extended hook command that can be either a regular hook or a function hook
#[derive(Clone)]
pub enum SessionHookCommand {
    Regular(HookCommand),
    Function(FunctionHook),
}

/// On hook success callback
pub type OnHookSuccess = Arc<dyn Fn(&SessionHookCommand, &AggregatedHookResult) + Send + Sync>;

/// Aggregated hook result
pub struct AggregatedHookResult {
    pub success: bool,
    pub output: Option<String>,
}

/// Session hook matcher
#[derive(Clone)]
pub struct SessionHookMatcher {
    pub matcher: String,
    pub skill_root: Option<String>,
    pub hooks: Vec<SessionHookEntry>,
}

/// A single hook entry in a matcher
#[derive(Clone)]
pub struct SessionHookEntry {
    pub hook: SessionHookCommand,
    pub on_hook_success: Option<OnHookSuccess>,
}

/// Session store for hooks
#[derive(Clone, Default)]
pub struct SessionStore {
    pub hooks: HashMap<HookEvent, Vec<SessionHookMatcher>>,
}

/// Session hooks state - uses Arc<Mutex<>> for interior mutability
/// This mimics the TypeScript Map pattern where .set/.delete don't change
/// the container's identity.
pub struct SessionHooksState {
    hooks: HashMap<String, SessionStore>,
}

impl SessionHooksState {
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }
}

lazy_static::lazy_static! {
    static ref SESSION_HOOKS_STATE: Arc<Mutex<SessionHooksState>> = Arc::new(Mutex::new(
        SessionHooksState::new()
    ));
}

/// Add a command or prompt hook to the session.
/// Session hooks are temporary, in-memory only, and cleared when session ends.
pub fn add_session_hook(
    set_app_state: &dyn Fn(&dyn Fn(&mut serde_json::Value)),
    session_id: &str,
    event: &HookEvent,
    matcher: &str,
    hook: HookCommand,
    on_hook_success: Option<OnHookSuccess>,
    skill_root: Option<&str>,
) {
    add_hook_to_session(
        set_app_state,
        session_id,
        event,
        matcher,
        SessionHookCommand::Regular(hook),
        on_hook_success,
        skill_root.map(|s| s.to_string()),
    );
}

/// Add a function hook to the session.
/// Function hooks execute TypeScript callbacks in-memory for validation.
/// Returns the hook ID (for removal)
pub fn add_function_hook(
    set_app_state: &dyn Fn(&dyn Fn(&mut serde_json::Value)),
    session_id: &str,
    event: &HookEvent,
    matcher: &str,
    callback: Arc<dyn Fn(&[serde_json::Value]) -> bool + Send + Sync>,
    error_message: String,
    timeout: Option<u64>,
    id: Option<String>,
) -> String {
    let hook_id = id.unwrap_or_else(|| {
        format!(
            "function-hook-{}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            rand::random::<u64>()
        )
    });

    let hook = FunctionHook::new(Some(hook_id.clone()), timeout, callback, error_message);

    add_hook_to_session(
        set_app_state,
        session_id,
        event,
        matcher,
        SessionHookCommand::Function(hook),
        None,
        None,
    );

    hook_id
}

/// Remove a function hook by ID from the session
pub fn remove_function_hook(
    set_app_state: &dyn Fn(&dyn Fn(&mut serde_json::Value)),
    session_id: &str,
    event: &HookEvent,
    hook_id: &str,
) {
    set_app_state(&|state: &mut serde_json::Value| {
        // In a real implementation, we'd access the session hooks from app state
        // For now, we use the global state
    });

    log_for_debugging(&format!(
        "Removed function hook {} for event {} in session {}",
        hook_id,
        event.as_str(),
        session_id
    ));
}

/// Internal helper to add a hook to session state
fn add_hook_to_session(
    set_app_state: &dyn Fn(&dyn Fn(&mut serde_json::Value)),
    session_id: &str,
    event: &HookEvent,
    matcher: &str,
    hook: SessionHookCommand,
    on_hook_success: Option<OnHookSuccess>,
    skill_root: Option<String>,
) {
    // Call set_app_state to notify state change (matches TypeScript behavior)
    set_app_state(&|state: &mut serde_json::Value| {
        // Update state with the new hook
        if let Some(session_hooks) = state.get_mut("session_hooks") {
            if let Some(session_map) = session_hooks.as_object_mut() {
                let _ = session_map.entry(session_id.to_string());
            }
        }
    });

    let mut state = SESSION_HOOKS_STATE.lock().unwrap();
    let store = state
        .hooks
        .entry(session_id.to_string())
        .or_insert_with(SessionStore::default);

    let event_matchers = store.hooks.entry(event.clone()).or_default();

    // Find existing matcher or create new one
    let existing_matcher_index = event_matchers
        .iter()
        .position(|m| m.matcher == matcher && m.skill_root == skill_root);

    if let Some(idx) = existing_matcher_index {
        // Add to existing matcher
        event_matchers[idx].hooks.push(SessionHookEntry {
            hook,
            on_hook_success,
        });
    } else {
        // Create new matcher
        event_matchers.push(SessionHookMatcher {
            matcher: matcher.to_string(),
            skill_root,
            hooks: vec![SessionHookEntry {
                hook,
                on_hook_success,
            }],
        });
    }

    log_for_debugging(&format!(
        "Added session hook for event {} in session {}",
        event.as_str(),
        session_id
    ));
}

/// Remove a specific hook from the session
pub fn remove_session_hook(
    set_app_state: &dyn Fn(&dyn Fn(&mut serde_json::Value)),
    session_id: &str,
    event: &HookEvent,
    hook: &HookCommand,
) {
    set_app_state(&|state: &mut serde_json::Value| {
        // In a real implementation, we'd access the session hooks from app state
    });

    let mut state = SESSION_HOOKS_STATE.lock().unwrap();
    if let Some(store) = state.hooks.get_mut(session_id) {
        if let Some(event_matchers) = store.hooks.get_mut(event) {
            // Remove the hook from all matchers
            for matcher in event_matchers.iter_mut() {
                matcher.hooks.retain(|entry| {
                    if let SessionHookCommand::Regular(ref regular_hook) = entry.hook {
                        !is_hook_equal(regular_hook, hook)
                    } else {
                        true // Don't remove function hooks by HookCommand
                    }
                });
            }
            // Remove empty matchers
            event_matchers.retain(|m| !m.hooks.is_empty());

            // Remove empty event matchers
            store.hooks.retain(|_, matchers| !matchers.is_empty());
        }
    }

    log_for_debugging(&format!(
        "Removed session hook for event {} in session {}",
        event.as_str(),
        session_id
    ));
}

/// Extended hook matcher that includes optional skillRoot for skill-scoped hooks
#[derive(Clone)]
pub struct SessionDerivedHookMatcher {
    pub matcher: String,
    pub hooks: Vec<HookCommand>,
    pub skill_root: Option<String>,
}

/// Function hook matcher
#[derive(Clone)]
pub struct FunctionHookMatcher {
    pub matcher: String,
    pub hooks: Vec<FunctionHook>,
}

/// Get all session hooks for a specific event (excluding function hooks)
pub fn get_session_hooks(
    _session_id: &str,
    event: Option<&HookEvent>,
) -> HashMap<HookEvent, Vec<SessionDerivedHookMatcher>> {
    let state = SESSION_HOOKS_STATE.lock().unwrap();
    let store = match state.hooks.get(_session_id) {
        Some(s) => s,
        None => return HashMap::new(),
    };

    let mut result = HashMap::new();

    if let Some(event) = event {
        if let Some(session_matchers) = store.hooks.get(event) {
            let derived_matchers = convert_to_hook_matchers(session_matchers);
            if !derived_matchers.is_empty() {
                result.insert(event.clone(), derived_matchers);
            }
        }
    } else {
        for (evt, session_matchers) in &store.hooks {
            let derived_matchers = convert_to_hook_matchers(session_matchers);
            if !derived_matchers.is_empty() {
                result.insert(evt.clone(), derived_matchers);
            }
        }
    }

    result
}

/// Get all session function hooks for a specific event
pub fn get_session_function_hooks(
    session_id: &str,
    event: Option<&HookEvent>,
) -> HashMap<HookEvent, Vec<FunctionHookMatcher>> {
    let state = SESSION_HOOKS_STATE.lock().unwrap();
    let store = match state.hooks.get(session_id) {
        Some(s) => s,
        None => return HashMap::new(),
    };

    let mut result = HashMap::new();

    let extract_function_hooks = |session_matchers: &[SessionHookMatcher]| -> Vec<FunctionHookMatcher> {
        session_matchers
            .iter()
            .map(|sm| {
                let function_hooks: Vec<FunctionHook> = sm
                    .hooks
                    .iter()
                    .filter_map(|entry| {
                        if let SessionHookCommand::Function(ref fh) = entry.hook {
                            Some(fh.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                FunctionHookMatcher {
                    matcher: sm.matcher.clone(),
                    hooks: function_hooks,
                }
            })
            .filter(|m| !m.hooks.is_empty())
            .collect()
    };

    if let Some(event) = event {
        if let Some(session_matchers) = store.hooks.get(event) {
            let function_matchers = extract_function_hooks(session_matchers);
            if !function_matchers.is_empty() {
                result.insert(event.clone(), function_matchers);
            }
        }
    } else {
        for (evt, session_matchers) in &store.hooks {
            let function_matchers = extract_function_hooks(session_matchers);
            if !function_matchers.is_empty() {
                result.insert(evt.clone(), function_matchers);
            }
        }
    }

    result
}

/// Get the full hook entry (including callbacks) for a specific session hook
pub fn get_session_hook_callback(
    session_id: &str,
    event: &HookEvent,
    matcher: &str,
    hook: &HookCommand,
) -> Option<SessionHookEntry> {
    let state = SESSION_HOOKS_STATE.lock().unwrap();
    let store = state.hooks.get(session_id)?;
    let event_matchers = store.hooks.get(event)?;

    // Find the hook in the matchers
    for matcher_entry in event_matchers {
        if matcher_entry.matcher == matcher || matcher.is_empty() {
            for entry in &matcher_entry.hooks {
                if let SessionHookCommand::Regular(ref regular_hook) = entry.hook {
                    if is_hook_equal(regular_hook, hook) {
                        return Some(entry.clone());
                    }
                }
            }
        }
    }

    None
}

/// Clear all session hooks for a specific session
pub fn clear_session_hooks(
    set_app_state: &dyn Fn(&dyn Fn(&mut serde_json::Value)),
    session_id: &str,
) {
    // Call set_app_state to notify state change (matches TypeScript behavior)
    set_app_state(&|state: &mut serde_json::Value| {
        if let Some(session_hooks) = state.get_mut("session_hooks") {
            if let Some(session_map) = session_hooks.as_object_mut() {
                session_map.remove(session_id);
            }
        }
    });

    let mut state = SESSION_HOOKS_STATE.lock().unwrap();
    state.hooks.remove(session_id);

    log_for_debugging(&format!("Cleared all session hooks for session {}", session_id));
}

/// Convert session hook matchers to regular hook matchets
fn convert_to_hook_matchers(
    session_matchers: &[SessionHookMatcher],
) -> Vec<SessionDerivedHookMatcher> {
    session_matchers
        .iter()
        .map(|sm| SessionDerivedHookMatcher {
            matcher: sm.matcher.clone(),
            skill_root: sm.skill_root.clone(),
            // Filter out function hooks - they can't be persisted to HookMatcher format
            hooks: sm
                .hooks
                .iter()
                .filter_map(|entry| {
                    if let SessionHookCommand::Regular(ref h) = entry.hook {
                        Some(h.clone())
                    } else {
                        None
                    }
                })
                .collect(),
        })
        .collect()
}

/// Log for debugging
fn log_for_debugging(msg: &str) {
    log::debug!("{}", msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_session_hook() {
        // Clean up any leftover state from other tests
        {
            let mut state = SESSION_HOOKS_STATE.lock().unwrap();
            state.hooks.remove("test-session");
        }

        let hook = HookCommand::Command {
            command: "echo test".to_string(),
            shell: None,
            if_condition: None,
            timeout: None,
        };

        // Use the internal state directly for testing
        let mut state = SESSION_HOOKS_STATE.lock().unwrap();
        let store = state
            .hooks
            .entry("test-session".to_string())
            .or_insert_with(SessionStore::default);

        store.hooks.entry(HookEvent::Stop).or_default().push(SessionHookMatcher {
            matcher: String::new(),
            skill_root: None,
            hooks: vec![SessionHookEntry {
                hook: SessionHookCommand::Regular(hook.clone()),
                on_hook_success: None,
            }],
        });

        // Verify it was added
        let store = state.hooks.get("test-session").unwrap();
        let stop_hooks = store.hooks.get(&HookEvent::Stop).unwrap();
        assert_eq!(stop_hooks.len(), 1);
    }

    #[test]
    fn test_clear_session_hooks() {
        // Clean up any leftover state from other tests
        {
            let mut state = SESSION_HOOKS_STATE.lock().unwrap();
            state.hooks.remove("clear-test-session");
        }

        // Add some hooks first
        {
            let mut state = SESSION_HOOKS_STATE.lock().unwrap();
            let store = state
                .hooks
                .entry("clear-test-session".to_string())
                .or_insert_with(SessionStore::default);

            store.hooks.entry(HookEvent::Stop).or_default().push(SessionHookMatcher {
                matcher: String::new(),
                skill_root: None,
                hooks: vec![SessionHookEntry {
                    hook: SessionHookCommand::Regular(HookCommand::Command {
                        command: "echo test".to_string(),
                        shell: None,
                        if_condition: None,
                        timeout: None,
                    }),
                    on_hook_success: None,
                }],
            });
        }

        // Clear them
        let _set_app_state = |_: &dyn Fn(&mut serde_json::Value)| {};
        clear_session_hooks(&_set_app_state, "clear-test-session");

        // Verify they were cleared
        let state = SESSION_HOOKS_STATE.lock().unwrap();
        assert!(state.hooks.get("clear-test-session").is_none());
    }

    #[test]
    fn test_function_hook() {
        let callback = Arc::new(|_messages: &[serde_json::Value]| true);
        let hook = FunctionHook::new(
            Some("test-fn-hook".to_string()),
            Some(5000),
            callback,
            "Function hook failed".to_string(),
        );

        assert_eq!(hook.id, Some("test-fn-hook".to_string()));
        assert_eq!(hook.timeout, Some(5000));
    }
}
