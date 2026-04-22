// Source: ~/claudecode/openclaudecode/src/utils/hooks/execAgentHook.ts
#![allow(dead_code)]

use std::sync::Arc;
use uuid::Uuid;

use crate::types::Message;
use crate::utils::hooks::helpers::{add_arguments_to_prompt, hook_response_json_schema};
use crate::utils::hooks::session_hooks::clear_session_hooks;

/// Maximum number of turns for an agent hook
const MAX_AGENT_TURNS: usize = 50;

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

/// Represents an agent hook configuration
pub struct AgentHook {
    /// The prompt to send to the agent
    pub prompt: String,
    /// Optional timeout in seconds
    pub timeout: Option<u64>,
    /// Optional model override
    pub model: Option<String>,
}

/// Execute an agent-based hook using a multi-turn LLM query
pub async fn exec_agent_hook(
    hook: &AgentHook,
    hook_name: &str,
    hook_event: &str,
    json_input: &str,
    signal: tokio::sync::watch::Receiver<bool>,
    tool_use_context: Arc<crate::utils::hooks::can_use_tool::ToolUseContext>,
    tool_use_id: Option<String>,
    _messages: &[Message],
    agent_name: Option<&str>,
) -> HookResult {
    let effective_tool_use_id = tool_use_id.unwrap_or_else(|| format!("hook-{}", Uuid::new_v4()));

    // Get transcript path from context
    let transcript_path = format!("session_{}_transcript.json", tool_use_context.session_id);

    let hook_start = std::time::Instant::now();

    // Replace $ARGUMENTS with the JSON input
    let processed_prompt = add_arguments_to_prompt(&hook.prompt, json_input);
    log_for_debugging(&format!(
        "Hooks: Processing agent hook with prompt: {}",
        processed_prompt.chars().take(200).collect::<String>()
    ));

    // Create user message
    let user_message = create_user_message(&processed_prompt);
    let mut agent_messages = vec![user_message];

    log_for_debugging(&format!(
        "Hooks: Starting agent query with {} messages",
        agent_messages.len()
    ));

    // Setup timeout
    let hook_timeout_ms = hook.timeout.map_or(60_000, |t| t * 1000);

    // Create abort controller
    let (abort_tx, abort_rx) = tokio::sync::watch::channel(false);

    // Combine parent signal with timeout
    let abort_tx_clone = abort_tx.clone();
    let timeout_handle = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(hook_timeout_ms)).await;
        let _ = abort_tx_clone.send(true);
    });

    // Get model
    let model = hook.model.clone().unwrap_or_else(get_small_fast_model);

    // Create unique agent ID for this hook agent
    let hook_agent_id = format!("hook-agent-{}", Uuid::new_v4());

    // Create a modified tool use context for the agent
    let agent_tool_use_context = Arc::new(crate::utils::hooks::can_use_tool::ToolUseContext {
        session_id: format!("{}-{}", tool_use_context.session_id, hook_agent_id),
        cwd: tool_use_context.cwd.clone(),
        is_non_interactive_session: true,
        options: Some(crate::utils::hooks::can_use_tool::ToolUseContextOptions {
            tools: Some(Vec::new()), // Would include filtered tools + structured output tool
        }),
    });

    // Register a session-level stop hook to enforce structured output
    // This would call register_structured_output_enforcement

    let mut structured_output_result: Option<serde_json::Value> = None;
    let mut turn_count = 0;
    let mut hit_max_turns = false;

    // Simulate multi-turn query loop
    // In the TS version, this uses query() for multi-turn execution
    // Here we'd use the crate's query function
    for message in simulate_query_loop(&agent_messages, &transcript_path, &model).await {
        // Skip streaming events
        if message.get("type") == Some(&serde_json::json!("stream_event"))
            || message.get("type") == Some(&serde_json::json!("stream_request_start"))
        {
            continue;
        }

        // Count assistant turns
        if message.get("type") == Some(&serde_json::json!("assistant")) {
            turn_count += 1;

            // Check if we've hit the turn limit
            if turn_count >= MAX_AGENT_TURNS {
                hit_max_turns = true;
                log_for_debugging(&format!(
                    "Hooks: Agent turn {} hit max turns, aborting",
                    turn_count
                ));
                let _ = abort_tx.send(true);
                break;
            }
        }

        // Check for structured output in attachments
        if let Some(attachment) = message.get("attachment") {
            if let Some(attachment_type) = attachment.get("type") {
                if attachment_type == "structured_output" {
                    if let Some(data) = attachment.get("data") {
                        // Validate against hook response schema
                        if let Ok(parsed) = serde_json::from_value::<
                            crate::utils::hooks::hook_helpers::HookResponse,
                        >(data.clone())
                        {
                            structured_output_result = Some(data.clone());
                            log_for_debugging(&format!(
                                "Hooks: Got structured output: {}",
                                serde_json::to_string(data).unwrap_or_default()
                            ));
                            // Got structured output, abort and exit
                            let _ = abort_tx.send(true);
                            break;
                        }
                    }
                }
            }
        }

        // Check abort signal
        if *abort_rx.borrow() {
            break;
        }
    }

    timeout_handle.abort();

    // Clean up the session hook we registered for this agent
    // clear_session_hooks would be called here

    // Check if we got a result
    if structured_output_result.is_none() {
        if hit_max_turns {
            log_for_debugging(&format!(
                "Hooks: Agent hook did not complete within {} turns",
                MAX_AGENT_TURNS
            ));
            log_event(
                "tengu_agent_stop_hook_max_turns",
                &serde_json::json!({
                    "duration_ms": hook_start.elapsed().as_millis(),
                    "turn_count": turn_count,
                    "agent_name": agent_name.unwrap_or("unknown"),
                }),
            );
            return HookResult::Cancelled;
        }

        log_for_debugging("Hooks: Agent hook did not return structured output");
        log_event(
            "tengu_agent_stop_hook_error",
            &serde_json::json!({
                "duration_ms": hook_start.elapsed().as_millis(),
                "turn_count": turn_count,
                "error_type": 1, // 1 = no structured output
                "agent_name": agent_name.unwrap_or("unknown"),
            }),
        );
        return HookResult::Cancelled;
    }

    // Return result based on structured output
    let result = structured_output_result.unwrap();
    if let Some(ok) = result.get("ok").and_then(|v| v.as_bool()) {
        if !ok {
            let reason = result
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            log_for_debugging(&format!(
                "Hooks: Agent hook condition was not met: {}",
                reason
            ));
            return HookResult::Blocking {
                blocking_error: format!("Agent hook condition was not met: {}", reason),
                command: hook.prompt.clone(),
            };
        }

        // Condition was met
        log_for_debugging("Hooks: Agent hook condition was met");
        log_event(
            "tengu_agent_stop_hook_success",
            &serde_json::json!({
                "duration_ms": hook_start.elapsed().as_millis(),
                "turn_count": turn_count,
                "agent_name": agent_name.unwrap_or("unknown"),
            }),
        );
        return HookResult::Success {
            hook_name: hook_name.to_string(),
            hook_event: hook_event.to_string(),
            tool_use_id: effective_tool_use_id,
        };
    }

    HookResult::Cancelled
}

/// Create a user message with the given content
fn create_user_message(content: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "user",
        "message": {
            "content": content
        }
    })
}

/// Get the small fast model (simplified)
fn get_small_fast_model() -> String {
    "claude-3-haiku-20240307".to_string()
}

/// Simulate a query loop (placeholder for actual query implementation)
async fn simulate_query_loop(
    _messages: &[serde_json::Value],
    _transcript_path: &str,
    _model: &str,
) -> Vec<serde_json::Value> {
    // This would use the actual query() function from the crate
    Vec::new()
}

/// Log event for analytics (simplified)
fn log_event(event_name: &str, _metadata: &serde_json::Value) {
    log::debug!("Analytics event: {}", event_name);
}

/// Log for debugging
fn log_for_debugging(msg: &str) {
    log::debug!("{}", msg);
}
