use serde_json::{json, Value};

pub fn validate_json_schema(data: &Value, schema: &Value) -> Result<(), String> {
    if let Some(schema_type) = schema.get("type") {
        match schema_type.as_str() {
            Some("object") => {
                if !data.is_object() {
                    return Err("Expected object".to_string());
                }
                if let Some(properties) = schema.get("properties") {
                    if let Some(obj) = data.as_object() {
                        for (key, value) in obj {
                            if let Some(prop_schema) = properties.get(key) {
                                validate_json_schema(value, prop_schema)?;
                            }
                        }
                    }
                }
            }
            Some("array") => {
                if !data.is_array() {
                    return Err("Expected array".to_string());
                }
                if let Some(items) = schema.get("items") {
                    if let Some(arr) = data.as_array() {
                        for item in arr {
                            validate_json_schema(item, items)?;
                        }
                    }
                }
            }
            Some("string") => {
                if !data.is_string() {
                    return Err("Expected string".to_string());
                }
            }
            Some("number") | Some("integer") => {
                if !data.is_number() {
                    return Err("Expected number".to_string());
                }
            }
            Some("boolean") => {
                if !data.is_boolean() {
                    return Err("Expected boolean".to_string());
                }
            }
            Some("null") => {
                if !data.is_null() {
                    return Err("Expected null".to_string());
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn generate_schema_for_type(value: &Value) -> Value {
    match value {
        Value::Null => json!({"type": "null"}),
        Value::Bool(_) => json!({"type": "boolean"}),
        Value::Number(_) => json!({"type": "number"}),
        Value::String(_) => json!({"type": "string"}),
        Value::Array(arr) => {
            if let Some(first) = arr.first() {
                json!({"type": "array", "items": generate_schema_for_type(first)})
            } else {
                json!({"type": "array"})
            }
        }
        Value::Object(obj) => {
            let properties: serde_json::Map<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), generate_schema_for_type(v)))
                .collect();
            json!({"type": "object", "properties": properties})
        }
    }
}
