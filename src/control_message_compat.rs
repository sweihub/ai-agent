use serde_json::Value;

pub fn normalize_control_message_keys(obj: &mut Value) {
    if let Value::Object(map) = obj {
        if let Some(request_id) = map.remove("requestId") {
            if !map.contains_key("request_id") {
                map.insert("request_id".to_string(), request_id);
            }
        }
        if let Some(response) = map.get_mut("response") {
            normalize_control_message_keys(response);
        }
    }
}
