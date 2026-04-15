//! Bridge UI for console logging.
//!
//! Translated from openclaudecode/src/bridge/bridgeUI.ts
//!
//! Console-based UI for the bridge. For a full TUI implementation,
//! see the ai-code project which uses ratatui.

use crate::bridge::bridge_status_util::{
    build_bridge_connect_url, build_bridge_session_url, format_duration, timestamp,
    truncate_to_width, StatusState, TOOL_DISPLAY_EXPIRY_MS,
};
use crate::bridge::bridge_types::{BridgeConfig, SessionActivity, SessionActivityType, SpawnMode};

/// Bridge logger implementation for console output
pub struct BridgeLoggerImpl {
    verbose: bool,
    write: Box<dyn Fn(&str) + Send + Sync>,
    status_line_count: usize,
    current_state: StatusState,
    current_state_text: String,
    repo_name: String,
    branch: String,
    debug_log_path: String,
    connect_url: String,
    cached_ingress_url: String,
    cached_environment_id: String,
    active_session_url: Option<String>,
    qr_visible: bool,
    last_tool_summary: Option<String>,
    last_tool_time: u64,
    session_active: u32,
    session_max: u32,
    spawn_mode_display: Option<SpawnMode>,
    spawn_mode: SpawnMode,
    session_display_info: std::collections::HashMap<String, SessionDisplayInfo>,
    connecting: bool,
    connecting_tick: u64,
}

/// Per-session display info for multi-session mode
#[derive(Debug, Clone)]
struct SessionDisplayInfo {
    title: Option<String>,
    url: String,
    activity: Option<SessionActivity>,
}

impl BridgeLoggerImpl {
    /// Create a new bridge logger
    pub fn new(verbose: bool, write: Option<Box<dyn Fn(&str) + Send + Sync>>) -> Self {
        let write_fn = write.unwrap_or_else(|| Box::new(|s| print!("{}", s)));
        Self {
            verbose,
            write: write_fn,
            status_line_count: 0,
            current_state: StatusState::Idle,
            current_state_text: "Ready".to_string(),
            repo_name: String::new(),
            branch: String::new(),
            debug_log_path: String::new(),
            connect_url: String::new(),
            cached_ingress_url: String::new(),
            cached_environment_id: String::new(),
            active_session_url: None,
            qr_visible: false,
            last_tool_summary: None,
            last_tool_time: 0,
            session_active: 0,
            session_max: 1,
            spawn_mode_display: None,
            spawn_mode: SpawnMode::SingleSession,
            session_display_info: std::collections::HashMap::new(),
            connecting: false,
            connecting_tick: 0,
        }
    }

    /// Print the bridge banner
    pub fn print_banner(&mut self, config: &BridgeConfig, environment_id: &str) {
        self.cached_ingress_url = config.session_ingress_url.clone();
        self.cached_environment_id = environment_id.to_string();
        self.connect_url =
            build_bridge_connect_url(environment_id, Some(&config.session_ingress_url));

        if self.verbose {
            (self.write)(&format!("Remote Control v{}\n", env!("CARGO_PKG_VERSION")));
        }
        if self.verbose {
            if config.spawn_mode != SpawnMode::SingleSession {
                (self.write)(&format!("Spawn mode: {:?}\n", config.spawn_mode));
                (self.write)(&format!(
                    "Max concurrent sessions: {}\n",
                    config.max_sessions
                ));
            }
            (self.write)(&format!("Environment ID: {}\n", environment_id));
        }
        if config.sandbox {
            (self.write)("Sandbox: Enabled\n");
        }
        (self.write)("\n");

        // Start connecting spinner
        self.start_connecting();
    }

    /// Log session start
    pub fn log_session_start(&self, session_id: &str, prompt: &str) {
        if self.verbose {
            let short = truncate_to_width(prompt, 80);
            (self.write)(&format!(
                "[{}] Session started: \"{}\" ({})\n",
                timestamp(),
                short,
                session_id
            ));
        }
    }

    /// Log session complete
    pub fn log_session_complete(&self, session_id: &str, duration_ms: u64) {
        (self.write)(&format!(
            "[{}] Session completed ({}) {}\n",
            timestamp(),
            format_duration(duration_ms),
            session_id
        ));
    }

    /// Log session failed
    pub fn log_session_failed(&self, session_id: &str, error: &str) {
        (self.write)(&format!(
            "[{}] Session failed: {} {}\n",
            timestamp(),
            error,
            session_id
        ));
    }

    /// Log status message
    pub fn log_status(&self, message: &str) {
        (self.write)(&format!("[{}] {}\n", timestamp(), message));
    }

    /// Log verbose message
    pub fn log_verbose(&self, message: &str) {
        if self.verbose {
            (self.write)(&format!("[{}] {}\n", timestamp(), message));
        }
    }

    /// Log error message
    pub fn log_error(&self, message: &str) {
        (self.write)(&format!("[{}] Error: {}\n", timestamp(), message));
    }

