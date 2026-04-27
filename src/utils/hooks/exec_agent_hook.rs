// Source: ~/claudecode/openclaudecode/src/utils/hooks/execAgentHook.ts
#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

use crate::types::Message;
use crate::utils::hooks::helpers::{add_arguments_to_prompt, hook_response_json_schema};
use crate::utils::hooks::hook_helpers::SYNTHETIC_OUTPUT_TOOL_NAME;
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
    register_structured_output_enforcement_impl(&hook_agent_id);

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
    clear_session_hooks_impl(&hook_agent_id);

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

/// Execute a multi-turn agent query loop using direct API calls.
/// Avoids importing from crate::query_engine and crate::agent to prevent
/// a compile-time type cycle: hooks -> exec_agent_hook -> query_engine -> hooks.
///
/// For each turn:
/// 1. Read the transcript file if it exists and prepend to context
/// 2. Call the Anthropic API with JSON schema output
/// 3. Parse the response for structured output (ok/reason)
/// 4. Return events compatible with the exec_agent_hook consumer
async fn simulate_query_loop(
    messages: &[serde_json::Value],
    transcript_path: &str,
    model: &str,
) -> Vec<serde_json::Value> {
    use crate::utils::hooks::hook_helpers::hook_response_schema;

    // Extract prompt text from input messages.
    let prompt = messages
        .iter()
        .filter_map(|m| {
            Some(
                m.get("message")
                    .and_then(|msg| msg.get("content"))
                    .or_else(|| m.get("content"))?
                    .as_str()?
                    .to_string(),
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    // Read transcript file content if available
    let transcript_content = tokio::fs::read_to_string(transcript_path)
        .await
        .unwrap_or_default();

    // Build system prompt with transcript context
    let system_prompt = format!(
        "You are verifying a stop condition in Claude Code. Your task is to verify that \
         the agent completed the given plan.\n\nConversation transcript:{}\n\n\
         Use the transcript above to analyze the conversation history.\
         Return your verification result as JSON.",
        if transcript_content.is_empty() {
            " (not available)".to_string()
        } else {
            format!("\n---\n{}\n---", transcript_content.chars().take(50000).collect::<String>())
        }
    );

    // Build the query messages
    let user_msg = serde_json::json!({
        "role": "user",
        "content": prompt
    });
    let query_messages = vec![user_msg];

    // Resolve API credentials
    let base_url = std::env::var("AI_API_BASE_URL")
        .unwrap_or_else(|_| "https://api.anthropic.com".to_string());
    let api_key = std::env::var("AI_AUTH_TOKEN")
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .or_else(|_| std::env::var("ANTHROPIC_AUTH_TOKEN"))
        .ok();

    if api_key.is_none() {
        log_for_debugging("Hooks: No API key available, skipping agent query");
        return Vec::new();
    }
    let api_key = api_key.unwrap();

    let url = format!("{}/v1/messages", base_url);
    let request_body = serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "system": [{"type": "text", "text": system_prompt}],
        "messages": query_messages,
        "temperature": 0.0,
        "output": {
            "type": "json_schema",
            "name": "hook_response",
            "schema": hook_response_schema(),
            "strict": true
        }
    });

    let client = reqwest::Client::new();
    let mut req_builder = client.post(&url)
        .json(&request_body)
        .header("Content-Type", "application/json");

    if base_url.contains("anthropic.com") {
        req_builder = req_builder
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01");
    } else {
        req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
    }

    let mut result = Vec::new();
    // Emit assistant turn event for turn counting
    result.push(serde_json::json!({ "type": "assistant" }));

    match req_builder.send().await {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            if !status.is_success() {
                log_for_debugging(&format!("Hooks: API error {}: {}", status, body));
                result.push(serde_json::json!({ "type": "done" }));
                return result;
            }

            let parsed: serde_json::Value = match serde_json::from_str(&body) {
                Ok(v) => v,
                Err(e) => {
                    log_for_debugging(&format!("Hooks: Failed to parse API response: {}", e));
                    result.push(serde_json::json!({ "type": "done" }));
                    return result;
                }
            };

            // Extract text from response (supports both Anthropic and OpenAI formats)
            let text = extract_text(&parsed);
            if text.is_empty() {
                log_for_debugging("Hooks: Empty response from model");
                result.push(serde_json::json!({ "type": "done" }));
                return result;
            }

            log_for_debugging(&format!("Hooks: Model response: {}", text));

            // Emit structured output attachment so the caller detects it
            result.push(serde_json::json!({
                "type": "attachment",
                "attachment": {
                    "type": "structured_output",
                    "data": serde_json::from_str::<serde_json::Value>(&text).unwrap_or_else(|_| {
                        serde_json::json!({"ok": false, "reason": "Failed to parse model response"})
                    })
                }
            }));
        }
        Err(e) => {
            log_for_debugging(&format!("Hooks: Request failed: {}", e));
        }
    }

    result.push(serde_json::json!({ "type": "done" }));
    result
}

/// Extract text content from an API response (supports both Anthropic and OpenAI formats)
fn extract_text(response: &serde_json::Value) -> String {
    // OpenAI format: choices[].message.content
    if let Some(content) = response.get("choices").and_then(|c| c.as_array())
        .and_then(|c| c.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str()) {
        return content.to_string();
    }
    // Anthropic format: content[].text
    if let Some(blocks) = response.get("content").and_then(|c| c.as_array()) {
        let mut texts = Vec::new();
        for block in blocks {
            if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                texts.push(text.to_string());
            }
        }
        if !texts.is_empty() {
            return texts.join("\n");
        }
    }
    String::new()
}

/// No-op set_app_state for use with session hook functions that require a
/// state setter.  The real session-hook state lives in an internal static,
/// so this placeholder is sufficient.
fn noop_set_app_state(_updater: &dyn Fn(&mut serde_json::Value)) {
    // No-op — internal SESSION_HOOKS_STATE handles the actual storage.
}

/// Register structured output enforcement for the given session/agent ID.
/// Wraps the hook_helpers function with a no-op set_app_state.
fn register_structured_output_enforcement_impl(session_id: &str) {
    crate::utils::hooks::hook_helpers::register_structured_output_enforcement(
        &noop_set_app_state,
        session_id,
    );
}

/// Clear session hooks for the given session/agent ID.
/// Wraps the session_hooks function with a no-op set_app_state.
fn clear_session_hooks_impl(session_id: &str) {
    clear_session_hooks(&noop_set_app_state, session_id);
}

/// Log event for analytics (simplified)
fn log_event(event_name: &str, _metadata: &serde_json::Value) {
    log::debug!("Analytics event: {}", event_name);
}

/// Log for debugging
fn log_for_debugging(msg: &str) {
    log::debug!("{}", msg);
}
