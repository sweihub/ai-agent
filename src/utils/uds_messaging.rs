use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdsMessage {
    pub id: String,
    pub msg_type: MessageType,
    pub payload: serde_json::Value,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Request,
    Response,
    Event,
    Error,
}

impl UdsMessage {
    pub fn new(id: String, msg_type: MessageType, payload: serde_json::Value) -> Self {
        Self {
            id,
            msg_type,
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn request(id: String, payload: serde_json::Value) -> Self {
        Self::new(id, MessageType::Request, payload)
    }

    pub fn response(id: String, payload: serde_json::Value) -> Self {
        Self::new(id, MessageType::Response, payload)
    }

    pub fn error(id: String, message: String) -> Self {
        Self::new(
            id,
            MessageType::Error,
            serde_json::json!({ "message": message }),
        )
    }

    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn deserialize(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = UdsMessage::request("1".to_string(), serde_json::json!({ "action": "ping" }));
        let serialized = msg.serialize().unwrap();
        let deserialized = UdsMessage::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.id, "1");
    }
}
