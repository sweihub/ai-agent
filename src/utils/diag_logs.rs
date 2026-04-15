//! Diagnostic logging utilities
//!
//! Logs diagnostic information to a logfile. This information is sent
//! via the environment manager to session-ingress to monitor issues from
//! within the container.
//!
//! *Important* - this function MUST NOT be called with any PII, including
//! file paths, project names, repo names, prompts, etc.

use crate::constants::env::ai_code;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// Diagnostic log level
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// A diagnostic log entry
#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticLogEntry {
    pub timestamp: String,
    pub level: DiagnosticLogLevel,
    pub event: String,
    pub data: std::collections::HashMap<String, serde_json::Value>,
}

/// Get the diagnostic log file path from environment
fn get_diagnostic_log_file() -> Option<String> {
    std::env::var(ai_code::DIAGNOSTICS_FILE).ok()
}

/// Logs diagnostic information to a logfile. This information is sent
/// via the environment manager to session-ingress to monitor issues from
/// within the container.
///
/// *Important* - this function MUST NOT be called with any PII, including
/// file paths, project names, repo names, prompts, etc.
///
/// # Arguments
/// * `level` - Log level. Only used for information, not filtering
/// * `event` - A specific event: "started", "mcp_connected", etc.
/// * `data` - Optional additional data to log
pub fn log_for_diagnostics_no_pii(
    level: DiagnosticLogLevel,
    event: &str,
    data: Option<std::collections::HashMap<String, serde_json::Value>>,
) {
    let log_file = match get_diagnostic_log_file() {
        Some(path) => path,
        None => return,
    };

    let entry = DiagnosticLogEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        level,
        event: event.to_string(),
        data: data.unwrap_or_default(),
    };

    let line = serde_json::to_string(&entry).unwrap_or_default();
    let line = format!("{}\n", line);

    // Try to append first
    if let Ok(file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
    {
        if file.write_all(line.as_bytes()).is_ok() {
            return;
        }
    }

    // If append fails, try creating the directory first
    if let Some(parent) = Path::new(&log_file).parent() {
        let _ = fs::create_dir_all(parent);
        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .and_then(|mut file| file.write_all(line.as_bytes()));
    }
}

/// Wraps an async function with diagnostic timing logs.
/// Logs `{event}_started` before execution and `{event}_completed` after with duration_ms.
///
/// # Arguments
/// * `event` - Event name prefix (e.g., "git_status" -> logs "git_status_started" and "git_status_completed")
/// * `fut` - Async function to execute and time
/// * `get_data` - Optional function to extract additional data from the result for the completion log
/// # Returns
/// The result of the wrapped function
pub async fn with_diagnostics_timing<T, F, R>(
    event: &str,
    fut: F,
    get_data: Option<R>,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    F: Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    R: FnOnce(&T) -> std::collections::HashMap<String, serde_json::Value>,
{
    let start_time = std::time::Instant::now();
    log_for_diagnostics_no_pii(DiagnosticLogLevel::Info, &format!("{}_started", event), None);

    match fut.await {
        Ok(result) => {
            let mut additional_data = std::collections::HashMap::new();
            additional_data.insert(
                "duration_ms".to_string(),
                serde_json::json!(start_time.elapsed().as_millis() as u64),
            );
            
            if let Some(ref get_data_fn) = get_data {
                additional_data.extend(get_data_fn(&result));
            }
            
            log_for_diagnostics_no_pii(
                DiagnosticLogLevel::Info,
                &format!("{}_completed", event),
                Some(additional_data),
            );
            Ok(result)
        }
        Err(error) => {
            let mut data = std::collections::HashMap::new();
            data.insert(
                "duration_ms".to_string(),
                serde_json::json!(start_time.elapsed().as_millis() as u64),
            );
            log_for_diagnostics_no_pii(
                DiagnosticLogLevel::Error,
                &format!("{}_failed", event),
                Some(data),
            );
            Err(error)
        }
    }
}
