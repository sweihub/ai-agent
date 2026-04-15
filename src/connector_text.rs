//! Connector text block types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorTextBlock {
    #[serde(default)]
    pub text_type: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

pub fn is_connector_text_block(value: &serde_json::Value) -> bool {
    value.get("text").is_some()
}
