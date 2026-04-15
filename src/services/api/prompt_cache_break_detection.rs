// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/promptCacheBreakDetection.ts
//! Prompt cache break detection module
//! Detects when prompt caching breaks and logs analytics

use std::collections::HashMap;
use std::sync::Mutex;

use once_cell::sync::Lazy;

/// Maximum tracked sources to prevent unbounded memory growth
const MAX_TRACKED_SOURCES: usize = 10;

/// Minimum absolute token drop required to trigger a cache break warning
const MIN_CACHE_MISS_TOKENS: i64 = 2000;

/// Cache TTL thresholds in milliseconds
const CACHE_TTL_5MIN_MS: u64 = 5 * 60 * 1000;
pub const CACHE_TTL_1HOUR_MS: u64 = 60 * 60 * 1000;

/// Tracked source prefixes
const TRACKED_SOURCE_PREFIXES: &[&str] = &[
    "repl_main_thread",
    "sdk",
    "agent:custom",
    "agent:default",
    "agent:builtin",
];

/// Previous state for a tracked source
#[derive(Debug, Clone)]
pub struct PreviousState {
    pub system_hash: u64,
    pub tools_hash: u64,
    pub cache_control_hash: u64,
    pub tool_names: Vec<String>,
    pub per_tool_hashes: HashMap<String, u64>,
    pub system_char_count: usize,
    pub model: String,
    pub fast_mode: bool,
    pub global_cache_strategy: String,
    pub betas: Vec<String>,
    pub auto_mode_active: bool,
    pub is_using_overage: bool,
    pub cached_mc_enabled: bool,
    pub effort_value: String,
    pub extra_body_hash: u64,
    pub call_count: u32,
    pub pending_changes: Option<PendingChanges>,
    pub prev_cache_read_tokens: Option<i64>,
    pub cache_deletions_pending: bool,
    pub diffable_content: String,
}

/// Pending changes from previous state
#[derive(Debug, Clone)]
pub struct PendingChanges {
    pub system_prompt_changed: bool,
    pub tool_schemas_changed: bool,
    pub model_changed: bool,
    pub fast_mode_changed: bool,
    pub cache_control_changed: bool,
    pub global_cache_strategy_changed: bool,
    pub betas_changed: bool,
    pub auto_mode_changed: bool,
    pub overage_changed: bool,
    pub cached_mc_changed: bool,
    pub effort_changed: bool,
    pub extra_body_changed: bool,
    pub added_tool_count: usize,
    pub removed_tool_count: usize,
    pub system_char_delta: i64,
    pub added_tools: Vec<String>,
    pub removed_tools: Vec<String>,
    pub changed_tool_schemas: Vec<String>,
    pub previous_model: String,
    pub new_model: String,
    pub prev_global_cache_strategy: String,
    pub new_global_cache_strategy: String,
    pub added_betas: Vec<String>,
    pub removed_betas: Vec<String>,
    pub prev_effort_value: String,
    pub new_effort_value: String,
    pub prev_diffable_content: String,
}

/// Prompt state snapshot
#[derive(Debug, Clone)]
pub struct PromptStateSnapshot {
    pub system: Vec<serde_json::Value>,
    pub tool_schemas: Vec<serde_json::Value>,
    pub query_source: String,
    pub model: String,
    pub agent_id: Option<String>,
    pub fast_mode: Option<bool>,
    pub global_cache_strategy: Option<String>,
    pub betas: Option<Vec<String>>,
    pub auto_mode_active: Option<bool>,
    pub is_using_overage: Option<bool>,
    pub cached_mc_enabled: Option<bool>,
    pub effort_value: Option<String>,
    pub extra_body_params: Option<serde_json::Value>,
}

/// Previous state by source
static PREVIOUS_STATE_BY_SOURCE: Lazy<Mutex<HashMap<String, PreviousState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Check if model should be excluded from cache break detection
fn is_excluded_model(model: &str) -> bool {
    model.contains("haiku")
}

/// Get tracking key for a query source
pub fn get_tracking_key(query_source: &str, agent_id: Option<&str>) -> Option<String> {
    if query_source == "compact" {
        return Some("repl_main_thread".to_string());
    }

    for prefix in TRACKED_SOURCE_PREFIXES {
        if query_source.starts_with(prefix) {
            return Some(agent_id.map(String::from).unwrap_or_else(|| query_source.to_string()));
        }
    }

    None
}

