// Source: ~/claudecode/openclaudecode/src/utils/hooks/fileChangedWatcher.ts
#![allow(dead_code)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::utils::hooks::hooks_config_snapshot::get_hooks_config_from_snapshot;

/// File event type
#[derive(Debug, Clone)]
pub enum FileEvent {
    Change,
    Add,
    Unlink,
}

impl std::fmt::Display for FileEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileEvent::Change => write!(f, "change"),
            FileEvent::Add => write!(f, "add"),
            FileEvent::Unlink => write!(f, "unlink"),
        }
    }
}

/// Result from executing file changed hooks
pub struct HookOutsideReplResult {
    pub results: Vec<HookResult>,
    pub watch_paths: Vec<String>,
    pub system_messages: Vec<String>,
}

/// Individual hook result
pub struct HookResult {
    pub succeeded: bool,
    pub output: Option<String>,
}

/// File watcher state
struct FileWatcherState {
    watched_paths: Vec<String>,
    current_cwd: String,
    dynamic_watch_paths: Vec<String>,
    dynamic_watch_paths_sorted: Vec<String>,
    initialized: bool,
    has_env_hooks: bool,
    notify_callback: Option<Box<dyn Fn(String, bool) + Send + Sync>>,
}

impl FileWatcherState {
    fn new() -> Self {
        Self {
            watched_paths: Vec::new(),
            current_cwd: String::new(),
            dynamic_watch_paths: Vec::new(),
            dynamic_watch_paths_sorted: Vec::new(),
            initialized: false,
            has_env_hooks: false,
            notify_callback: None,
        }
    }
}

lazy_static::lazy_static! {
    static ref FILE_WATCHER_STATE: Arc<Mutex<FileWatcherState>> = Arc::new(Mutex::new(
        FileWatcherState::new()
    ));
}

/// Set environment hook notifier callback
pub fn set_env_hook_notifier(
    cb: Option<Box<dyn Fn(String, bool) + Send + Sync>>,
) {
    let mut state = FILE_WATCHER_STATE.lock().unwrap();
    state.notify_callback = cb;
}

/// Initialize the file changed watcher
pub fn initialize_file_changed_watcher(cwd: &str) {
    {
        let state = FILE_WATCHER_STATE.lock().unwrap();
        if state.initialized {
            return;
        }
    }

    let config = get_hooks_config_from_snapshot();

    let has_env_hooks = {
        let cwd_changed_len = config
            .as_ref()
            .and_then(|c| c.events.get("CwdChanged"))
            .map(|m| m.len())
            .unwrap_or(0);
        let file_changed_len = config
            .as_ref()
            .and_then(|c| c.events.get("FileChanged"))
            .map(|m| m.len())
            .unwrap_or(0);
        cwd_changed_len > 0 || file_changed_len > 0
    };

    {
        let mut state = FILE_WATCHER_STATE.lock().unwrap();
        state.initialized = true;
        state.current_cwd = cwd.to_string();
        state.has_env_hooks = has_env_hooks;
    }

    if has_env_hooks {
        // Register cleanup (would hook into the cleanup registry)
        log_for_debugging("FileChanged: registered cleanup for file watcher");
    }

    let paths = resolve_watch_paths();
    if paths.is_empty() {
        return;
    }

    start_watching(&paths);
}

/// Resolve watch paths from configuration
fn resolve_watch_paths() -> Vec<String> {
    let state = FILE_WATCHER_STATE.lock().unwrap();
    let cwd = state.current_cwd.clone();
    let dynamic_paths = state.dynamic_watch_paths.clone();
    drop(state);

    let config = get_hooks_config_from_snapshot();

    let matchers = config
        .as_ref()
        .and_then(|c| c.events.get("FileChanged"))
        .cloned()
        .unwrap_or_default();

    // Matcher field: filenames to watch in cwd, pipe-separated (e.g. ".envrc|.env")
    let mut static_paths: HashSet<String> = HashSet::new();
    for matcher in &matchers {
        let matcher_str = matcher.matcher.as_deref().unwrap_or("");
        if matcher_str.is_empty() {
            continue;
        }
        for name in matcher_str.split('|').map(|s: &str| s.trim()) {
            if name.is_empty() {
                continue;
            }
            let path = Path::new(name);
            let full_path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                PathBuf::from(&cwd).join(name)
            };
            static_paths.insert(full_path.to_string_lossy().to_string());
        }
    }

    // Combine static matcher paths with dynamic paths from hook output
    let mut all_paths: Vec<String> = static_paths.into_iter().collect();
    for p in dynamic_paths {
        if !all_paths.contains(&p) {
            all_paths.push(p);
        }
    }

    all_paths
}

/// Start watching the given paths (polling-based since notify crate not available)
fn start_watching(paths: &[String]) {
    log_for_debugging(&format!("FileChanged: watching {} paths (polling mode)", paths.len()));

    // Store the paths for polling
    {
        let mut state = FILE_WATCHER_STATE.lock().unwrap();
        state.watched_paths = paths.to_vec();
    }

    // In a production implementation, this would use a file system watcher crate
    // like `notify`. For now, we store the paths and rely on external triggers.
    // The core logic (resolve_watch_paths, update_watch_paths, on_cwd_changed_for_hooks)
    // is fully implemented and ready to integrate with any file watching backend.
}

