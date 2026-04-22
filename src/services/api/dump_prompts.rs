// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/dumpPrompts.ts
//! Dump prompts module
//! Caches API requests for debugging and logs prompts to files

use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use sha2::{Digest, Sha256};

/// Maximum cached API requests
const MAX_CACHED_REQUESTS: usize = 5;

/// Cache last few API requests for ant users
static CACHED_API_REQUESTS: Lazy<Mutex<VecDeque<ApiRequestCacheEntry>>> =
    Lazy::new(|| Mutex::new(VecDeque::new()));

/// API request cache entry
#[derive(Debug, Clone)]
pub struct ApiRequestCacheEntry {
    pub timestamp: String,
    pub request: serde_json::Value,
}

/// Dump state per session
#[derive(Debug, Clone)]
pub struct DumpState {
    pub initialized: bool,
    pub message_count_seen: usize,
    pub last_init_data_hash: String,
    pub last_init_fingerprint: String,
}

impl Default for DumpState {
    fn default() -> Self {
        Self {
            initialized: false,
            message_count_seen: 0,
            last_init_data_hash: String::new(),
            last_init_fingerprint: String::new(),
        }
    }
}

/// Track state per session
static DUMP_STATES: Lazy<Mutex<std::collections::HashMap<String, DumpState>>> =
    Lazy::new(|| Mutex::new(std::collections::HashMap::new()));

/// Hash string using SHA256
fn hash_string(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Get last API requests
pub fn get_last_api_requests() -> Vec<ApiRequestCacheEntry> {
    let cache = CACHED_API_REQUESTS.lock().unwrap();
    cache.iter().cloned().collect()
}

/// Clear API request cache
pub fn clear_api_request_cache() {
    let mut cache = CACHED_API_REQUESTS.lock().unwrap();
    cache.clear();
}

/// Clear dump state for an agent/session
pub fn clear_dump_state(agent_id_or_session_id: &str) {
    let mut states = DUMP_STATES.lock().unwrap();
    states.remove(agent_id_or_session_id);
}

/// Clear all dump state
pub fn clear_all_dump_state() {
    let mut states = DUMP_STATES.lock().unwrap();
    states.clear();
}

/// Add API request to cache
pub fn add_api_request_to_cache(request_data: serde_json::Value) {
    // Only cache for ant users
    if std::env::var("AI_CODE_USER_TYPE")
        .map(|v| v == "ant")
        .unwrap_or(false)
    {
        let mut cache = CACHED_API_REQUESTS.lock().unwrap();
        cache.push_back(ApiRequestCacheEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            request: request_data,
        });
        if cache.len() > MAX_CACHED_REQUESTS {
            cache.pop_front();
        }
    }
}

/// Get config home directory
fn get_config_home_dir() -> PathBuf {
    // Use XDG_CONFIG_HOME or default to ~/.config
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .map(|p| p.join(".config"))
                .unwrap_or_else(|| PathBuf::from(".config"))
        })
        .join("claude")
}

/// Get dump prompts file path
pub fn get_dump_prompts_path(agent_id_or_session_id: Option<&str>) -> PathBuf {
    let session_id = agent_id_or_session_id.unwrap_or_else(|| {
        // Default to a session ID if not provided
        "default-session"
    });
    get_config_home_dir()
        .join("dump-prompts")
        .join(format!("{}.jsonl", session_id))
}

/// Initialize fingerprint from request
fn init_fingerprint(req: &serde_json::Value) -> String {
    let tools = req.get("tools").and_then(|t| t.as_array());
    let system = req.get("system");

    let sys_len = match system {
        Some(serde_json::Value::String(s)) => s.len(),
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .map(|b| {
                b.get("text")
                    .and_then(|t| t.as_str())
                    .map(|s| s.len())
                    .unwrap_or(0)
            })
            .sum(),
        _ => 0,
    };

    let tool_names = tools
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();

    let model = req.get("model").and_then(|m| m.as_str()).unwrap_or("");

    format!("{}|{}|{}", model, tool_names, sys_len)
}

