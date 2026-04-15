//! Session spawner for bridge sessions.
//!
//! Translated from openclaudode/src/bridge/sessionRunner.ts

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

// =============================================================================
// CONSTANTS
// =============================================================================

const MAX_ACTIVITIES: usize = 10;
const MAX_STDERR_LINES: usize = 10;

// =============================================================================
// TYPES
// =============================================================================

/// Status when a session ends
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionDoneStatus {
    Completed,
    Failed,
    Interrupted,
}

impl std::fmt::Display for SessionDoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionDoneStatus::Completed => write!(f, "completed"),
            SessionDoneStatus::Failed => write!(f, "failed"),
            SessionDoneStatus::Interrupted => write!(f, "interrupted"),
        }
    }
}

/// Type of session activity
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionActivityType {
    ToolStart,
    Text,
    Result,
    Error,
}

/// A session activity (ring buffer of recent activities)
#[derive(Debug, Clone)]
pub struct SessionActivity {
    pub activity_type: SessionActivityType,
    pub summary: String,
    pub timestamp: i64,
}

/// Permission request from child CLI
#[derive(Debug, Clone)]
pub struct PermissionRequest {
    pub request_id: String,
    pub request: PermissionRequestInner,
}

#[derive(Debug, Clone)]
pub struct PermissionRequestInner {
    pub subtype: String,
    pub tool_name: String,
    pub input: serde_json::Value,
    pub tool_use_id: String,
}

/// Session spawn options
#[derive(Clone)]
pub struct SessionSpawnOpts {
    pub session_id: String,
    pub sdk_url: String,
    pub access_token: String,
    /// When true, spawn the child with CCR v2 env vars
    pub use_ccr_v2: bool,
    /// Required when useCcrV2 is true. Obtained from POST /worker/register.
    pub worker_epoch: Option<u64>,
    /// Fires once with the text of the first real user message seen
    pub on_first_user_message: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

/// Session handle for controlling a spawned session
pub struct SessionHandle {
    pub session_id: String,
    pub done: Arc<Mutex<Option<SessionDoneStatus>>>,
    pub activities: Arc<Mutex<VecDeque<SessionActivity>>>,
    pub current_activity: Arc<Mutex<Option<SessionActivity>>>,
    pub access_token: Arc<Mutex<String>>,
    pub last_stderr: Arc<Mutex<VecDeque<String>>>,
    child: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<std::process::ChildStdin>>>,
    killed: Arc<Mutex<bool>>,
    sigkill_sent: Arc<Mutex<bool>>,
}

impl SessionHandle {
    /// Kill the session gracefully (SIGTERM)
    pub fn kill(&self) {
        let mut killed = self.killed.lock().unwrap();
        if *killed {
            return;
        }
        *killed = true;

        if let Ok(mut child_guard) = self.child.lock() {
            if let Some(ref mut child) = *child_guard {
                let _ = child.kill();
            }
        }
    }

    /// Force kill the session (SIGKILL)
    pub fn force_kill(&self) {
        let mut sent = self.sigkill_sent.lock().unwrap();
        if *sent {
            return;
        }

        if let Ok(mut child_guard) = self.child.lock() {
            if let Some(ref mut child) = *child_guard {
                if child.id() > 0 {
                    *sent = true;
                    let _ = child.kill();
                }
            }
        }
    }

    /// Write directly to child stdin
    pub fn write_stdin(&self, data: &str) {
        if let Ok(mut stdin_guard) = self.stdin.lock() {
            if let Some(ref mut stdin) = *stdin_guard {
                let _ = stdin.write_all(data.as_bytes());
                let _ = stdin.flush();
            }
        }
    }

    /// Update the access token for a running session
    pub fn update_access_token(&self, token: String) {
        if let Ok(mut access) = self.access_token.lock() {
            *access = token.clone();
        }

        // Send the fresh token to the child process via stdin
        let msg = serde_json::json!({
            "type": "update_environment_variables",
            "variables": { "AI_CODE_SESSION_ACCESS_TOKEN": token }
        });
        self.write_stdin(&format!("{}\n", msg));
    }