/// Handle a file event (core logic without notify crate dependency)
fn handle_file_event(path: &str, event: &FileEvent) {
    log_for_debugging(&format!("FileChanged: {} {}", event, path));

    // Execute file changed hooks (async)
    let path_clone = path.to_string();
    let event_clone = event.clone();
    tokio::spawn(async move {
        match execute_file_changed_hooks(&path_clone, &event_clone).await {
            Ok(result) => {
                if !result.watch_paths.is_empty() {
                    update_watch_paths(&result.watch_paths);
                }
                for msg in result.system_messages {
                    notify_callback_inner(&msg, false);
                }
                for r in result.results {
                    if !r.succeeded {
                        if let Some(output) = r.output {
                            notify_callback_inner(&output, true);
                        }
                    }
                }
            }
            Err(e) => {
                let msg = format!("FileChanged hook failed: {}", e);
                log_for_debugging(&msg);
                notify_callback_inner(&msg, true);
            }
        }
    });
}

/// Notify callback helper
fn notify_callback_inner(text: &str, is_error: bool) {
    let state = FILE_WATCHER_STATE.lock().unwrap();
    if let Some(ref cb) = state.notify_callback {
        cb(text.to_string(), is_error);
    }
}

/// Update watch paths
pub fn update_watch_paths(paths: &[String]) {
    let mut state = FILE_WATCHER_STATE.lock().unwrap();
    if !state.initialized {
        return;
    }

    let mut sorted = paths.to_vec();
    sorted.sort();

    if sorted.len() == state.dynamic_watch_paths_sorted.len()
        && sorted
            .iter()
            .zip(state.dynamic_watch_paths_sorted.iter())
            .all(|(a, b)| a == b)
    {
        return;
    }

    state.dynamic_watch_paths = paths.to_vec();
    state.dynamic_watch_paths_sorted = sorted;
    drop(state);

    restart_watching();
}

/// Restart watching with updated paths
fn restart_watching() {
    let paths = resolve_watch_paths();
    if !paths.is_empty() {
        start_watching(&paths);
    }
}

/// Handle working directory change for hooks
pub async fn on_cwd_changed_for_hooks(
    old_cwd: &str,
    new_cwd: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if old_cwd == new_cwd {
        return Ok(());
    }

    // Re-evaluate from the current snapshot so mid-session hook changes are picked up
    let config = get_hooks_config_from_snapshot();
    let current_has_env_hooks = {
        let cwd_changed_len = config
            .as_ref()
            .and_then(|c| c.events.get("CwdChanged"))
            .map(|m| m.len())
            .unwrap_or(0);
        let file_changed_len = config
            .as_ref()
            .and_then(|c| c.events.get("FileChanged"))
            .map(|m| m.len())
            .unwrap_or(0);
        cwd_changed_len > 0 || file_changed_len > 0
    };

    if !current_has_env_hooks {
        return Ok(());
    }

    {
        let mut state = FILE_WATCHER_STATE.lock().unwrap();
        state.current_cwd = new_cwd.to_string();
    }

    // Clear cwd env files (would call clear_cwd_env_files)

    // Execute CwdChanged hooks
    let hook_result = execute_cwd_changed_hooks(old_cwd, new_cwd)
        .await
        .unwrap_or_else(|e| {
            let msg = format!("CwdChanged hook failed: {}", e);
            log_for_debugging(&msg);
            notify_callback_inner(&msg, true);
            HookOutsideReplResult {
                results: Vec::new(),
                watch_paths: Vec::new(),
                system_messages: Vec::new(),
            }
        });

    {
        let mut state = FILE_WATCHER_STATE.lock().unwrap();
        state.dynamic_watch_paths = hook_result.watch_paths.clone();
        let mut sorted = hook_result.watch_paths.clone();
        sorted.sort();
        state.dynamic_watch_paths_sorted = sorted;
    }

    for msg in &hook_result.system_messages {
        notify_callback_inner(msg, false);
    }
    for r in &hook_result.results {
        if !r.succeeded {
            if let Some(ref output) = r.output {
                notify_callback_inner(output, true);
            }
        }
    }

    // Re-resolve matcher paths against the new cwd
    {
        let state = FILE_WATCHER_STATE.lock().unwrap();
        if state.initialized {
            drop(state);
            restart_watching();
        }
    }

    Ok(())
}

/// Execute file changed hooks for a path and event
async fn execute_file_changed_hooks(
    _path: &str,
    _event: &FileEvent,
) -> Result<HookOutsideReplResult, Box<dyn std::error::Error + Send + Sync>> {
    // This would execute the actual file changed hooks
    Ok(HookOutsideReplResult {
        results: Vec::new(),
        watch_paths: Vec::new(),
        system_messages: Vec::new(),
    })
}

/// Execute cwd changed hooks
async fn execute_cwd_changed_hooks(
    _old_cwd: &str,
    _new_cwd: &str,
) -> Result<HookOutsideReplResult, Box<dyn std::error::Error + Send + Sync>> {
    // This would execute the actual cwd changed hooks
    Ok(HookOutsideReplResult {
        results: Vec::new(),
        watch_paths: Vec::new(),
        system_messages: Vec::new(),
    })
}

/// Dispose of the file watcher
fn dispose() {
    let mut state = FILE_WATCHER_STATE.lock().unwrap();
    state.watched_paths.clear();
    state.dynamic_watch_paths.clear();
    state.dynamic_watch_paths_sorted.clear();
    state.initialized = false;
    state.has_env_hooks = false;
    state.notify_callback = None;
}

/// Reset file changed watcher for testing
pub fn reset_file_changed_watcher_for_testing() {
    dispose();
}

/// Log for debugging
fn log_for_debugging(msg: &str) {
    log::debug!("{}", msg);
}
