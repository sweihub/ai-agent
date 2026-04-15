// Source: /data/home/swei/claudecode/openclaudecode/src/skills/bundled/debug.ts
//! Debug logging utilities
//!
 //! Translated from openclaudecode/src/utils/debug.ts

use crate::constants::env::{ai, system};
use crate::utils::debug_filter::{parse_debug_filter, should_show_debug_message, DebugFilter};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

/// Debug log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DebugLogLevel {
    Verbose,
    Debug,
    Info,
    Warn,
    Error,
}

impl DebugLogLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "verbose" => DebugLogLevel::Verbose,
            "debug" => DebugLogLevel::Debug,
            "info" => DebugLogLevel::Info,
            "warn" => DebugLogLevel::Warn,
            "error" => DebugLogLevel::Error,
            _ => DebugLogLevel::Debug,
        }
    }
}

static LEVEL_ORDER: Lazy<HashMap<DebugLogLevel, u8>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(DebugLogLevel::Verbose, 0);
    m.insert(DebugLogLevel::Debug, 1);
    m.insert(DebugLogLevel::Info, 2);
    m.insert(DebugLogLevel::Warn, 3);
    m.insert(DebugLogLevel::Error, 4);
    m
});

/// Minimum log level to include in debug output
static MIN_DEBUG_LOG_LEVEL: Lazy<Mutex<Option<DebugLogLevel>>> = Lazy::new(|| Mutex::new(None));

pub fn get_min_debug_log_level() -> DebugLogLevel {
    let mut level = MIN_DEBUG_LOG_LEVEL.lock().unwrap();
    if let Some(l) = *level {
        return l;
    }

    let raw = std::env::var(ai::CODE_DEBUG_LOG_LEVEL)
        .ok()
        .map(|s| s.to_lowercase().trim().to_string());

    let l = if let Some(ref raw) = raw {
        if LEVEL_ORDER.keys().any(|k| {
            let key = format!("{:?}", k).to_lowercase();
            key == *raw
        }) {
            DebugLogLevel::from_str(raw)
        } else {
            DebugLogLevel::Debug
        }
    } else {
        DebugLogLevel::Debug
    };

    *level = Some(l);
    l
}

static RUNTIME_DEBUG_ENABLED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

pub fn is_debug_mode() -> bool {
    let runtime = *RUNTIME_DEBUG_ENABLED.lock().unwrap();

    if runtime {
        return true;
    }

    // Check environment variables
    if std::env::var(system::DEBUG).is_ok() || std::env::var(system::DEBUG_SDK).is_ok() {
        return true;
    }

    // Check command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--debug" || arg == "-d") {
        return true;
    }

    // Check for --debug=pattern syntax
    if args.iter().any(|arg| arg.starts_with("--debug=")) {
        return true;
    }

    // Check for --debug-file
    if args.iter().any(|arg| arg.starts_with("--debug-file")) {
        return true;
    }

    false
}

/// Enables debug logging mid-session
pub fn enable_debug_logging() -> bool {
    let mut runtime = RUNTIME_DEBUG_ENABLED.lock().unwrap();
    let was_active = *runtime
        || std::env::var(ai::USER_TYPE)
            .map(|v| v == "ant")
            .unwrap_or(false);
    *runtime = true;
    was_active
}

/// Get debug filter from command line arguments
pub fn get_debug_filter() -> Option<DebugFilter> {
    let args: Vec<String> = std::env::args().collect();

    for arg in &args {
        if arg.starts_with("--debug=") {
            let pattern = arg.strip_prefix("--debug=").unwrap();
            return parse_debug_filter(Some(pattern));
        }
    }

    None
}

pub fn is_debug_to_stderr() -> bool {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .any(|arg| arg == "--debug-to-stderr" || arg == "-d2e")
}

pub fn get_debug_file_path() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();

    for (i, arg) in args.iter().enumerate() {
        if arg.starts_with("--debug-file=") {
            return Some(arg.strip_prefix("--debug-file=").unwrap().to_string());
        }
        if arg == "--debug-file" && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }

    None
}

fn should_log_debug_message(message: &str) -> bool {
    // Non-ants only write debug logs when debug mode is active
    let user_type = std::env::var(ai::USER_TYPE).unwrap_or_default();
    if user_type != "ant" && !is_debug_mode() {
        return false;
    }

    let filter = get_debug_filter();
    should_show_debug_message(message, &filter)
}

/// Log a debug message
pub fn log_for_debugging(message: &str, level: DebugLogLevel) {
    let min_level = get_min_debug_log_level();
    if LEVEL_ORDER[&level] < LEVEL_ORDER[&min_level] {
        return;
    }

    if !should_log_debug_message(message) {
        return;
    }

    let timestamp = chrono::Utc::now().to_rfc3339();
    let level_str = match level {
        DebugLogLevel::Verbose => "VERBOSE",
        DebugLogLevel::Debug => "DEBUG",
        DebugLogLevel::Info => "INFO",
        DebugLogLevel::Warn => "WARN",
        DebugLogLevel::Error => "ERROR",
    };
    let output = format!("{} [{}] {}\n", timestamp, level_str, message.trim());

    if is_debug_to_stderr() {
        eprint!("{}", output);
        return;
    }

    // Write to debug file
    let path = get_debug_log_path();
    if let Some(parent) = std::path::Path::new(&path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut f| {
            use std::io::Write;
            f.write_all(output.as_bytes())
        });
}

pub fn get_debug_log_path() -> String {
    if let Some(path) = get_debug_file_path() {
        return path;
    }

    if let Ok(dir) = std::env::var(ai::CODE_DEBUG_LOGS_DIR) {
        return dir;
    }

    // Default path
    let config_home = std::env::var(ai::CONFIG_HOME)
        .or_else(|_| std::env::var(ai::CLAUDE_CONFIG_HOME))
        .or_else(|_| std::env::var(system::HOME).map(|h| format!("{}/.ai", h)))
        .unwrap_or_else(|_| "~/.ai".to_string());

    format!("{}/debug/debug.txt", config_home)
}

/// Logs errors for Ants only, always visible in production.
pub fn log_ant_error(context: &str, error: &str) {
    let user_type = std::env::var(ai::USER_TYPE).unwrap_or_default();
    if user_type != "ant" {
        return;
    }

    let message = format!("[ANT-ONLY] {} error: {}", context, error);
    log_for_debugging(&message, DebugLogLevel::Error);
}