    /// Log reconnected
    pub fn log_reconnected(&self, disconnected_ms: u64) {
        (self.write)(&format!(
            "[{}] Reconnected after {}\n",
            timestamp(),
            format_duration(disconnected_ms)
        ));
    }

    /// Set repository info
    pub fn set_repo_info(&mut self, repo: &str, branch_name: &str) {
        self.repo_name = repo.to_string();
        self.branch = branch_name.to_string();
    }

    /// Set debug log path
    pub fn set_debug_log_path(&mut self, path: &str) {
        self.debug_log_path = path.to_string();
    }

    /// Update to idle status
    pub fn update_idle_status(&mut self) {
        self.stop_connecting();
        self.current_state = StatusState::Idle;
        self.current_state_text = "Ready".to_string();
        self.last_tool_summary = None;
        self.last_tool_time = 0;
        self.active_session_url = None;
        self.render_status_line();
    }

    /// Set attached state
    pub fn set_attached(&mut self, session_id: &str) {
        self.stop_connecting();
        self.current_state = StatusState::Attached;
        self.current_state_text = "Connected".to_string();
        self.last_tool_summary = None;
        self.last_tool_time = 0;

        // Multi-session: keep footer/QR on the environment connect URL
        if self.session_max <= 1 {
            self.active_session_url = Some(build_bridge_session_url(
                session_id,
                &self.cached_environment_id,
                Some(&self.cached_ingress_url),
            ));
        }
        self.render_status_line();
    }

    /// Update reconnecting status
    pub fn update_reconnecting_status(&mut self, delay_str: &str, elapsed_str: &str) {
        self.stop_connecting();
        self.clear_status_lines();
        self.current_state = StatusState::Reconnecting;

        // Simple status display
        let status = format!(
            "Reconnecting - retrying in {} - disconnected {}\n",
            delay_str, elapsed_str
        );
        self.write(&status);
    }

    /// Update failed status
    pub fn update_failed_status(&mut self, error: &str) {
        self.stop_connecting();
        self.clear_status_lines();
        self.current_state = StatusState::Failed;

        let mut suffix = String::new();
        if !self.repo_name.is_empty() {
            suffix = format!(" · {}", self.repo_name);
        }
        if !self.branch.is_empty() {
            suffix = format!("{} · {}", suffix, self.branch);
        }

        let error_suffix = if error.is_empty() {
            String::new()
        } else {
            format!("\n{}", error)
        };
        let status = format!("Remote Control Failed{}{}\n", suffix, error_suffix);
        self.write(&status);
        self.write("Something went wrong, please try again\n");
    }

    /// Update session status
    pub fn update_session_status(
        &mut self,
        _session_id: &str,
        _elapsed: &str,
        activity: &SessionActivity,
        _trail: &[String],
    ) {
        // Cache tool activity for the second status line
        if activity.activity_type == SessionActivityType::ToolStart {
            self.last_tool_summary = Some(activity.summary.clone());
            self.last_tool_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
        }
        self.render_status_line();
    }

    /// Clear status
    pub fn clear_status(&mut self) {
        self.stop_connecting();
        self.clear_status_lines();
    }

    /// Toggle QR code visibility
    pub fn toggle_qr(&mut self) {
        self.qr_visible = !self.qr_visible;
        self.render_status_line();
    }

    /// Update session count
    pub fn update_session_count(&mut self, active: u32, max: u32, mode: SpawnMode) {
        if self.session_active == active && self.session_max == max && self.spawn_mode == mode {
            return;
        }
        self.session_active = active;
        self.session_max = max;
        self.spawn_mode = mode;
    }

    /// Set spawn mode display
    pub fn set_spawn_mode_display(&mut self, mode: Option<SpawnMode>) {
        if self.spawn_mode_display == mode {
            return;
        }
        self.spawn_mode_display = mode;
        if let Some(m) = mode {
            self.spawn_mode = m;
        }
    }

    /// Add session
    pub fn add_session(&mut self, session_id: &str, url: &str) {
        self.session_display_info.insert(
            session_id.to_string(),
            SessionDisplayInfo {
                title: None,
                url: url.to_string(),
                activity: None,
            },
        );
    }

    /// Update session activity
    pub fn update_session_activity(&mut self, session_id: &str, activity: &SessionActivity) {
        if let Some(info) = self.session_display_info.get_mut(session_id) {
            info.activity = Some(activity.clone());
        }
    }

    /// Set session title
    pub fn set_session_title(&mut self, session_id: &str, title: &str) {
        if let Some(info) = self.session_display_info.get_mut(session_id) {
            info.title = Some(title.to_string());
        }

        // Guard against reconnecting/failed
        if self.current_state == StatusState::Reconnecting
            || self.current_state == StatusState::Failed
        {
            return;
        }

        if self.session_max == 1 {
            // Single-session: show title in the main status line too.
            self.current_state = StatusState::Titled;
            self.current_state_text = truncate_to_width(title, 40);
        }
        self.render_status_line();
    }

    /// Remove session
    pub fn remove_session(&mut self, session_id: &str) {
        self.session_display_info.remove(session_id);
    }

