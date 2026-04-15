// Source: ~/claudecode/openclaudecode/src/utils/toolSchemaCache.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Cached schema for a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSchema {
    /// The tool name.
    pub name: String,
    /// The tool description.
    pub description: String,
    /// The input schema as JSON.
    pub input_schema: serde_json::Value,
    /// Whether the tool is strict.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    /// Whether the tool supports eager input streaming.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eager_input_streaming: Option<bool>,
}

/// Session-scoped cache of rendered tool schemas. Tool schemas render at server
/// position 2 (before system prompt), so any byte-level change busts the entire
/// ~11K-token tool block AND everything downstream. Memoizing per-session locks
/// the schema bytes at first render.
static TOOL_SCHEMA_CACHE: LazyLock<std::sync::Mutex<HashMap<String, CachedSchema>>> =
    LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

/// Get the tool schema cache.
pub fn get_tool_schema_cache() -> std::sync::MutexGuard<'static, HashMap<String, CachedSchema>> {
    TOOL_SCHEMA_CACHE.lock().unwrap()
}

/// Clear the tool schema cache.
pub fn clear_tool_schema_cache() {
    TOOL_SCHEMA_CACHE.lock().unwrap().clear();
}

/// Get a cached schema by name.
pub fn get_cached_schema(tool_name: &str) -> Option<CachedSchema> {
    get_tool_schema_cache().get(tool_name).cloned()
}

/// Cache a schema.
pub fn cache_schema(tool_name: &str, schema: CachedSchema) {
    get_tool_schema_cache().insert(tool_name.to_string(), schema);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_and_get() {
        clear_tool_schema_cache();

        let schema = CachedSchema {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: serde_json::json!({}),
            strict: Some(true),
            eager_input_streaming: None,
        };

        cache_schema("test_tool", schema.clone());
        let cached = get_cached_schema("test_tool");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().name, "test_tool");
    }

    #[test]
    fn test_clear_cache() {
        cache_schema(
            "test_tool",
            CachedSchema {
                name: "test_tool".to_string(),
                description: "test".to_string(),
                input_schema: serde_json::json!({}),
                strict: None,
                eager_input_streaming: None,
            },
        );
        clear_tool_schema_cache();
        assert!(get_cached_schema("test_tool").is_none());
    }
}
