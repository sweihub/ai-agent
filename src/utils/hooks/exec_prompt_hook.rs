// Source: ~/claudecode/openclaudecode/src/utils/hooks/execPromptHook.ts
#![allow(dead_code)]

use std::sync::Arc;
use uuid::Uuid;

use crate::types::Message;
use crate::utils::hooks::hook_helpers::{add_arguments_to_prompt, HookResponse};

/// Result of a hook execution
pub enum HookResult {
    Success {
        hook_name: String,
        hook_event: String,
        tool_use_id: String,
    },
    Blocking {
        blocking_error: String,
        command: String,
        prevent_continuation: bool,
        stop_reason: String,
    },
    Cancelled,
    NonBlockingError {
        hook_name: String,
        hook_event: String,
        tool_use_id: String,
        stderr: String,
        stdout: String,
        exit_code: i32,
    },
}

/// Represents a prompt hook configuration
pub struct PromptHook {
    /// The prompt to send to the model
    pub prompt: String,
    /// Optional timeout in seconds
    pub timeout: Option<u64>,
    /// Optional model override
    pub model: Option<String>,
}

/// Execute a prompt-based hook using an LLM
pub async fn exec_prompt_hook(
    hook: &PromptHook,
    hook_name: &str,
    hook_event: &str,
    json_input: &str,
    _signal: tokio::sync::watch::Receiver<bool>,
    tool_use_context: Arc<crate::utils::hooks::can_use_tool::ToolUseContext>,
    messages: Option<&[Message]>,
    tool_use_id: Option<String>,
) -> HookResult {
    // Use provided tool_use_id or generate a new one
    let effective_tool_use_id = tool_use_id.unwrap_or_else(|| format!("hook-{}", Uuid::new_v4()));

    // Replace $ARGUMENTS with the JSON input
    let processed_prompt = add_arguments_to_prompt(&hook.prompt, json_input);
    log_for_debugging(&format!(
        "Hooks: Processing prompt hook with prompt: {}",
        processed_prompt.chars().take(200).collect::<String>()
    ));

    // Create user message directly
    let user_message = create_user_message(&processed_prompt);

    // Prepend conversation history if provided
    let messages_to_query: Vec<serde_json::Value> = if let Some(msgs) = messages {
        let mut msg_vec: Vec<serde_json::Value> = msgs
            .iter()
            .map(|m| message_to_json(m))
            .collect();
        msg_vec.push(message_to_json_user(&user_message));
        msg_vec
    } else {
        vec![message_to_json_user(&user_message)]
    };

    log_for_debugging(&format!(
        "Hooks: Querying model with {} messages",
        messages_to_query.len()
    ));

    // Query the model with a small fast model
    let hook_timeout_ms = hook.timeout.map_or(30_000, |t| t * 1000);

    // Create abort channel
    let (abort_tx, abort_rx) = tokio::sync::watch::channel(false);

    // Setup timeout
    let timeout_handle = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(hook_timeout_ms)).await;
        let _ = abort_tx.send(true);
    });

    // Build the query
    let model = hook.model.clone().unwrap_or_else(get_small_fast_model);
    let system_prompt = r#"You are evaluating a hook in Claude Code.