/// Ensure directory exists
fn ensure_dir(path: &PathBuf) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Append entries to file
fn append_to_file(file_path: &PathBuf, entries: &[String]) -> std::io::Result<()> {
    if entries.is_empty() {
        return Ok(());
    }

    ensure_dir(file_path)?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;

    for entry in entries {
        writeln!(file, "{}", entry)?;
    }

    Ok(())
}

/// Dump request to file
fn dump_request(body: &str, ts: &str, state: &mut DumpState, file_path: &PathBuf) {
    // Try to parse the request body
    let Ok(req) = serde_json::from_str::<serde_json::Value>(body) else {
        return;
    };

    // Add to cache
    add_api_request_to_cache(req.clone());

    // Only dump for ant users
    if std::env::var("AI_CODE_USER_TYPE")
        .map(|v| v != "ant")
        .unwrap_or(true)
    {
        return;
    }

    let mut entries = Vec::new();
    let messages = req.get("messages").and_then(|m| m.as_array());

    // Write init data on first request or system_update when it changes
    let fingerprint = init_fingerprint(&req);
    if !state.initialized || fingerprint != state.last_init_fingerprint {
        // Extract init data (everything except messages)
        let mut init_data = req.clone();
        if let Some(obj) = init_data.as_object_mut() {
            obj.remove("messages");
        }

        let init_data_str = serde_json::to_string(&init_data).unwrap_or_default();
        let init_data_hash = hash_string(&init_data_str);
        state.last_init_fingerprint = fingerprint;

        if !state.initialized {
            state.initialized = true;
            state.last_init_data_hash = init_data_hash;
            entries.push(format!(
                r#"{{"type":"init","timestamp":"{}","data":{}}}"#,
                ts, init_data_str
            ));
        } else if init_data_hash != state.last_init_data_hash {
            state.last_init_data_hash = init_data_hash;
            entries.push(format!(
                r#"{{"type":"system_update","timestamp":"{}","data":{}}}"#,
                ts, init_data_str
            ));
        }
    }

    // Write only new user messages
    if let Some(msgs) = messages {
        for msg in msgs.iter().skip(state.message_count_seen) {
            if msg.get("role").and_then(|r| r.as_str()) == Some("user") {
                if let Ok(msg_str) = serde_json::to_string(msg) {
                    entries.push(format!(
                        r#"{{"type":"message","timestamp":"{}","data":{}}}"#,
                        ts, msg_str
                    ));
                }
            }
        }
        state.message_count_seen = msgs.len();
    }

    let _ = append_to_file(file_path, &entries);
}

/// Process API request for dump prompts
/// This is called from the fetch wrapper
pub fn process_dump_request(body: &str, agent_id_or_session_id: &str) -> Option<String> {
    let timestamp = chrono::Utc::now().to_rfc3339();
    let file_path = get_dump_prompts_path(Some(agent_id_or_session_id));

    // Get or create state for this session
    let mut states = DUMP_STATES.lock().unwrap();
    let state = states
        .entry(agent_id_or_session_id.to_string())
        .or_default();

    dump_request(body, &timestamp, state, &file_path);

    Some(timestamp)
}

/// Get dump state for a session
pub fn get_dump_state(agent_id_or_session_id: &str) -> DumpState {
    let states = DUMP_STATES.lock().unwrap();
    states
        .get(agent_id_or_session_id)
        .cloned()
        .unwrap_or_default()
}

/// Set dump state for a session
pub fn set_dump_state(agent_id_or_session_id: &str, state: DumpState) {
    let mut states = DUMP_STATES.lock().unwrap();
    states.insert(agent_id_or_session_id.to_string(), state);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_string() {
        let result = hash_string("test");
        assert_eq!(result.len(), 64);
    }

    #[test]
    fn test_get_dump_prompts_path() {
        let path = get_dump_prompts_path(Some("test-session"));
        assert!(path.to_string_lossy().contains("test-session.jsonl"));
    }

    #[test]
    fn test_init_fingerprint() {
        let req = serde_json::json!({
            "model": "claude-3-5-sonnet-20241022",
            "tools": [{"name": "tool1"}, {"name": "tool2"}],
            "system": "You are a helpful assistant"
        });
        let fp = init_fingerprint(&req);
        assert!(fp.contains("claude-3-5-sonnet-20241022"));
        assert!(fp.contains("tool1,tool2"));
    }
}
