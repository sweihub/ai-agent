// Source: ~/claudecode/openclaudecode/src/utils/controlMessageCompat.ts

/// Normalize camelCase `requestId` to snake_case `request_id` on incoming
/// control messages (control_request, control_response).
///
/// Older iOS app builds send `requestId` due to a missing Swift CodingKeys
/// mapping. Without this shim, `is_sdk_control_request` in repl_bridge rejects
/// the message (it checks `'request_id' in value`), and structured_io reads
/// `message.response.request_id` as undefined -- both silently drop the message.
///
/// If both `request_id` and `requestId` are present, snake_case wins.
pub fn normalize_control_message_keys(obj: serde_json::Value) -> serde_json::Value {
    match obj {
        serde_json::Value::Object(mut map) => {
            // Check for requestId at top level
            if let Some(request_id) = map.remove("requestId") {
                if !map.contains_key("request_id") {
                    map.insert("request_id".to_string(), request_id);
                }
            }

            // Check for requestId in response
            if let Some(serde_json::Value::Object(response)) = map.get_mut("response") {
                if let Some(request_id) = response.remove("requestId") {
                    if !response.contains_key("request_id") {
                        response.insert("request_id".to_string(), request_id);
                    }
                }
            }

            serde_json::Value::Object(map)
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_request_id() {
        let input = serde_json::json!({
            "requestId": "abc123"
        });
        let output = normalize_control_message_keys(input);
        assert_eq!(output["request_id"], "abc123");
        assert!(output.get("requestId").is_none());
    }

    #[test]
    fn test_normalize_response_request_id() {
        let input = serde_json::json!({
            "response": {
                "requestId": "def456"
            }
        });
        let output = normalize_control_message_keys(input);
        assert_eq!(output["response"]["request_id"], "def456");
    }

    #[test]
    fn test_snake_case_wins() {
        let input = serde_json::json!({
            "request_id": "existing",
            "requestId": "new"
        });
        let output = normalize_control_message_keys(input);
        assert_eq!(output["request_id"], "existing");
    }

    #[test]
    fn test_non_object_unchanged() {
        let input = serde_json::json!("hello");
        let output = normalize_control_message_keys(input);
        assert_eq!(output, "hello");
    }
}