    /// Get current activity
    pub fn get_current_activity(&self) -> Option<SessionActivity> {
        self.current_activity.lock().ok().and_then(|g| g.clone())
    }

    /// Get activities
    pub fn get_activities(&self) -> Vec<SessionActivity> {
        self.activities
            .lock()
            .ok()
            .map(|g| g.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get last stderr lines
    pub fn get_last_stderr(&self) -> Vec<String> {
        self.last_stderr
            .lock()
            .ok()
            .map(|g| g.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get access token
    pub fn get_access_token(&self) -> String {
        self.access_token
            .lock()
            .ok()
            .map(|g| g.clone())
            .unwrap_or_default()
    }
}

/// Session spawner dependencies
pub struct SessionSpawnerDeps {
    pub exec_path: String,
    /// Arguments that must precede the CLI flags when spawning
    pub script_args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
    pub verbose: bool,
    pub sandbox: bool,
    pub debug_file: Option<String>,
    pub permission_mode: Option<String>,
    pub on_debug: Arc<dyn Fn(String) + Send + Sync>,
    pub on_activity: Option<Arc<dyn Fn(String, SessionActivity) + Send + Sync>>,
    pub on_permission_request: Option<Arc<dyn Fn(String, PermissionRequest, String) + Send + Sync>>,
}

impl Default for SessionSpawnerDeps {
    fn default() -> Self {
        Self {
            exec_path: String::new(),
            script_args: Vec::new(),
            env: std::collections::HashMap::new(),
            verbose: false,
            sandbox: false,
            debug_file: None,
            permission_mode: None,
            on_debug: Arc::new(|_| {}),
            on_activity: None,
            on_permission_request: None,
        }
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Map tool names to human-readable verbs for status display
fn tool_verb(name: &str) -> String {
    let verb = match name {
        "Read" => "Reading",
        "Write" => "Writing",
        "Edit" => "Editing",
        "MultiEdit" => "Editing",
        "Bash" => "Running",
        "Glob" => "Searching",
        "Grep" => "Searching",
        "WebFetch" => "Fetching",
        "WebSearch" => "Searching",
        "Task" => "Running task",
        "FileReadTool" => "Reading",
        "FileWriteTool" => "Writing",
        "FileEditTool" => "Editing",
        "GlobTool" => "Searching",
        "GrepTool" => "Searching",
        "BashTool" => "Running",
        "NotebookEditTool" => "Editing notebook",
        "LSP" => "LSP",
        _ => name,
    };
    verb.to_string()
}

/// Extract summary from tool invocation
fn tool_summary(name: &str, input: &serde_json::Value) -> String {
    let verb = tool_verb(name);

    let target = input
        .get("file_path")
        .or_else(|| input.get("filePath"))
        .or_else(|| input.get("pattern"))
        .or_else(|| input.get("command"))
        .or_else(|| input.get("url"))
        .or_else(|| input.get("query"))
        .and_then(|v| v.as_str())
        .map(|s| {
            if s.len() > 60 {
                format!("{}...", &s[..60])
            } else {
                s.to_string()
            }
        });

    match target {
        Some(t) => format!("{} {}", verb, t),
        None => verb.to_string(),
    }
}

/// Build a short preview of tool input for debug logging
fn input_preview(input: &serde_json::Value) -> String {
    let mut parts = Vec::new();
    if let Some(obj) = input.as_object() {
        for (key, val) in obj.iter().take(3) {
            if let Some(s) = val.as_str() {
                let truncated = if s.len() > 100 {
                    format!("{}...", &s[..100])
                } else {
                    s.to_string()
                };
                parts.push(format!("{}=\"{}\"", key, truncated));
            }
        }
    }
    parts.join(" ")
}

/// Extract activities from an NDJSON line
fn extract_activities(
    line: &str,
    session_id: &str,
    on_debug: &Arc<dyn Fn(String) + Send + Sync>,
) -> Vec<SessionActivity> {
    let parsed: serde_json::Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let obj = match parsed.as_object() {
        Some(o) => o,
        None => return Vec::new(),
    };

    let mut activities = Vec::new();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    // Handle assistant messages (tool_use, text)
    if let Some(msg_type) = obj.get("type").and_then(|v| v.as_str()) {
        if msg_type == "assistant" {
            if let Some(message) = obj.get("message").and_then(|v| v.as_object()) {
                if let Some(content) = message.get("content").and_then(|v| v.as_array()) {
                    for block in content {
                        let block_obj = match block.as_object() {
                            Some(o) => o,
                            None => continue,
                        };

                        let block_type = match block_obj.get("type").and_then(|v| v.as_str()) {
                            Some(t) => t,
                            None => continue,
                        };

                        if block_type == "tool_use" {
                            let name = block_obj
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Tool");
                            let input = block_obj.get("input").unwrap_or(&serde_json::Value::Null);
                            let summary = tool_summary(name, input);

                            on_debug(format!(
                                "[bridge:activity] sessionId={} tool_use name={} {}",
                                session_id,
                                name,
                                input_preview(input)
                            ));

                            activities.push(SessionActivity {
                                activity_type: SessionActivityType::ToolStart,
                                summary,
                                timestamp: now,
                            });
                        } else if block_type == "text" {
                            if let Some(text) = block_obj.get("text").and_then(|v| v.as_str()) {
                                if !text.is_empty() {
                                    let summary = if text.len() > 80 {
                                        format!("{}...", &text[..80])
                                    } else {
                                        text.to_string()
                                    };

                                    on_debug(format!(
                                        "[bridge:activity] sessionId={} text \"{}\"",
                                        session_id,
                                        if text.len() > 100 {
                                            format!("{}...", &text[..100])
                                        } else {
                                            text.to_string()
                                        }
                                    ));

                                    activities.push(SessionActivity {
                                        activity_type: SessionActivityType::Text,
                                        summary,
                                        timestamp: now,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        } else if msg_type == "result" {
            let subtype = obj.get("subtype").and_then(|v| v.as_str());

            if subtype == Some("success") {
                on_debug(format!(
                    "[bridge:activity] sessionId={} result subtype=success",
                    session_id
                ));

                activities.push(SessionActivity {
                    activity_type: SessionActivityType::Result,
                    summary: "Session completed".to_string(),
                    timestamp: now,
                });
            } else if let Some(sub) = subtype {
                let errors = obj.get("errors").and_then(|v| v.as_array());
                let error_summary = errors
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("Error: {}", sub));

                on_debug(format!(
                    "[bridge:activity] sessionId={} result subtype={} error=\"{}\"",
                    session_id, sub, error_summary
                ));

                activities.push(SessionActivity {
                    activity_type: SessionActivityType::Error,
                    summary: error_summary,
                    timestamp: now,
                });
            } else {
                on_debug(format!(
                    "[bridge:activity] sessionId={} result subtype=undefined",
                    session_id
                ));
            }
        }
    }

    activities
}

/// Extract plain text from a user message NDJSON line.
fn extract_user_message_text(msg: &serde_json::Value) -> Option<String> {
    let obj = msg.as_object()?;

    // Skip tool-result user messages (wrapped subagent results) and synthetic messages
    if obj.get("parent_tool_use_id").is_some()
        || obj
            .get("isSynthetic")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        || obj
            .get("isReplay")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    {
        return None;
    }

    let message = obj.get("message")?.as_object()?;
    let content = message.get("content")?;

    let text = if let Some(s) = content.as_str() {
        Some(s.to_string())
    } else if let Some(arr) = content.as_array() {
        for block in arr {
            if let Some(block_obj) = block.as_object() {
                if block_obj.get("type").and_then(|v| v.as_str()) == Some("text") {
                    if let Some(text) = block_obj.get("text").and_then(|v| v.as_str()) {
                        return Some(text.trim().to_string());
                    }
                }
            }
        }
        None
    } else {
        None
    };

    text.filter(|s| !s.is_empty())
}

// =============================================================================
// SESSION SPAWNER
// =============================================================================

/// Create a session spawner
pub fn create_session_spawner(
    deps: SessionSpawnerDeps,
) -> impl Fn(SessionSpawnOpts, &str) -> SessionHandle {
    move |opts: SessionSpawnOpts, dir: &str| {
        let on_debug = &deps.on_debug;

        // Build args
        let mut args = deps.script_args.clone();
        args.push("--print".to_string());
        args.push("--sdk-url".to_string());
        args.push(opts.sdk_url.clone());
        args.push("--session-id".to_string());
        args.push(opts.session_id.clone());
        args.push("--input-format".to_string());
        args.push("stream-json".to_string());
        args.push("--output-format".to_string());
        args.push("stream-json".to_string());
        args.push("--replay-user-messages".to_string());

        if deps.verbose {
            args.push("--verbose".to_string());
        }

        if let Some(ref debug_file) = deps.debug_file {
            args.push("--debug-file".to_string());
            args.push(debug_file.clone());
        }

        if let Some(ref permission_mode) = deps.permission_mode {
            args.push("--permission-mode".to_string());
            args.push(permission_mode.clone());
        }

        // Build env
        let mut env = deps.env.clone();
        env.remove("AI_CODE_OAUTH_TOKEN");
        env.insert(
            "AI_CODE_ENVIRONMENT_KIND".to_string(),
            "bridge".to_string(),
        );

        if deps.sandbox {
            env.insert("AI_CODE_FORCE_SANDBOX".to_string(), "1".to_string());
        }

        env.insert(
            "AI_CODE_SESSION_ACCESS_TOKEN".to_string(),
            opts.access_token.clone(),
        );

        // v1: HybridTransport
        env.insert(
            "AI_CODE_POST_FOR_SESSION_INGRESS_V2".to_string(),
            "1".to_string(),
        );

        // v2: SSETransport + CCRClient
        if opts.use_ccr_v2 {
            env.insert("AI_CODE_USE_CCR_V2".to_string(), "1".to_string());
            if let Some(epoch) = opts.worker_epoch {
                env.insert("AI_CODE_WORKER_EPOCH".to_string(), epoch.to_string());
            }
        }

        on_debug(format!(
            "[bridge:session] Spawning sessionId={} sdkUrl={} accessToken={}",
            opts.session_id,
            opts.sdk_url,
            if opts.access_token.is_empty() {
                "MISSING"
            } else {
                "present"
            }
        ));
        on_debug(format!("[bridge:session] Child args: {:?}", args));

        // Spawn child process
        let mut child = Command::new(&deps.exec_path);
        child.args(&args);
        child.current_dir(dir);
        child.envs(&env);
        child.stdin(Stdio::piped());
        child.stdout(Stdio::piped());
        child.stderr(Stdio::piped());

        #[cfg(windows)]
        child.windows_hide(true);

        let mut child = match child.spawn() {
            Ok(c) => c,
            Err(e) => {
                on_debug(format!(
                    "[bridge:session] sessionId={} spawn error: {}",
                    opts.session_id, e
                ));
                // Return a failed session handle
                return SessionHandle {
                    session_id: opts.session_id,
                    done: Arc::new(Mutex::new(Some(SessionDoneStatus::Failed))),
                    activities: Arc::new(Mutex::new(VecDeque::new())),
                    current_activity: Arc::new(Mutex::new(None)),
                    access_token: Arc::new(Mutex::new(opts.access_token)),
                    last_stderr: Arc::new(Mutex::new(VecDeque::new())),
                    child: Arc::new(Mutex::new(None)),
                    stdin: Arc::new(Mutex::new(None)),
                    killed: Arc::new(Mutex::new(true)),
                    sigkill_sent: Arc::new(Mutex::new(true)),
                };
            }
        };

        let pid = child.id();
        on_debug(format!(
            "[bridge:session] sessionId={} pid={}",
            opts.session_id, pid
        ));

        // Get stdin
        let stdin = child.stdin.take();

        // Initialize state
        let activities: Arc<Mutex<VecDeque<SessionActivity>>> =
            Arc::new(Mutex::new(VecDeque::with_capacity(MAX_ACTIVITIES)));
        let current_activity: Arc<Mutex<Option<SessionActivity>>> = Arc::new(Mutex::new(None));
        let last_stderr: Arc<Mutex<VecDeque<String>>> =
            Arc::new(Mutex::new(VecDeque::with_capacity(MAX_STDERR_LINES)));
        let done_status: Arc<Mutex<Option<SessionDoneStatus>>> = Arc::new(Mutex::new(None));

        let session_id = opts.session_id.clone();
        let on_activity = deps.on_activity.clone();
        let on_permission_request = deps.on_permission_request.clone();
        let verbose = deps.verbose;

        // Handle stdout
        if let Some(stdout) = child.stdout.take() {
            let activities_clone = activities.clone();
            let current_activity_clone = current_activity.clone();
            let session_id_clone = session_id.clone();
            let on_debug_clone = on_debug.clone();
            let on_activity_clone = on_activity.clone();
            let opts_clone = opts.clone();

            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    // Log message
                    on_debug_clone(format!(
                        "[bridge:ws] sessionId={} <<< {}",
                        session_id_clone,
                        if line.len() > 200 {
                            format!("{}...", &line[..200])
                        } else {
                            line.clone()
                        }
                    ));

                    // Forward in verbose mode
                    if verbose {
                        eprintln!("{}", line);
                    }

                    // Extract activities
                    let extracted = extract_activities(&line, &session_id_clone, &on_debug_clone);
                    for activity in extracted {
                        if let Ok(mut acts) = activities_clone.lock() {
                            if acts.len() >= MAX_ACTIVITIES {
                                acts.pop_front();
                            }
                            acts.push_back(activity.clone());

                            if let Ok(mut current) = current_activity_clone.lock() {
                                *current = Some(activity.clone());
                            }

                            if let Some(ref callback) = on_activity_clone {
                                callback(session_id_clone.clone(), activity);
                            }
                        }
                    }

                    // Check for control_request and user messages
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&line) {
                        if let Some(obj) = parsed.as_object() {
                            if obj.get("type").and_then(|v| v.as_str()) == Some("control_request") {
                                if let Some(request) =
                                    obj.get("request").and_then(|v| v.as_object())
                                {
                                    if request.get("subtype").and_then(|v| v.as_str())
                                        == Some("can_use_tool")
                                    {
                                        if let Some(ref callback) = on_permission_request {
                                            let perm_request = PermissionRequest {
                                                request_id: obj
                                                    .get("request_id")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("")
                                                    .to_string(),
                                                request: PermissionRequestInner {
                                                    subtype: request
                                                        .get("subtype")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("")
                                                        .to_string(),
                                                    tool_name: request
                                                        .get("tool_name")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("")
                                                        .to_string(),
                                                    input: request
                                                        .get("input")
                                                        .cloned()
                                                        .unwrap_or(serde_json::Value::Null),
                                                    tool_use_id: request
                                                        .get("tool_use_id")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("")
                                                        .to_string(),
                                                },
                                            };
                                            callback(
                                                opts_clone.session_id.clone(),
                                                perm_request,
                                                opts_clone.access_token.clone(),
                                            );
                                        }
                                    }
                                }
                            } else if obj.get("type").and_then(|v| v.as_str()) == Some("user") {
                                if let Some(text) = extract_user_message_text(&parsed) {
                                    if let Some(ref callback) = opts_clone.on_first_user_message {
                                        callback(text);
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }

        // Handle stderr
        if let Some(stderr) = child.stderr.take() {
            let last_stderr_clone = last_stderr.clone();
            let on_debug_clone = on_debug.clone();

            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    // Forward to stderr in verbose mode
                    if verbose {
                        eprintln!("{}", line);
                    }

                    // Ring buffer of last N lines
                    if let Ok(mut stderr_lines) = last_stderr_clone.lock() {
                        if stderr_lines.len() >= MAX_STDERR_LINES {
                            stderr_lines.pop_front();
                        }
                        stderr_lines.push_back(line.clone());
                    }

                    on_debug_clone(line);
                }
            });
        }

        // Wait for child to exit
        let session_id_clone = session_id.clone();
        let on_debug_clone = on_debug.clone();
        let done_status_clone = done_status.clone();
        let child_for_handle = Arc::new(Mutex::new(Some(child)));
        let child_for_thread = child_for_handle.clone();

        thread::spawn(move || {
            let mut child_guard = child_for_thread.lock().unwrap();
            if let Some(ref mut child) = *child_guard {
                let status = child.wait();
                let on_debug = on_debug_clone;

                match status {
                    Ok(exit_status) => {
                        // Check for interruption signals via exit code
                        // 15 = SIGTERM, 2 = SIGINT (platform-specific)
                        let code = exit_status.code().unwrap_or(-1);
                        if code == 15 || code == 2 || code == -11 {
                            on_debug(format!(
                                "[bridge:session] sessionId={} interrupted exit_code={} pid={}",
                                session_id_clone,
                                code,
                                child.id()
                            ));
                            if let Ok(mut status) = done_status_clone.lock() {
                                *status = Some(SessionDoneStatus::Interrupted);
                            }
                        } else if exit_status.success() {
                            on_debug(format!(
                                "[bridge:session] sessionId={} completed exit_code=0 pid={}",
                                session_id_clone,
                                child.id()
                            ));
                            if let Ok(mut status) = done_status_clone.lock() {
                                *status = Some(SessionDoneStatus::Completed);
                            }
                        } else {
                            on_debug(format!(
                                "[bridge:session] sessionId={} failed exit_code={:?} pid={}",
                                session_id_clone,
                                exit_status.code(),
                                child.id()
                            ));
                            if let Ok(mut status) = done_status_clone.lock() {
                                *status = Some(SessionDoneStatus::Failed);
                            }
                        }
                    }
                    Err(e) => {
                        on_debug(format!(
                            "[bridge:session] sessionId={} wait error: {}",
                            session_id_clone, e
                        ));
                        if let Ok(mut status) = done_status_clone.lock() {
                            *status = Some(SessionDoneStatus::Failed);
                        }
                    }
                }
            }
        });

        SessionHandle {
            session_id: opts.session_id,
            done: done_status,
            activities,
            current_activity,
            access_token: Arc::new(Mutex::new(opts.access_token)),
            last_stderr,
            child: child_for_handle,
            stdin: Arc::new(Mutex::new(stdin)),
            killed: Arc::new(Mutex::new(false)),
            sigkill_sent: Arc::new(Mutex::new(false)),
        }
    }
}

/// Sanitize a session ID for use in file names.
/// Strips any characters that could cause path traversal or other filesystem issues.
pub fn safe_filename_id(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_filename_id() {
        assert_eq!(safe_filename_id("session_abc123"), "session_abc123");
        assert_eq!(safe_filename_id("cse_abc-123"), "cse_abc-123");
        assert_eq!(safe_filename_id("../etc/passwd"), "___etc_passwd");
    }

    #[test]
    fn test_tool_summary() {
        let input = serde_json::json!({ "file_path": "/path/to/file.txt" });
        assert_eq!(tool_summary("Read", &input), "Reading /path/to/file.txt");

        let input2 = serde_json::json!({ "command": "ls -la" });
        assert_eq!(tool_summary("Bash", &input2), "Running ls -la");
    }

    #[test]
    fn test_input_preview() {
        let input = serde_json::json!({
            "file_path": "/test.txt",
            "content": "hello world"
        });
        let preview = input_preview(&input);
        assert!(preview.contains("file_path="));
    }
}
