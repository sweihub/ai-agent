use crate::types::message::AssistantMessage;
use crate::uuid::Uuid;
use serde::{Deserialize, Serialize};

/// Create a synthetic AssistantMessage for remote permission requests.
/// The ToolUseConfirm type requires an AssistantMessage, but in remote mode
/// we don't have a real one — the tool use runs on the CCR container.
pub fn create_synthetic_assistant_message(
    request: &SdkControlPermissionRequest,
    request_id: &str,
) -> AssistantMessage {
    let tool_use = ToolUseContent {
        id: request.tool_use_id.clone(),
        name: request.tool_name.clone(),
        input: request.input.clone(),
        #[cfg(feature = "input_schema")]
        input_schema: serde_json::Value::Object(serde_json::Map::new()),
    };

    let message = crate::types::message::Message {
        id: format!("remote-{}", request_id),
        message_type: "message".to_string(),
        role: "assistant".to_string(),
        content: vec![crate::types::message::ContentBlock::ToolUse(tool_use)],
        model: String::new(),
        stop_reason: None,
        stop_sequence: None,
        container: None,
        context_management: None,
        usage: crate::types::message::Usage {
            input_tokens: 0,
            output_tokens: 0,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
            web_search_requests: None,
            web_search_unit_price: None,
            web_searchcached_input_tokens: None,
            image_unit_price: None,
            image_cached_input_tokens: None,
        },
    };

    AssistantMessage {
        message,
        uuid: Uuid::new_v4(),
        request_id: None,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

/// SDK control permission request from remote
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SdkControlPermissionRequest {
    pub tool_use_id: String,
    pub tool_name: String,
    pub input: std::collections::HashMap<String, serde_json::Value>,
}

/// Create a minimal Tool stub for tools that aren't loaded locally.
/// This happens when the remote CCR has tools (e.g., MCP tools) that the
/// local CLI doesn't know about. The stub routes to FallbackPermissionRequest.
#[derive(Debug, Clone)]
pub struct ToolStub {
    pub name: String,
    pub input_schema: serde_json::Value,
    pub is_enabled: bool,
    pub is_mcp: bool,
    pub needs_permissions: bool,
    pub is_read_only: bool,
}

impl ToolStub {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            input_schema: serde_json::Value::Object(serde_json::Map::new()),
            is_enabled: true,
            is_mcp: false,
            needs_permissions: true,
            is_read_only: false,
        }
    }

    pub fn is_enabled_fn(&self) -> bool {
        self.is_enabled
    }

    pub fn user_facing_name(&self) -> String {
        self.name.clone()
    }

    pub fn render_tool_use_message(
        &self,
        input: &std::collections::HashMap<String, serde_json::Value>,
    ) -> String {
        if input.is_empty() {
            return String::new();
        }

        input
            .iter()
            .take(3)
            .map(|(key, value)| {
                let value_str = match value {
                    serde_json::Value::String(s) => s.clone(),
                    other => serde_json::to_string(other).unwrap_or_default(),
                };
                format!("{}: {}", key, value_str)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}