/// Strip cache_control from items
fn strip_cache_control(items: &[serde_json::Value]) -> Vec<serde_json::Value> {
    items
        .iter()
        .map(|item| {
            if let Some(obj) = item.as_object() {
                if obj.contains_key("cache_control") {
                    let mut new_obj = obj.clone();
                    new_obj.remove("cache_control");
                    return serde_json::Value::Object(new_obj);
                }
            }
            item.clone()
        })
        .collect()
}

/// Compute hash of data (simplified djb2)
fn compute_hash(data: &str) -> u64 {
    let mut hash: u64 = 5381;
    for c in data.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(c as u64);
    }
    hash
}

/// Sanitize tool name
fn sanitize_tool_name(name: &str) -> String {
    if name.starts_with("mcp__") {
        "mcp".to_string()
    } else {
        name.to_string()
    }
}

/// Compute per-tool hashes
fn compute_per_tool_hashes(stripped_tools: &[serde_json::Value], names: &[String]) -> HashMap<String, u64> {
    let mut hashes = HashMap::new();
    for (i, tool) in stripped_tools.iter().enumerate() {
        let name = names.get(i).cloned().unwrap_or_else(|| format!("__idx_{}", i));
        let tool_str = serde_json::to_string(tool).unwrap_or_default();
        hashes.insert(name, compute_hash(&tool_str));
    }
    hashes
}

/// Get system character count
fn get_system_char_count(system: &[serde_json::Value]) -> usize {
    system
        .iter()
        .map(|b| b.get("text").and_then(|t| t.as_str()).map(|s| s.len()).unwrap_or(0))
        .sum()
}

/// Build diffable content
fn build_diffable_content(system: &[serde_json::Value], tools: &[serde_json::Value], model: &str) -> String {
    let system_text = system
        .iter()
        .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
        .collect::<Vec<_>>()
        .join("\n\n");

    let tool_details: Vec<String> = tools
        .iter()
        .map(|t| {
            let name = t.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
            let desc = t.get("description").and_then(|d| d.as_str()).unwrap_or("");
            let schema = t.get("input_schema").map(|s| serde_json::to_string(s).unwrap_or_default()).unwrap_or_default();
            format!("{}\n  description: {}\n  input_schema: {}", name, desc, schema)
        })
        .collect();

    format!(
        "Model: {}\n\n=== System Prompt ===\n\n{}\n\n=== Tools ({}) ===\n\n{}",
        model,
        system_text,
        tools.len(),
        tool_details.join("\n\n")
    )
}