Your response must be a JSON object matching one of the following schemas:
1. If the condition is met, return: {"ok": true}
2. If the condition is not met, return: {"ok": false, "reason": "Reason for why it is not met}"#;

    // Make the API call
    let response = query_model_without_streaming(
        &messages_to_query,
        system_prompt,
        &model,
        &tool_use_context,
    )
    .await;

    timeout_handle.abort();

    // Check if aborted
    if *abort_rx.borrow() {
        return HookResult::Cancelled;
    }

    match response {
        Ok(content) => {
            // Update response length for spinner display (not applicable in Rust)
            let full_response = content.trim();
            log_for_debugging(&format!("Hooks: Model response: {}", full_response));

            // Parse JSON response
            let json = match serde_json::from_str::<serde_json::Value>(full_response) {
                Ok(j) => j,
                Err(_) => {
                    log_for_debugging(&format!(
                        "Hooks: error parsing response as JSON: {}",
                        full_response
                    ));
                    return HookResult::NonBlockingError {
                        hook_name: hook_name.to_string(),
                        hook_event: hook_event.to_string(),
                        tool_use_id: effective_tool_use_id,
                        stderr: "JSON validation failed".to_string(),
                        stdout: full_response.to_string(),
                        exit_code: 1,
                    };
                }
            };

            // Validate against hook response schema
            let parsed = serde_json::from_value::<HookResponse>(json.clone());
            match parsed {
                Ok(hook_resp) => {
                    // Failed to meet condition
                    if !hook_resp.ok {
                        let reason = hook_resp.reason.unwrap_or_default();
                        log_for_debugging(&format!(
                            "Hooks: Prompt hook condition was not met: {}",
                            reason
                        ));
                        return HookResult::Blocking {
                            blocking_error: format!(
                                "Prompt hook condition was not met: {}",
                                reason
                            ),
                            command: hook.prompt.clone(),
                            prevent_continuation: true,
                            stop_reason: reason,
                        };
                    }

                    // Condition was met
                    log_for_debugging("Hooks: Prompt hook condition was met");
                    return HookResult::Success {
                        hook_name: hook_name.to_string(),
                        hook_event: hook_event.to_string(),
                        tool_use_id: effective_tool_use_id,
                    };
                }
                Err(err) => {
                    log_for_debugging(&format!(
                        "Hooks: model response does not conform to expected schema: {}",
                        err
                    ));
                    return HookResult::NonBlockingError {
                        hook_name: hook_name.to_string(),
                        hook_event: hook_event.to_string(),
                        tool_use_id: effective_tool_use_id,
                        stderr: format!("Schema validation failed: {}", err),
                        stdout: full_response.to_string(),
                        exit_code: 1,
                    };
                }
            }
        }
        Err(e) => {
            log_for_debugging(&format!("Hooks: Prompt hook error: {}", e));
            return HookResult::NonBlockingError {
                hook_name: hook_name.to_string(),
                hook_event: hook_event.to_string(),
                tool_use_id: effective_tool_use_id,
                stderr: format!("Error executing prompt hook: {}", e),
                stdout: String::new(),
                exit_code: 1,
            };
        }
    }
}

/// Create a user message with the given content
fn create_user_message(content: &str) -> Message {
    Message {
        role: crate::types::api_types::MessageRole::User,
        content: content.to_string(),
        attachments: None,
        tool_call_id: None,
        tool_calls: None,
        is_error: None,
        is_meta: None,
    }
}

/// Convert Message to JSON value
fn message_to_json(_msg: &Message) -> serde_json::Value {
    // Simplified conversion
    serde_json::json!({
        "type": "user",
        "message": { "content": "..." }
    })
}

/// Convert user message struct to JSON value
fn message_to_json_user(_msg: &Message) -> serde_json::Value {
    serde_json::json!({
        "type": "user",
        "message": { "content": "..." }
    })
}

/// Get the small fast model (simplified)
fn get_small_fast_model() -> String {
    "claude-3-haiku-20240307".to_string()
}

/// Query model without streaming (simplified)
async fn query_model_without_streaming(
    _messages: &[serde_json::Value],
    system_prompt: &str,
    _model: &str,
    _tool_use_context: &crate::utils::hooks::can_use_tool::ToolUseContext,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // This would call the actual API query function
    // For now, return an error indicating this is not implemented
    Err(format!(
        "query_model_without_streaming not fully implemented in port. System prompt: {}",
        system_prompt.chars().take(100).collect::<String>()
    )
    .into())
}

/// Log for debugging
fn log_for_debugging(msg: &str) {
    log::debug!("{}", msg);
}
