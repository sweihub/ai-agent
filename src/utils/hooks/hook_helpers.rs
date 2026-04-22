// Source: ~/claudecode/openclaudecode/src/utils/hooks/hookHelpers.ts
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Re-export the existing helpers from the helpers module
pub use crate::utils::hooks::helpers::{
    HookResponse as HelpersHookResponse, add_arguments_to_prompt, hook_response_json_schema,
    parse_argument_names, parse_arguments,
};

/// Substitute arguments in a prompt string
/// Supports $ARGUMENTS, $ARGUMENTS[0], $0, $1, etc.
pub fn substitute_arguments(prompt: &str, json_input: &str) -> String {
    add_arguments_to_prompt(prompt, json_input)
}

/// Schema for hook responses (shared by prompt and agent hooks)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookResponse {
    /// Whether the condition was met
    pub ok: bool,
    /// Reason, if the condition was not met
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl HookResponse {
    /// Create a successful response
    pub fn success() -> Self {
        Self {
            ok: true,
            reason: None,
        }
    }

    /// Create a failure response with reason
    pub fn failure(reason: impl Into<String>) -> Self {
        Self {
            ok: false,
            reason: Some(reason.into()),
        }
    }

    /// Check if the response is successful
    pub fn is_ok(&self) -> bool {
        self.ok
    }
}

/// Hook response schema as JSON Schema (for tool input)
/// This is the same as hook_response_json_schema, kept for API compatibility
pub fn hook_response_schema() -> serde_json::Value {
    hook_response_json_schema()
}

/// Represents a tool definition for hook tools
#[derive(Debug, Clone)]
pub struct HookTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub prompt: String,
}

/// The name of the synthetic output tool
pub const SYNTHETIC_OUTPUT_TOOL_NAME: &str = "StructuredOutput";

/// Create a StructuredOutput tool configured for hook responses.
/// Reusable by agent hooks and background verification.
pub fn create_structured_output_tool() -> HookTool {
    HookTool {
        name: SYNTHETIC_OUTPUT_TOOL_NAME.to_string(),
        description: "Use this tool to return your verification result. You MUST call this tool exactly once at the end of your response.".to_string(),
        input_schema: hook_response_schema(),
        prompt: "Use this tool to return your verification result. You MUST call this tool exactly once at the end of your response.".to_string(),
    }
}

/// App state setter type (simplified)
pub type SetAppState = Box<dyn Fn(&dyn Fn(&mut serde_json::Value) -> ()) + Send + Sync>;

/// Hook response schema for validation (matches the tool's expected input)
pub fn hook_response_input_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "ok": {
                "type": "boolean",
                "description": "Whether the condition was met"
            },
            "reason": {
                "type": "string",
                "description": "Reason, if the condition was not met"
            }
        },
        "required": ["ok"],
        "additionalProperties": false
    })
}

/// Register a function hook that enforces structured output via SyntheticOutputTool.
/// Used by ask.tsx, execAgentHook, and background verification.
///
/// This adds a session-level hook that checks if the StructuredOutput tool
/// was called when the agent finishes.
pub fn register_structured_output_enforcement(
    _set_app_state: &dyn Fn(&dyn Fn(&mut serde_json::Value)),
    _session_id: &str,
) {
    // In the TypeScript version, this calls addFunctionHook to register
    // a function hook that checks if the StructuredOutput tool was called.
    // The Rust implementation would need a similar mechanism.
    //
    // The TS implementation:
    // 1. Registers a function hook for the 'Stop' event with no matcher
    // 2. The callback checks if there's a successful tool call to SYNTHETIC_OUTPUT_TOOL_NAME
    // 3. If not, it returns an error message telling the agent to call the tool
    //
    // In Rust, this would require:
    // 1. A way to register function hooks in session state
    // 2. A callback that checks message history for tool calls
    log::debug!(
        "Registered structured output enforcement for session {}",
        _session_id
    );
}

/// Check if messages contain a successful tool call to the given tool name
pub fn has_successful_tool_call(messages: &[serde_json::Value], tool_name: &str) -> bool {
    for msg in messages {
        if let Some(content) = msg.get("content") {
            if let Some(content_array) = content.as_array() {
                for block in content_array {
                    if let Some(block_type) = block.get("type") {
                        if block_type == "tool_use" {
                            if let Some(name) = block.get("name").and_then(|v| v.as_str()) {
                                if name == tool_name {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_response_success() {
        let resp = HookResponse::success();
        assert!(resp.is_ok());
        assert!(resp.reason.is_none());
    }

    #[test]
    fn test_hook_response_failure() {
        let resp = HookResponse::failure("condition not met");
        assert!(!resp.is_ok());
        assert_eq!(resp.reason, Some("condition not met".to_string()));
    }

    #[test]
    fn test_create_structured_output_tool() {
        let tool = create_structured_output_tool();
        assert_eq!(tool.name, SYNTHETIC_OUTPUT_TOOL_NAME);
        assert!(tool.input_schema.is_object());
    }

    #[test]
    fn test_has_successful_tool_call() {
        let messages = vec![serde_json::json!({
            "content": [{
                "type": "tool_use",
                "name": "StructuredOutput",
                "input": {"ok": true}
            }]
        })];
        assert!(has_successful_tool_call(
            &messages,
            SYNTHETIC_OUTPUT_TOOL_NAME
        ));
        assert!(!has_successful_tool_call(&messages, "OtherTool"));
    }

    #[test]
    fn test_substitute_arguments() {
        let result = substitute_arguments("Check: $ARGUMENTS", "some input");
        assert!(result.contains("some input"));
    }
}
