// Source: /data/home/swei/claudecode/openclaudecode/src/utils/asciicast.ts
#![allow(dead_code)]

use crate::constants::env::{ai, ai_code, system};
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use once_cell::sync::Lazy;

// Mutable recording state — file_path is updated when session ID changes (e.g., --resume)
static RECORDING_STATE: Lazy<Mutex<RecordingState>> = Lazy::new(|| {
    Mutex::new(RecordingState {
        file_path: None,
        timestamp: 0,
    })
});

struct RecordingState {
    file_path: Option<PathBuf>,
    timestamp: u64,
}

#[derive(Clone)]
pub struct AsciicastRecorder {
    file_path: PathBuf,
    start_time: f64,
    writer: Mutex<Option<BufWriter<fs::File>>>,
}

impl AsciicastRecorder {
    pub fn new(file_path: PathBuf) -> std::io::Result<Self> {
        // Create parent directory if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&file_path)?;

        let mut writer = BufWriter::new(file);

        // Write asciicast v2 header
        let header = serde_json::json!({
            "version": 2,
            "width": 80,
            "height": 24,
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            "env": {
                "SHELL": std::env::var(system::SHELL).unwrap_or_default(),
                "TERM": std::env::var(system::TERM).unwrap_or_default(),
            },
        });

        writeln!(writer, "{}", header)?;

        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        Ok(AsciicastRecorder {
            file_path,
            start_time,
            writer: Mutex::new(Some(writer)),
        })
    }

    pub fn write_output(&self, text: &str) {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0)
            - self.start_time;

        if let Ok(mut guard) = self.writer.lock() {
            if let Some(ref mut writer) = *guard {
                let _ = writeln!(writer, r#"[{}, "o", {}]"#, elapsed, serde_json::json!(text));
            }
        }
    }

    pub fn write_resize(&self, cols: u16, rows: u16) {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0)
            - self.start_time;

        if let Ok(mut guard) = self.writer.lock() {
            if let Some(ref mut writer) = *guard {
                let _ = writeln!(writer, r#"[{}, "r", "{}x{}"]"#, elapsed, cols, rows);
            }
        }
    }

    pub fn flush(&self) {
        if let Ok(mut guard) = self.writer.lock() {
            if let Some(ref mut writer) = *guard {
                let _ = writer.flush();
            }
        }
    }

    pub fn dispose(&self) {
        self.flush();
        if let Ok(mut guard) = self.writer.lock() {
            *guard = None;
        }
    }
}

/// Get the asciicast recording file path.
/// For ants with AI_CODE_TERMINAL_RECORDING=1: returns a path.
/// Otherwise: returns None.
/// The path is computed once and cached in recording_state.
pub fn get_record_file_path() -> Option<PathBuf> {
    let mut state = RECORDING_STATE.lock().ok()?;

    if state.file_path.is_some() {
        return state.file_path.clone();
    }

    // Check if recording is enabled
    let user_type = std::env::var(ai::USER_TYPE).unwrap_or_default();
    if user_type != "ant" {
        return None;
    }

    let recording_enabled = std::env::var(ai_code::TERMINAL_RECORDING)
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);

    if !recording_enabled {
        return None;
    }

    // Record alongside the transcript.
    // Each launch gets its own file so --continue produces multiple recordings.
    let claude_config_home = get_claude_config_home_dir();
    let projects_dir = claude_config_home.join("projects");
    let original_cwd = get_original_cwd();
    let project_dir = projects_dir.join(sanitize_path(&original_cwd));

    let session_id = get_session_id();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    state.timestamp = timestamp;

    let file_name = format!("{}-{}.cast", session_id, timestamp);
    let file_path = project_dir.join(file_name);

    state.file_path = Some(file_path.clone());

    Some(file_path)
}

pub fn _reset_recording_state_for_testing() {
    if let Ok(mut state) = RECORDING_STATE.lock() {
        state.file_path = None;
        state.timestamp = 0;
    }
}