/// Record prompt state (phase 1)
pub fn record_prompt_state(snapshot: PromptStateSnapshot) {
    let key = match get_tracking_key(&snapshot.query_source, snapshot.agent_id.as_deref()) {
        Some(k) => k,
        None => return,
    };

    let system = &snapshot.system;
    let tool_schemas = &snapshot.tool_schemas;
    let model = &snapshot.model;

    let stripped_system = strip_cache_control(system);
    let stripped_tools = strip_cache_control(tool_schemas);

    let system_hash = compute_hash(&serde_json::to_string(&stripped_system).unwrap_or_default());
    let tools_hash = compute_hash(&serde_json::to_string(&stripped_tools).unwrap_or_default());
    let cache_control_hash = compute_hash(&serde_json::to_string(
        &system.iter().map(|b| b.get("cache_control").cloned().unwrap_or(serde_json::Value::Null)).collect::<Vec<_>>()
    ).unwrap_or_default());

    let tool_names: Vec<String> = tool_schemas
        .iter()
        .map(|t| t.get("name").and_then(|n| n.as_str()).unwrap_or("unknown").to_string())
        .collect();

    let system_char_count = get_system_char_count(system);
    let diffable_content = build_diffable_content(system, tool_schemas, model);
    let is_fast_mode = snapshot.fast_mode.unwrap_or(false);
    let mut sorted_betas = snapshot.betas.clone().unwrap_or_default();
    sorted_betas.sort();
    let effort_str = snapshot.effort_value.clone().unwrap_or_default();
    let extra_body_hash = snapshot.extra_body_params
        .as_ref()
        .map(|p| compute_hash(&serde_json::to_string(p).unwrap_or_default()))
        .unwrap_or(0);

    let global_cache_strategy = snapshot.global_cache_strategy.clone().unwrap_or_default();

    let mut states = PREVIOUS_STATE_BY_SOURCE.lock().unwrap();

    if let Some(prev) = states.get_mut(&key) {
        prev.call_count += 1;

        let system_prompt_changed = system_hash != prev.system_hash;
        let tool_schemas_changed = tools_hash != prev.tools_hash;
        let model_changed = model != &prev.model;
        let fast_mode_changed = is_fast_mode != prev.fast_mode;
        let cache_control_changed = cache_control_hash != prev.cache_control_hash;
        let global_cache_strategy_changed = global_cache_strategy != prev.global_cache_strategy;
        let betas_changed = sorted_betas != prev.betas;
        let auto_mode_changed = snapshot.auto_mode_active.unwrap_or(false) != prev.auto_mode_active;
        let overage_changed = snapshot.is_using_overage.unwrap_or(false) != prev.is_using_overage;
        let cached_mc_changed = snapshot.cached_mc_enabled.unwrap_or(false) != prev.cached_mc_enabled;
        let effort_changed = effort_str != prev.effort_value;
        let extra_body_changed = extra_body_hash != prev.extra_body_hash;

        if system_prompt_changed || tool_schemas_changed || model_changed || fast_mode_changed
            || cache_control_changed || global_cache_strategy_changed || betas_changed
            || auto_mode_changed || overage_changed || cached_mc_changed || effort_changed
            || extra_body_changed
        {
            let prev_tool_set: std::collections::HashSet<_> = prev.tool_names.iter().collect();
            let new_tool_set: std::collections::HashSet<_> = tool_names.iter().collect();

            let added_tools: Vec<String> = tool_names.iter().filter(|n| !prev_tool_set.contains(n)).cloned().collect();
            let removed_tools: Vec<String> = prev.tool_names.iter().filter(|n| !new_tool_set.contains(n)).cloned().collect();

            let mut changed_tool_schemas = Vec::new();
            if tool_schemas_changed {
                let new_hashes = compute_per_tool_hashes(&stripped_tools, &tool_names);
                for name in &tool_names {
                    if prev_tool_set.contains(&name) {
                        if new_hashes.get(name) != prev.per_tool_hashes.get(name) {
                            changed_tool_schemas.push(name.clone());
                        }
                    }
                }
                prev.per_tool_hashes = new_hashes;
            }

            let prev_beta_set: std::collections::HashSet<_> = prev.betas.iter().collect();
            let new_beta_set: std::collections::HashSet<_> = sorted_betas.iter().collect();

            prev.pending_changes = Some(PendingChanges {
                system_prompt_changed,
                tool_schemas_changed,
                model_changed,
                fast_mode_changed,
                cache_control_changed,
                global_cache_strategy_changed,
                betas_changed,
                auto_mode_changed,
                overage_changed,
                cached_mc_changed,
                effort_changed,
                extra_body_changed,
                added_tool_count: added_tools.len(),
                removed_tool_count: removed_tools.len(),
                system_char_delta: system_char_count as i64 - prev.system_char_count as i64,
                added_tools,
                removed_tools,
                changed_tool_schemas,
                previous_model: prev.model.clone(),
                new_model: model.clone(),
                prev_global_cache_strategy: prev.global_cache_strategy.clone(),
                new_global_cache_strategy: global_cache_strategy.clone(),
                added_betas: sorted_betas.iter().filter(|b| !prev_beta_set.contains(b)).cloned().collect(),
                removed_betas: prev.betas.iter().filter(|b| !new_beta_set.contains(b)).cloned().collect(),
                prev_effort_value: prev.effort_value.clone(),
                new_effort_value: effort_str.clone(),
                prev_diffable_content: prev.diffable_content.clone(),
            });
        } else {
            prev.pending_changes = None;
        }

        prev.system_hash = system_hash;
        prev.tools_hash = tools_hash;
        prev.cache_control_hash = cache_control_hash;
        prev.tool_names = tool_names;
        prev.system_char_count = system_char_count;
        prev.model = model.clone();
        prev.fast_mode = is_fast_mode;
        prev.global_cache_strategy = global_cache_strategy;
        prev.betas = sorted_betas;
        prev.auto_mode_active = snapshot.auto_mode_active.unwrap_or(false);
        prev.is_using_overage = snapshot.is_using_overage.unwrap_or(false);
        prev.cached_mc_enabled = snapshot.cached_mc_enabled.unwrap_or(false);
        prev.effort_value = effort_str;
        prev.extra_body_hash = extra_body_hash;
        prev.diffable_content = diffable_content;
    } else {
        // Evict oldest entries if at capacity
        while states.len() >= MAX_TRACKED_SOURCES {
            if let Some(oldest) = states.keys().next().cloned() {
                states.remove(&oldest);
            }
        }

        states.insert(key, PreviousState {
            system_hash,
            tools_hash,
            cache_control_hash,
            tool_names: tool_names.clone(),
            per_tool_hashes: compute_per_tool_hashes(&stripped_tools, &tool_names),
            system_char_count,
            model: model.clone(),
            fast_mode: is_fast_mode,
            global_cache_strategy,
            betas: sorted_betas,
            auto_mode_active: snapshot.auto_mode_active.unwrap_or(false),
            is_using_overage: snapshot.is_using_overage.unwrap_or(false),
            cached_mc_enabled: snapshot.cached_mc_enabled.unwrap_or(false),
            effort_value: effort_str,
            extra_body_hash,
            call_count: 1,
            pending_changes: None,
            prev_cache_read_tokens: None,
            cache_deletions_pending: false,
            diffable_content,
        });
    }
}

