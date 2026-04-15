//! Bridge pointer for crash-recovery.
//!
//! Translated from openclaudecode/openclaudecode/src/bridge/bridgePointer.ts
//!
//! Crash-recovery pointer for Remote Control sessions.
//!
//! Written immediately after a bridge session is created, periodically
//! refreshed during the session, and cleared on clean shutdown. If the
//! process dies unclean (crash, kill -9, terminal closed), the pointer
//! persists. On next startup, `claude remote-control` detects it and offers
//! to resume via the --session-id flow.
//!
//! Staleness is checked against the file's mtime (not an embedded timestamp)
//! so that a periodic re-write with the same content serves as a refresh.
//!
//! Scoped per working directory (alongside transcript JSONL files) so two
//! concurrent bridges in different repos don't clobber each other.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Upper bound on worktree fanout. git worktree list is naturally bounded
/// (50 is a LOT), but this caps the parallel stat() burst and guards against
/// pathological setups. Above this, --continue falls back to current-dir-only.
const MAX_WORKTREE_FANOUT: usize = 50;

/// Crash-recovery pointer TTL in milliseconds (4 hours)
pub const BRIDGE_POINTER_TTL_MS: u64 = 4 * 60 * 60 * 1000;

/// Bridge pointer source
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BridgePointerSource {
    Standalone,
    Repl,
}

/// Bridge pointer data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgePointer {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "environmentId")]
    pub environment_id: String,
    pub source: BridgePointerSource,
}

/// Bridge pointer with age information
#[derive(Debug, Clone)]
pub struct BridgePointerWithAge {
    pub session_id: String,
    pub environment_id: String,
    pub source: BridgePointerSource,
    pub age_ms: u64,
}

/// Get the bridge pointer path for a directory
pub fn get_bridge_pointer_path(dir: &str) -> PathBuf {
    // Get projects dir and sanitize the path
    let projects_dir = get_projects_dir();
    let sanitized = sanitize_path(dir);
    projects_dir.join(sanitized).join("bridge-pointer.json")
}

/// Get the projects directory (simplified implementation)
fn get_projects_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ai-code")
        .join("projects")
}

/// Sanitize a path for safe file system use
fn sanitize_path(path: &str) -> String {
    // Remove potentially dangerous characters
    path.chars()
        .map(|c| {
            if c == '/' || c == '\\' || c == ':' {
                '-'
            } else {
                c
            }
        })
        .collect()
}

/// Write the pointer. Also used to refresh mtime during long sessions —
/// calling with the same IDs is a cheap no-content-change write that bumps
/// the staleness clock. Best-effort — a crash-recovery file must never
/// itself cause a crash. Logs and swallows on error.
pub async fn write_bridge_pointer(dir: &str, pointer: &BridgePointer) -> Result<(), String> {
    let path = get_bridge_pointer_path(dir);

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // Serialize and write the pointer
    let content =
        serde_json::to_string_pretty(pointer).map_err(|e| format!("Failed to serialize: {}", e))?;

    fs::write(&path, content).map_err(|e| format!("Failed to write pointer: {}", e))?;

    log_for_debugging(&format!("[bridge:pointer] wrote {}", path.display()));

    Ok(())
}

/// Read the pointer and its age (ms since last write). Operates directly
/// and handles errors — no existence check. Returns None on any failure:
/// missing file, corrupted JSON, schema mismatch, or stale (mtime > 4h ago).
/// Stale/invalid pointers are deleted so they don't keep re-prompting after
/// the backend has already GC'd the env.
pub async fn read_bridge_pointer(dir: &str) -> Option<BridgePointerWithAge> {
    let path = get_bridge_pointer_path(dir);

    // Get file metadata for mtime
    let metadata = match fs::metadata(&path) {
        Ok(m) => m,
        Err(_) => return None,
    };

    let mtime_ms = metadata
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis() as u64;

    // Read the file content
    let raw = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return None,
    };

    // Parse the JSON
    let parsed: BridgePointer = match serde_json::from_str(&raw) {
        Ok(p) => p,
        Err(_) => {
            log_for_debugging(&format!(
                "[bridge:pointer] invalid schema, clearing: {}",
                path.display()
            ));
            let _ = clear_bridge_pointer(dir).await;
            return None;
        }
    };

    // Check staleness
    let age_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        - mtime_ms;

    if age_ms > BRIDGE_POINTER_TTL_MS {
        log_for_debugging(&format!(
            "[bridge:pointer] stale (>4h mtime), clearing: {}",
            path.display()
        ));
        let _ = clear_bridge_pointer(dir).await;
        return None;
    }

    Some(BridgePointerWithAge {
        session_id: parsed.session_id,
        environment_id: parsed.environment_id,
        source: parsed.source,
        age_ms,
    })
}

