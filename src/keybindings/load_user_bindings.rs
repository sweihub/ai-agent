//! Load user bindings

use std::collections::HashMap;
use std::path::Path;

pub fn load_user_bindings(
    path: &Path,
) -> std::io::Result<HashMap<String, HashMap<String, String>>> {
    let content = std::fs::read_to_string(path)?;
    let parsed: serde_json::Value =
        serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);

    let mut bindings = HashMap::new();

    if let Some(arr) = parsed.as_array() {
        for block in arr {
            if let (Some(context), Some(bindings_obj)) = (
                block.get("context").and_then(|v| v.as_str()),
                block.get("bindings").and_then(|v| v.as_object()),
            ) {
                let mut context_bindings = HashMap::new();
                for (key, value) in bindings_obj {
                    if let Some(action) = value.as_str() {
                        context_bindings.insert(key.clone(), action.to_string());
                    }
                }
                bindings.insert(context.to_string(), context_bindings);
            }
        }
    }

    Ok(bindings)
}