/// Check response for cache break (phase 2)
pub async fn check_response_for_cache_break(
    query_source: &str,
    cache_read_tokens: i64,
    _cache_creation_tokens: i64,
    _messages: &[serde_json::Value],
    agent_id: Option<&str>,
    _request_id: Option<&str>,
) {
    let key = match get_tracking_key(query_source, agent_id) {
        Some(k) => k,
        None => return,
    };

    let mut states = PREVIOUS_STATE_BY_SOURCE.lock().unwrap();

    let state = match states.get_mut(&key) {
        Some(s) => s,
        None => return,
    };

    // Skip excluded models
    if is_excluded_model(&state.model) {
        return;
    }

    let prev_cache_read = state.prev_cache_read_tokens;
    state.prev_cache_read_tokens = Some(cache_read_tokens);

    // Skip first call
    if prev_cache_read.is_none() {
        return;
    }

    let prev_cache_read = prev_cache_read.unwrap();

    // Handle cache deletions (expected drop)
    if state.cache_deletions_pending {
        state.cache_deletions_pending = false;
        log::debug!("[PROMPT CACHE] cache deletion applied, cache read: {} → {} (expected drop)", prev_cache_read, cache_read_tokens);
        state.pending_changes = None;
        return;
    }

    // Detect cache break
    let token_drop = prev_cache_read - cache_read_tokens;
    if cache_read_tokens >= (prev_cache_read as f64 * 0.95) as i64 || token_drop < MIN_CACHE_MISS_TOKENS {
        state.pending_changes = None;
        return;
    }

    // Build explanation
    let mut parts = Vec::new();
    if let Some(ref changes) = state.pending_changes {
        if changes.model_changed {
            parts.push(format!("model changed ({} → {})", changes.previous_model, changes.new_model));
        }
        if changes.system_prompt_changed {
            let char_info = if changes.system_char_delta == 0 {
                String::new()
            } else if changes.system_char_delta > 0 {
                format!(" (+{} chars)", changes.system_char_delta)
            } else {
                format!(" ({} chars)", changes.system_char_delta)
            };
            parts.push(format!("system prompt changed{}", char_info));
        }
        if changes.tool_schemas_changed {
            let tool_diff = if changes.added_tool_count > 0 || changes.removed_tool_count > 0 {
                format!(" (+{}/-{} tools)", changes.added_tool_count, changes.removed_tool_count)
            } else {
                " (tool prompt/schema changed, same tool set)".to_string()
            };
            parts.push(format!("tools changed{}", tool_diff));
        }
        if changes.fast_mode_changed {
            parts.push("fast mode toggled".to_string());
        }
        if changes.global_cache_strategy_changed {
            parts.push(format!("global cache strategy changed ({} → {})",
                if changes.prev_global_cache_strategy.is_empty() { "none" } else { &changes.prev_global_cache_strategy },
                if changes.new_global_cache_strategy.is_empty() { "none" } else { &changes.new_global_cache_strategy }
            ));
        }
        if changes.cache_control_changed && !changes.global_cache_strategy_changed && !changes.system_prompt_changed {
            parts.push("cache_control changed (scope or TTL)".to_string());
        }
        if changes.betas_changed {
            let added = if !changes.added_betas.is_empty() { format!("+{}", changes.added_betas.join(",")) } else { String::new() };
            let removed = if !changes.removed_betas.is_empty() { format!("-{}", changes.removed_betas.join(",")) } else { String::new() };
            let diff = [added, removed].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ");
            parts.push(if diff.is_empty() { "betas changed".to_string() } else { format!("betas changed ({})", diff) });
        }
        if changes.auto_mode_changed {
            parts.push("auto mode toggled".to_string());
        }
        if changes.overage_changed {
            parts.push("overage state changed (TTL latched, no flip)".to_string());
        }
        if changes.cached_mc_changed {
            parts.push("cached microcompact toggled".to_string());
        }
        if changes.effort_changed {
            parts.push(format!("effort changed ({} → {})",
                if changes.prev_effort_value.is_empty() { "default" } else { &changes.prev_effort_value },
                if changes.new_effort_value.is_empty() { "default" } else { &changes.new_effort_value }
            ));
        }
        if changes.extra_body_changed {
            parts.push("extra body params changed".to_string());
        }
    }

    let reason = if !parts.is_empty() {
        parts.join(", ")
    } else {
        "unknown cause".to_string()
    };

    // Log the cache break event (simplified - would log to analytics in full impl)
    log::warn!("[PROMPT CACHE BREAK] {} [source={}, call #{}, cache read: {} → {}]",
        reason, query_source, state.call_count, prev_cache_read, cache_read_tokens);

    state.pending_changes = None;
}

