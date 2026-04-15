// Source: ~/claudecode/openclaudecode/src/utils/zodToJsonSchema.rs

use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

/// JSON Schema type alias.
pub type JsonSchemaType = Value;

/// Session-scoped cache of rendered schemas. Tool schemas are wrapped with
/// lazy_schema() which guarantees the same reference per session, so we can
/// cache by identity. In Rust, we cache by the schema's debug representation
/// since we can't use pointer identity for arbitrary values.
static SCHEMA_CACHE: LazyLock<Mutex<HashMap<String, JsonSchemaType>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Convert a JSON Schema to JSON Schema format (identity for already-JSON values).
///
/// Converts a Zod v4 schema to JSON Schema format.
/// In Rust, since we don't have Zod, this acts as an identity function for
/// JSON Schema values with caching.
pub fn json_to_json_schema(schema: &JsonSchemaType) -> JsonSchemaType {
    // Use debug representation as cache key
    let cache_key = format!("{:?}", schema);

    let mut cache = SCHEMA_CACHE.lock().unwrap();
    if let Some(cached) = cache.get(&cache_key) {
        return cached.clone();
    }

    // Identity for JSON values (already in JSON Schema format)
    let result = schema.clone();
    cache.insert(cache_key, result.clone());
    result
}

/// Clear the schema cache.
pub fn clear_schema_cache() {
    SCHEMA_CACHE.lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_to_json_schema_identity() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            }
        });

        let result = json_to_json_schema(&schema);
        assert_eq!(result, schema);
    }

    #[test]
    fn test_clear_cache() {
        clear_schema_cache();
        // Should not panic
    }
}