/// Worktree-aware read for `--continue`. The REPL bridge writes its pointer
/// to the original CWD which EnterWorktreeTool/activeWorktreeSession can
/// mutate to a worktree path — but `claude remote-control --continue` runs
/// with resolve('.') = shell CWD. This fans out across git worktree
/// siblings to find the freshest pointer, matching /resume's semantics.
///
/// Fast path: checks `dir` first. Only shells out to `git worktree list` if
/// that misses — the common case (pointer in launch dir) is one stat, zero
/// exec. Fanout reads run in parallel; capped at MAX_WORKTREE_FANOUT.
///
/// Returns the pointer AND the dir it was found in, so the caller can clear
/// the right file on resume failure.
pub async fn read_bridge_pointer_across_worktrees(
    dir: &str,
) -> Option<(BridgePointerWithAge, String)> {
    // Fast path: current dir. Covers standalone bridge (always matches) and
    // REPL bridge when no worktree mutation happened.
    if let Some(pointer) = read_bridge_pointer(dir).await {
        return Some((pointer, dir.to_string()));
    }

    // Fanout: scan worktree siblings
    let worktrees = get_worktree_paths(dir).await?;
    if worktrees.len() <= 1 {
        return None;
    }
    if worktrees.len() > MAX_WORKTREE_FANOUT {
        log_for_debugging(&format!(
            "[bridge:pointer] {} worktrees exceeds fanout cap {}, skipping",
            worktrees.len(),
            MAX_WORKTREE_FANOUT
        ));
        return None;
    }

    // Dedupe against `dir`
    let dir_key = sanitize_path(dir);
    let candidates: Vec<&String> = worktrees
        .iter()
        .filter(|wt| sanitize_path(wt) != dir_key)
        .collect();

    // Parallel stat+read
    let mut results: Vec<Option<(BridgePointerWithAge, String)>> = Vec::new();
    for wt in candidates {
        if let Some(p) = read_bridge_pointer(wt).await {
            results.push(Some((p, wt.clone())));
        }
    }

    // Pick freshest (lowest ageMs)
    let mut freshest: Option<(BridgePointerWithAge, String)> = None;
    for r in results.into_iter().flatten() {
        match &freshest {
            Some(f) if r.0.age_ms >= f.0.age_ms => {}
            _ => freshest = Some(r),
        }
    }

    if let Some(ref f) = freshest {
        log_for_debugging(&format!(
            "[bridge:pointer] fanout found pointer in worktree {} (ageMs={})",
            f.1, f.0.age_ms
        ));
    }

    freshest
}

/// Get worktree paths (simplified implementation)
async fn get_worktree_paths(dir: &str) -> Option<Vec<String>> {
    use std::process::Command;

    let output = Command::new("git")
        .args(&["worktree", "list", "--porcelain"])
        .current_dir(dir)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let paths: Vec<String> = output_str
        .lines()
        .filter_map(|line| {
            if line.starts_with("worktree ") {
                Some(line.trim_start_matches("worktree ").to_string())
            } else {
                None
            }
        })
        .collect();

    if paths.is_empty() {
        // Fallback: just return the directory itself
        Some(vec![dir.to_string()])
    } else {
        Some(paths)
    }
}

/// Delete the pointer. Idempotent — ENOENT is expected when the process
/// shut down clean previously.
pub async fn clear_bridge_pointer(dir: &str) -> Result<(), String> {
    let path = get_bridge_pointer_path(dir);

    match fs::remove_file(&path) {
        Ok(_) => {
            log_for_debugging(&format!("[bridge:pointer] cleared {}", path.display()));
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // Expected on clean shutdown
        Err(e) => {
            log_for_debugging(&format!("[bridge:pointer] clear failed: {}", e));
            Err(format!("Failed to clear pointer: {}", e))
        }
    }
}

/// Simple logging helper
#[allow(unused_variables)]
fn log_for_debugging(msg: &str) {
    eprintln!("{}", msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_pointer_serialization() {
        let pointer = BridgePointer {
            session_id: "test-session".to_string(),
            environment_id: "test-env".to_string(),
            source: BridgePointerSource::Standalone,
        };

        let json = serde_json::to_string(&pointer).unwrap();
        let parsed: BridgePointer = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.session_id, "test-session");
        assert_eq!(parsed.environment_id, "test-env");
    }

    #[test]
    fn test_sanitize_path() {
        assert_eq!(sanitize_path("foo/bar"), "foo-bar");
        assert_eq!(sanitize_path("foo\\bar"), "foo-bar");
    }
}