/// Notify cache deletion (from microcompact)
pub fn notify_cache_deletion(query_source: &str, agent_id: Option<&str>) {
    let key = match get_tracking_key(query_source, agent_id) {
        Some(k) => k,
        None => return,
    };

    if let Some(state) = PREVIOUS_STATE_BY_SOURCE.lock().unwrap().get_mut(&key) {
        state.cache_deletions_pending = true;
    }
}

/// Notify compaction (reset baseline)
pub fn notify_compaction(query_source: &str, agent_id: Option<&str>) {
    let key = match get_tracking_key(query_source, agent_id) {
        Some(k) => k,
        None => return,
    };

    if let Some(state) = PREVIOUS_STATE_BY_SOURCE.lock().unwrap().get_mut(&key) {
        state.prev_cache_read_tokens = None;
    }
}

/// Cleanup agent tracking
pub fn cleanup_agent_tracking(agent_id: &str) {
    PREVIOUS_STATE_BY_SOURCE.lock().unwrap().remove(agent_id);
}

/// Reset all prompt cache break detection
pub fn reset_prompt_cache_break_detection() {
    PREVIOUS_STATE_BY_SOURCE.lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_excluded_model() {
        assert!(is_excluded_model("claude-3-5-haiku-20241022"));
        assert!(!is_excluded_model("claude-3-5-sonnet-20241022"));
    }

    #[test]
    fn test_get_tracking_key_compact() {
        let key = get_tracking_key("compact", None);
        assert_eq!(key, Some("repl_main_thread".to_string()));
    }

    #[test]
    fn test_get_tracking_key_repl() {
        let key = get_tracking_key("repl_main_thread", None);
        assert!(key.is_some());
    }

    #[test]
    fn test_sanitize_tool_name() {
        assert_eq!(sanitize_tool_name("mcp__server__tool"), "mcp");
        assert_eq!(sanitize_tool_name("my_tool"), "my_tool");
    }

    #[test]
    fn test_strip_cache_control() {
        let items = vec![
            serde_json::json!({"type": "text", "text": "hello", "cache_control": {"type": "ephemeral"}}),
            serde_json::json!({"type": "text", "text": "world"}),
        ];
        let stripped = strip_cache_control(&items);
        assert!(!stripped[0].as_object().unwrap().contains_key("cache_control"));
    }
}