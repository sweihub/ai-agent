#![allow(dead_code)]

pub fn serialize_to_ndjson<T: serde::Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

pub fn serialize_to_ndjson_safe(value: &serde_json::Value) -> String {
    serde_json::to_string(value).unwrap_or_default()
}