    /// Refresh display
    pub fn refresh_display(&mut self) {
        // Skip during reconnecting/failed
        if self.current_state == StatusState::Reconnecting
            || self.current_state == StatusState::Failed
        {
            return;
        }
        self.render_status_line();
    }

    // Helper methods

    fn start_connecting(&mut self) {
        self.stop_connecting();
        self.render_connecting_line();
        self.connecting = true;
    }

    fn stop_connecting(&mut self) {
        self.connecting = false;
    }

    fn render_connecting_line(&mut self) {
        self.clear_status_lines();

        let frames = ["-", "\\", "|", "/"];
        let frame = frames[(self.connecting_tick as usize) % frames.len()];

        let mut suffix = String::new();
        if !self.repo_name.is_empty() {
            suffix = format!(" · {}", self.repo_name);
        }
        if !self.branch.is_empty() {
            suffix = format!("{} · {}", suffix, self.branch);
        }

        let line = format!(
            "{} Connecting{}{}\n",
            frame,
            suffix,
            if suffix.is_empty() { "" } else { "" }
        );
        self.write(&line);
        self.status_line_count += 1;
    }

    fn render_status_line(&mut self) {
        // Skip during reconnecting/failed
        if self.current_state == StatusState::Reconnecting
            || self.current_state == StatusState::Failed
        {
            return;
        }

        self.clear_status_lines();

        let is_idle = self.current_state == StatusState::Idle;

        // Build suffix with repo and branch
        let mut suffix = String::new();
        if !self.repo_name.is_empty() {
            suffix = format!(" · {}", self.repo_name);
        }
        // In worktree mode each session gets its own branch
        if !self.branch.is_empty() && self.spawn_mode != SpawnMode::Worktree {
            suffix = format!("{} · {}", suffix, self.branch);
        }

        let indicator = if is_idle { "[*]" } else { "[+]" };
        let state_text = &self.current_state_text;

        // Build status line
        let status = format!("{} {}{}\n", indicator, state_text, suffix);
        self.write(&status);
        self.status_line_count += 1;

        // Session count and per-session list (multi-session mode only)
        if self.session_max > 1 {
            let mode_hint = match self.spawn_mode {
                SpawnMode::Worktree => "New sessions will be created in an isolated worktree",
                SpawnMode::SameDir => "New sessions will be created in the current directory",
                SpawnMode::SingleSession => "",
            };
            if !mode_hint.is_empty() {
                let line = format!(
                    "    Capacity: {}/{} · {}\n",
                    self.session_active, self.session_max, mode_hint
                );
                self.write(&line);
                self.status_line_count += 1;
            }

            for (_, info) in &self.session_display_info {
                let title_text = info.title.as_deref().unwrap_or("Attached");
                let truncated = truncate_to_width(title_text, 35);
                let act = &info.activity;
                let show_act = act.is_some()
                    && act
                        .as_ref()
                        .map(|a| {
                            a.activity_type != SessionActivityType::Result
                                && a.activity_type != SessionActivityType::Error
                        })
                        .unwrap_or(false);
                let act_text = if show_act {
                    format!(
                        " {}",
                        truncate_to_width(act.as_ref().unwrap().summary.as_str(), 40)
                    )
                } else {
                    String::new()
                };
                let line = format!("    {}{}\n", truncated, act_text);
                self.write(&line);
                self.status_line_count += 1;
            }
        }

        // Tool activity line for single-session mode
        if self.session_max == 1 && !is_idle {
            if let Some(ref summary) = self.last_tool_summary {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                if now - self.last_tool_time < TOOL_DISPLAY_EXPIRY_MS {
                    let line = format!("  {}\n", truncate_to_width(summary, 60));
                    self.write(&line);
                    self.status_line_count += 1;
                }
            }
        }

        // Footer text
        let url = self
            .active_session_url
            .as_deref()
            .unwrap_or(&self.connect_url);
        (self.write)("\n");
        self.status_line_count += 1;

        let footer_text = if is_idle {
            format!("Code everywhere with the Claude app or {}", url)
        } else {
            format!("Continue coding in the Claude app or {}", url)
        };
        (self.write)(&format!("{}\n", footer_text));
        self.status_line_count += 1;

        let qr_hint = if self.qr_visible {
            "space to hide QR code"
        } else {
            "space to show QR code"
        };
        (self.write)(&format!("{}\n", qr_hint));
        self.status_line_count += 1;
    }

    fn clear_status_lines(&mut self) {
        if self.status_line_count > 0 {
            // Move cursor up and clear lines
            let escape = format!("\x1b[{}A\x1b[J", self.status_line_count);
            (self.write)(&escape);
            self.status_line_count = 0;
        }
    }

    fn write(&self, text: &str) {
        (self.write)(text);
    }
}

/// Create a bridge logger with options
pub fn create_bridge_logger(
    verbose: bool,
    write: Option<Box<dyn Fn(&str) + Send + Sync>>,
) -> BridgeLoggerImpl {
    BridgeLoggerImpl::new(verbose, write)
}
