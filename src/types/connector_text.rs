// Source: ~/claudecode/openclaudecode/src/types/connectorText.ts

use serde::{Deserialize, Serialize};

/// A connector text block that may contain arbitrary fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorTextBlock {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub block_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// Check if a value is a ConnectorTextBlock (has a "text" field).
pub fn is_connector_text_block(value: &serde_json::Value) -> bool {
    value.is_object() && value.get("text").is_some()
}