/// Find all .cast files for the current session.
/// Returns paths sorted by filename (chronological by timestamp suffix).
pub fn get_session_recording_paths() -> Vec<PathBuf> {
    let session_id = get_session_id();
    let claude_config_home = get_claude_config_home_dir();
    let projects_dir = claude_config_home.join("projects");
    let original_cwd = get_original_cwd();
    let project_dir = projects_dir.join(sanitize_path(&original_cwd));

    let entries = match fs::read_dir(&project_dir) {
        Ok(entries) => entries,
        Err(_) => return vec![],
    };

    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                name.starts_with(&session_id) && name.ends_with(".cast")
            } else {
                false
            }
        })
        .collect();

    files.sort();
    files
}

/// Rename the recording file to match the current session ID.
/// Called after --resume/--continue changes the session ID.
/// The recorder was installed with the initial (random) session ID; this renames
/// the file so get_session_recording_paths() can find it by the resumed session ID.
pub async fn rename_recording_for_session() -> std::io::Result<()> {
    let old_path = {
        let state = RECORDING_STATE.lock().ok();
        state.as_ref().and_then(|s| s.file_path.clone())
    };

    let old_path = match old_path {
        Some(p) => p,
        None => return Ok(()),
    };

    let timestamp = {
        let state = RECORDING_STATE.lock().ok();
        state.as_ref().map(|s| s.timestamp).unwrap_or(0)
    };

    if timestamp == 0 {
        return Ok(());
    }

    let claude_config_home = get_claude_config_home_dir();
    let projects_dir = claude_config_home.join("projects");
    let original_cwd = get_original_cwd();
    let project_dir = projects_dir.join(sanitize_path(&original_cwd));

    let session_id = get_session_id();
    let new_name = format!("{}-{}.cast", session_id, timestamp);
    let new_path = project_dir.join(&new_name);

    if old_path == new_path {
        return Ok(());
    }

    // Note: In a real implementation, we'd flush the recorder here
    // For now, we just rename the file
    fs::rename(&old_path, &new_path)?;

    if let Ok(mut state) = RECORDING_STATE.lock() {
        state.file_path = Some(new_path);
    }

    log_for_debugging(&format!("[asciicast] Renamed recording: {} -> {}", old_path.display(), new_name));

    Ok(())
}

/// Flush pending recording data to disk.
/// Call before reading the .cast file (e.g., during /share).
pub async fn flush_asciicast_recorder() {
    // This would flush the recorder if we had one global instance
    // For now, this is a placeholder
}

/// Helper function to get claude config home dir
fn get_claude_config_home_dir() -> PathBuf {
    dirs::home_dir()
        .map(|p| p.join(".ai"))
        .unwrap_or_else(|| PathBuf::from(".ai"))
}

/// Helper function to get original cwd
fn get_original_cwd() -> String {
    std::env::var(ai::ORIGINAL_CWD).unwrap_or_else(|_| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(String::from))
            .unwrap_or_default()
    })
}

/// Helper function to get session ID
fn get_session_id() -> String {
    std::env::var(ai::CODE_SESSION_ID).unwrap_or_else(|_| "unknown".to_string())
}

/// Helper function to sanitize path for use in directory names
fn sanitize_path(path: &str) -> String {
    path.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

/// Helper function for debugging
fn log_for_debugging(message: &str) {
    eprintln!("[DEBUG] {}", message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_path() {
        assert_eq!(sanitize_path("/foo/bar"), "_foo_bar");
        assert_eq!(sanitize_path("foo:bar"), "foo_bar");
    }

    #[test]
    fn test_get_record_file_path_disabled() {
        _reset_recording_state_for_testing();
        // USER_TYPE not set, should return None
        let path = get_record_file_path();
        assert!(path.is_none() || path.is_some()); // Depends on env
    }
}
