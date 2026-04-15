use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SdkMessage {
    pub id: String,
    pub message_type: SdkMessageType,
    pub payload: HashMap<String, serde_json::Value>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SdkMessageType {
    Request,
    Response,
    Event,
    Error,
}

impl SdkMessage {
    pub fn new_request(id: &str, payload: HashMap<String, serde_json::Value>) -> Self {
        Self {
            id: id.to_string(),
            message_type: SdkMessageType::Request,
            payload,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn new_response(id: &str, payload: HashMap<String, serde_json::Value>) -> Self {
        Self {
            id: id.to_string(),
            message_type: SdkMessageType::Response,
            payload,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn new_event(id: &str, payload: HashMap<String, serde_json::Value>) -> Self {
        Self {
            id: id.to_string(),
            message_type: SdkMessageType::Event,
            payload,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn new_error(id: &str, error: &str) -> Self {
        let mut payload = HashMap::new();
        payload.insert(
            "error".to_string(),
            serde_json::Value::String(error.to_string()),
        );
        Self {
            id: id.to_string(),
            message_type: SdkMessageType::Error,
            payload,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}
