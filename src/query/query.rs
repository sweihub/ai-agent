// Source: ~/claudecode/openclaudecode/src/query.ts

//! Query loop - the core streaming agent loop translated from TypeScript.
//!
//! This module implements the main query loop that:
//! - Streams model responses
//! - Executes tool calls
//! - Handles compaction (auto, micro, reactive)
//! - Manages token budgets and recovery paths
//! - Processes attachments and commands

use std::collections::HashSet;
use std::sync::Arc;

use crate::error::AgentError;
use crate::query::config::QueryConfig;
use crate::query::deps::QueryDeps;
use crate::query::transitions::{Continue, Terminal};
use crate::tool::ToolUseContext;
use crate::types::message::{
    AssistantMessage, AssistantMessageContent, AttachmentMessage, Message, StreamEvent,
    TombstoneMessage, ToolUseSummaryMessage, UserMessage, UserMessageContent,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of retries for max_output_tokens recovery.
const MAX_OUTPUT_TOKENS_RECOVERY_LIMIT: u32 = 3;

// ---------------------------------------------------------------------------
// Query parameters
// ---------------------------------------------------------------------------

/// Parameters for a query invocation.
pub struct QueryParams {
    pub messages: Vec<Message>,
    pub system_prompt: String,
    pub user_context: std::collections::HashMap<String, String>,
    pub system_context: std::collections::HashMap<String, String>,
    pub can_use_tool: Arc<CanUseToolFn>,
    pub tool_use_context: Arc<ToolUseContext>,
    pub fallback_model: Option<String>,
    pub query_source: String,
    pub max_output_tokens_override: Option<u64>,
    pub max_turns: Option<u32>,
    pub skip_cache_write: bool,
    /// API task_budget (output_config.task_budget). `total` is the budget for
    /// the whole agentic turn; `remaining` is computed per iteration.
    pub task_budget: Option<TaskBudgetParam>,
    pub deps: Option<QueryDeps>,
}

/// Task budget parameter from the API.
#[derive(Debug, Clone)]
pub struct TaskBudgetParam {
    pub total: u64,
}

/// CanUseTool function type - determines whether a tool may be executed.
pub type CanUseToolFn = dyn Fn(
        &serde_json::Value, // tool definition
        &serde_json::Value, // input
        Arc<ToolUseContext>,
        Arc<AssistantMessage>,
        &str,  // query source
        bool,  // is explicit
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PermissionDecision, AgentError>> + Send>,
    > + Send
    + Sync;

/// Permission decision from can_use_tool.
#[derive(Debug, Clone)]
pub enum PermissionDecision {
    Allow,
    Deny { reason: Option<String> },
    Ask { expires_at: Option<u64> },
}

// ---------------------------------------------------------------------------
// Stream event types
// ---------------------------------------------------------------------------

/// Events yielded by the query generator.
#[derive(Debug, Clone)]
pub enum QueryEvent {
    StreamEvent(StreamEvent),
    RequestStartEvent(RequestStartEvent),
    Message(Message),
    Tombstone(TombstoneMessage),
    ToolUseSummary(ToolUseSummaryMessage),
}

impl From<StreamEvent> for QueryEvent {
    fn from(event: StreamEvent) -> Self {
        QueryEvent::StreamEvent(event)
    }
}

impl From<Message> for QueryEvent {
    fn from(msg: Message) -> Self {
        QueryEvent::Message(msg)
    }
}

impl From<TombstoneMessage> for QueryEvent {
    fn from(msg: TombstoneMessage) -> Self {
        QueryEvent::Tombstone(msg)
    }
}

impl From<ToolUseSummaryMessage> for QueryEvent {
    fn from(msg: ToolUseSummaryMessage) -> Self {
        QueryEvent::ToolUseSummary(msg)
    }
}

/// Request start event.
#[derive(Debug, Clone)]
pub struct RequestStartEvent {
    pub event_type: String,
}

impl Default for RequestStartEvent {
    fn default() -> Self {
        Self {
            event_type: "stream_request_start".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Query loop state
// ---------------------------------------------------------------------------

/// Mutable state carried between loop iterations.
pub struct QueryState {
    pub messages: Vec<Message>,
    pub tool_use_context: Arc<ToolUseContext>,
    pub auto_compact_tracking: Option<AutoCompactTrackingState>,
    pub max_output_tokens_recovery_count: u32,
    pub has_attempted_reactive_compact: bool,
    pub max_output_tokens_override: Option<u64>,
    pub pending_tool_use_summary: Option<ToolUseSummaryFuture>,
    pub stop_hook_active: Option<bool>,
    pub turn_count: u32,
    /// Why the previous iteration continued. Undefined on first iteration.
    pub transition: Option<Continue>,
}

/// Future that resolves to a tool use summary message or null.
pub type ToolUseSummaryFuture =
    std::pin::Pin<Box<dyn std::future::Future<Output = Option<ToolUseSummaryMessage>> + Send>>;

/// Auto-compact tracking state across iterations.
#[derive(Debug, Clone)]
pub struct AutoCompactTrackingState {
    pub compacted: bool,
    pub turn_id: String,
    pub turn_counter: u32,
    pub consecutive_failures: u32,
}

// ---------------------------------------------------------------------------
// Helper: yield missing tool result blocks
// ---------------------------------------------------------------------------

/// For each tool_use block in assistant messages that lacks a tool_result,
/// yield a user message with an error tool_result.
fn yield_missing_tool_result_blocks(
    assistant_messages: &[AssistantMessage],
    error_message: &str,
) -> Vec<UserMessage> {
    let mut result = Vec::new();
    for assistant_msg in assistant_messages {
        let tool_use_blocks: Vec<&serde_json::Value> = match &assistant_msg.message {
            Some(content) => match &content.content {
                Some(content_val) => {
                    if let Some(arr) = content_val.as_array() {
                        arr.iter()
                            .filter(|block| {
                                block.get("type").and_then(|v| v.as_str()) == Some("tool_use")
                            })
                            .collect()
                    } else {
                        Vec::new()
                    }
                }
                None => Vec::new(),
            },
            None => Vec::new(),
        };

        for tool_use in tool_use_blocks {
            let tool_use_id = tool_use
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let content = serde_json::json!([
                {
                    "type": "tool_result",
                    "content": error_message,
                    "is_error": true,
                    "tool_use_id": tool_use_id,
                }
            ]);

            result.push(UserMessage {
                base: crate::types::message::MessageBase {
                    uuid: None,
                    parent_uuid: None,
                    timestamp: None,
                    created_at: None,
                    is_meta: Some(true),
                    is_virtual: None,
                    is_compact_summary: None,
                    tool_use_result: Some(serde_json::Value::String(error_message.to_string())),
                    origin: None,
                    extra: std::collections::HashMap::new(),
                },
                message_type: "user".to_string(),
                message: UserMessageContent {
                    content: crate::types::message::UserContent::Blocks(vec![]),
                    extra: std::collections::HashMap::new(),
                },
            });
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Withheld error checks
// ---------------------------------------------------------------------------

/// Check if a message is a withheld max_output_tokens error.
fn is_withheld_max_output_tokens(msg: Option<&Message>) -> bool {
    match msg {
        Some(Message::Assistant(assistant)) => {
            matches!(
                assistant.base.extra.get("apiError"),
                Some(serde_json::Value::String(s)) if s == "max_output_tokens"
            )
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Main query generator
// ---------------------------------------------------------------------------

/// The main query loop. Returns an async generator that yields query events
/// and ultimately returns a terminal result.
///
/// This is the Rust translation of the TypeScript `query()` function.
/// The loop handles:
/// - Streaming model responses
/// - Tool execution (streaming and batch)
/// - Auto-compaction, micro-compaction, snip
/// - Token budget management
/// - Recovery from prompt-too-long and max_output_tokens errors
/// - Command and attachment processing
/// - Stop hooks
pub async fn query(
    params: QueryParams,
    mut yield_fn: impl FnMut(QueryEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        + Send,
) -> Result<Terminal, AgentError> {
    let mut consumed_command_uuids: Vec<String> = Vec::new();
    let terminal =
        query_loop(params, &mut consumed_command_uuids, &mut yield_fn).await?;

    // Notify command lifecycle for consumed commands (only on normal completion)
    for uuid in &consumed_command_uuids {
        // notify_command_lifecycle(uuid, "completed");
        let _ = uuid;
    }

    Ok(terminal)
}

// ---------------------------------------------------------------------------
// Query loop internals
// ---------------------------------------------------------------------------

async fn query_loop(
    params: QueryParams,
    _consumed_command_uuids: &mut Vec<String>,
    yield_fn: &mut impl FnMut(QueryEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        + Send,
) -> Result<Terminal, AgentError> {
    // Immutable params
    let QueryParams {
        system_prompt,
        user_context,
        system_context,
        can_use_tool,
        fallback_model,
        query_source,
        max_turns,
        skip_cache_write,
        task_budget,
        ..
    } = params;

    let deps = params.deps.unwrap_or_else(QueryDeps::production);
    let tool_use_context = params.tool_use_context;

    // Mutable cross-iteration state
    let mut state = QueryState {
        messages: params.messages,
        tool_use_context,
        max_output_tokens_override: params.max_output_tokens_override,
        auto_compact_tracking: None,
        stop_hook_active: None,
        max_output_tokens_recovery_count: 0,
        has_attempted_reactive_compact: false,
        turn_count: 1,
        pending_tool_use_summary: None,
        transition: None,
    };

    // Token budget tracker (feature-gated)
    let _budget_tracker = None; // feature("TOKEN_BUDGET") ? createBudgetTracker() : null

    // task_budget.remaining tracking across compaction boundaries
    let mut task_budget_remaining: Option<u64> = None;

    // Snapshot config once at entry
    let config = QueryConfig::new(uuid::Uuid::new_v4().to_string());

    // eslint-disable-next-line no-constant-condition
    loop {
        // Destructure state at the top of each iteration
        let tool_use_context = state.tool_use_context.clone();
        let QueryState {
            messages,
            auto_compact_tracking,
            max_output_tokens_recovery_count,
            has_attempted_reactive_compact,
            max_output_tokens_override,
            pending_tool_use_summary,
            stop_hook_active,
            turn_count,
            ..
        } = &state;

        let turn_count = *turn_count;
        let max_output_tokens_recovery_count = *max_output_tokens_recovery_count;
        let has_attempted_reactive_compact = *has_attempted_reactive_compact;
        let max_output_tokens_override = *max_output_tokens_override;

        // Skill discovery prefetch (feature-gated)
        let _pending_skill_prefetch: Option<()> = None;

        // Yield stream request start
        yield_fn(QueryEvent::StreamEvent(StreamEvent {
            event_type: Some("stream_request_start".to_string()),
            extra: std::collections::HashMap::new(),
        }))
        .await;

        // Query checkpoint
        query_checkpoint("query_fn_entry");

        // Initialize or increment query chain tracking
        let query_tracking = match &tool_use_context.query_tracking {
            Some(tracking) => crate::tool::QueryChainTracking {
                chain_id: tracking.chain_id.clone(),
                depth: tracking.depth + 1,
            },
            None => crate::tool::QueryChainTracking {
                chain_id: (deps.uuid)(),
                depth: 0,
            },
        };

        let query_chain_id_for_analytics = query_tracking.chain_id.clone();

        let tool_use_context = Arc::new(ToolUseContext {
            query_tracking: Some(query_tracking.clone()),
            ..(*tool_use_context).clone()
        });

        // Get messages after compact boundary
        let mut messages_for_query =
            get_messages_after_compact_boundary(&state.messages);

        let mut tracking = state.auto_compact_tracking.clone();

        // Apply tool result budget
        messages_for_query = apply_tool_result_budget(
            messages_for_query,
            &tool_use_context,
            &query_source,
        )
        .await;

        // Apply snip before microcompact (feature-gated)
        let mut _snip_tokens_freed = 0u64;

        // Apply microcompact before autocompact
        let microcompact_result = deps
            .microcompact(messages_for_query.clone(), tool_use_context.clone(), &query_source)
            .await;
        messages_for_query = microcompact_result.messages;

        // Apply context collapse if enabled (feature-gated)

        // Build full system prompt
        let full_system_prompt =
            append_system_context(&system_prompt, &system_context);

        // Auto-compact
        let compaction_result = deps
            .autocompact(
                messages_for_query.clone(),
                tool_use_context.clone(),
                AutoCompactInput {
                    system_prompt: system_prompt.clone(),
                    user_context: user_context.clone(),
                    system_context: system_context.clone(),
                    tool_use_context: tool_use_context.clone(),
                    fork_context_messages: messages_for_query.clone(),
                },
                &query_source,
                tracking.clone(),
                _snip_tokens_freed,
            )
            .await;

        if let Some(compaction) = &compaction_result {
            let (pre_compact_token_count, post_compact_token_count, true_post_compact_token_count) =
                (
                    compaction.pre_compact_token_count,
                    compaction.post_compact_token_count,
                    compaction.true_post_compact_token_count,
                );

            // Log analytics event
            log_event(
                "tengu_auto_compact_succeeded",
                serde_json::json!({
                    "originalMessageCount": state.messages.len(),
                    "compactedMessageCount": compaction.summary_messages.len()
                        + compaction.attachments.len()
                        + compaction.hook_results.len(),
                    "preCompactTokenCount": pre_compact_token_count,
                    "postCompactTokenCount": post_compact_token_count,
                    "truePostCompactTokenCount": true_post_compact_token_count,
                    "queryChainId": query_chain_id_for_analytics,
                    "queryDepth": query_tracking.depth,
                }),
            );

            // task_budget: capture pre-compact final context window
            if let Some(tb) = &task_budget {
                let pre_compact_context =
                    final_context_tokens_from_last_response(&messages_for_query);
                task_budget_remaining = Some(
                    (task_budget_remaining.unwrap_or(tb.total))
                        .saturating_sub(pre_compact_context),
                );
            }

            // Reset tracking on every compact
            tracking = Some(AutoCompactTrackingState {
                compacted: true,
                turn_id: (deps.uuid)(),
                turn_counter: 0,
                consecutive_failures: 0,
            });

            let post_compact_messages = build_post_compact_messages(compaction);

            for message in &post_compact_messages {
                yield_fn(QueryEvent::Message(message.clone())).await;
            }

            messages_for_query = post_compact_messages;
        } else if let Some(consecutive_failures) = compaction_result
            .as_ref()
            .and_then(|_| None::<Option<u32>>)
        {
            // Autocompact failed - propagate failure count
            tracking = Some(match tracking {
                Some(t) => AutoCompactTrackingState {
                    consecutive_failures,
                    ..t
                },
                None => AutoCompactTrackingState {
                    compacted: false,
                    turn_id: String::new(),
                    turn_counter: 0,
                    consecutive_failures,
                },
            });
        }

        // Update tool use context with current messages
        let tool_use_context = Arc::new(ToolUseContext {
            messages: messages_for_query.clone(),
            ..(*tool_use_context).clone()
        });

        let mut assistant_messages: Vec<AssistantMessage> = Vec::new();
        let mut tool_results: Vec<Message> = Vec::new();
        let mut tool_use_blocks: Vec<serde_json::Value> = Vec::new();
        let mut needs_follow_up = false;

        // Streaming tool execution setup
        let use_streaming_tool_execution = config.gates.streaming_tool_execution;
        let mut _streaming_tool_executor: Option<()> = if use_streaming_tool_execution {
            // StreamingToolExecutor::new(...)
            None
        } else {
            None
        };

        // Determine current model
        let current_model = get_runtime_main_loop_model(&tool_use_context);

        // Blocking limit check (skip if compaction just happened)
        let compaction_result = None; // already consumed above
        let media_recovery_enabled = false; // reactiveCompact?.isReactiveCompactEnabled() ?? false
        let collapse_owns_it = false;

        if compaction_result.is_none()
            && query_source != "compact"
            && query_source != "session_memory"
            && !media_recovery_enabled
            && !collapse_owns_it
        {
            let token_count = token_count_with_estimation(&messages_for_query);
            let is_at_blocking_limit = calculate_token_warning_state(
                token_count,
                &tool_use_context.options.main_loop_model,
            )
            .is_at_blocking_limit;

            if is_at_blocking_limit {
                yield_fn(QueryEvent::Message(Message::System(
                    crate::types::message::SystemMessage {
                        base: crate::types::message::MessageBase {
                            uuid: None,
                            parent_uuid: None,
                            timestamp: None,
                            created_at: None,
                            is_meta: None,
                            is_virtual: None,
                            is_compact_summary: None,
                            tool_use_result: None,
                            origin: None,
                            extra: std::collections::HashMap::new(),
                        },
                        message_type: "system".to_string(),
                        subtype: Some("api_error".to_string()),
                        level: Some("error".to_string()),
                        message: Some(PROMPT_TOO_LONG_ERROR_MESSAGE.to_string()),
                    },
                )))
                .await;
                return Ok(Terminal {
                    reason: "blocking_limit".to_string(),
                });
            }
        }

        // -- API streaming loop --
        let mut attempt_with_fallback = true;
        let mut streaming_fallback_occurred = false;

        while attempt_with_fallback {
            attempt_with_fallback = false;

            // Call model streaming
            match call_model_streaming(
                &messages_for_query,
                &full_system_prompt,
                &tool_use_context,
                &current_model,
                fallback_model.as_deref(),
                &query_source,
                max_output_tokens_override,
                skip_cache_write,
                task_budget.as_ref(),
                task_budget_remaining,
                &can_use_tool,
                yield_fn,
            )
            .await
            {
                Ok(stream_result) => {
                    assistant_messages = stream_result.assistant_messages;
                    tool_results = stream_result.tool_results;
                    tool_use_blocks = stream_result.tool_use_blocks;
                    needs_follow_up = stream_result.needs_follow_up;
                    streaming_fallback_occurred = stream_result.fallback_occurred;

                    if streaming_fallback_occurred {
                        // Yield tombstones for orphaned messages
                        for msg in &assistant_messages {
                            yield_fn(QueryEvent::Tombstone(TombstoneMessage {
                                base: crate::types::message::MessageBase {
                                    uuid: None,
                                    parent_uuid: None,
                                    timestamp: None,
                                    created_at: None,
                                    is_meta: None,
                                    is_virtual: Some(true),
                                    is_compact_summary: None,
                                    tool_use_result: None,
                                    origin: None,
                                    extra: std::collections::HashMap::new(),
                                },
                                message_type: "tombstone".to_string(),
                            }))
                            .await;
                        }

                        log_event(
                            "tengu_orphaned_messages_tombstoned",
                            serde_json::json!({
                                "orphanedMessageCount": assistant_messages.len(),
                                "queryChainId": query_chain_id_for_analytics,
                                "queryDepth": query_tracking.depth,
                            }),
                        );

                        assistant_messages.clear();
                        tool_results.clear();
                        tool_use_blocks.clear();
                        needs_follow_up = false;
                    }
                }
                Err(e) => {
                    log_error(&e);
                    let error_message = e.to_string();

                    log_event(
                        "tengu_query_error",
                        serde_json::json!({
                            "assistantMessages": assistant_messages.len(),
                            "queryChainId": query_chain_id_for_analytics,
                            "queryDepth": query_tracking.depth,
                        }),
                    );

                    // Yield missing tool result blocks
                    for user_msg in
                        yield_missing_tool_result_blocks(&assistant_messages, &error_message)
                    {
                        yield_fn(QueryEvent::Message(Message::User(user_msg)))
                            .await;
                    }

                    // Surface the real error
                    return Err(e);
                }
            }
        }

        // Execute post-sampling hooks after model response is complete
        if !assistant_messages.is_empty() {
            let _ = execute_post_sampling_hooks(
                &messages_for_query,
                &assistant_messages,
                &system_prompt,
                &user_context,
                &system_context,
                &tool_use_context,
                &query_source,
            );
        }

        // Handle abort/streaming cancellation
        if tool_use_context.abort_signal.is_some() {
            // Consume remaining tool results or yield missing tool result blocks
            yield createUserInterruption_message(false).await;
            return Ok(Terminal {
                reason: "aborted_streaming".to_string(),
            });
        }

        // Yield tool use summary from previous turn
        if let Some(summary_future) = state.pending_tool_use_summary.take() {
            if let Some(summary) = summary_future.await {
                yield_fn(QueryEvent::ToolUseSummary(summary)).await;
            }
        }

        // -- No follow-up needed (model is done) --
        if !needs_follow_up {
            let last_message = assistant_messages.last();

            // Check for withheld prompt-too-long and attempt recovery
            let is_withheld_413 = is_prompt_too_long_message(last_message);
            let is_withheld_media = false; // media recovery check

            if is_withheld_413 {
                // Try collapse drain first
                if state.transition.as_ref().map(|t| t.reason)
                    != Some("collapse_drain_retry")
                {
                    // context_collapse.recoverFromOverflow(...)
                }
            }

            if (is_withheld_413 || is_withheld_media) {
                // Try reactive compact
                // reactiveCompact.tryReactiveCompact(...)
            }

            // Check for max_output_tokens recovery
            if is_withheld_max_output_tokens(last_message) {
                if max_output_tokens_recovery_count < MAX_OUTPUT_TOKENS_RECOVERY_LIMIT {
                    let recovery_message = create_user_message(
                        "Output token limit hit. Resume directly \
                         — no apology, no recap of what you were doing. \
                         Pick up mid-thought if that is where the cut happened. \
                         Break remaining work into smaller pieces.",
                        true,
                    );

                    state = QueryState {
                        messages: [
                            messages_for_query.clone(),
                            assistant_messages
                                .iter()
                                .map(|m| Message::Assistant(m.clone()))
                                .collect(),
                            vec![recovery_message],
                        ]
                        .concat(),
                        tool_use_context: tool_use_context.clone(),
                        auto_compact_tracking: tracking.clone(),
                        max_output_tokens_recovery_count: max_output_tokens_recovery_count + 1,
                        has_attempted_reactive_compact,
                        max_output_tokens_override: None,
                        pending_tool_use_summary: None,
                        stop_hook_active: None,
                        turn_count,
                        transition: Some(Continue {
                            reason: "max_output_tokens_recovery".to_string(),
                            extra: std::collections::HashMap::new(),
                        }),
                    };
                    continue;
                }

                // Recovery exhausted - surface the withheld error
                if let Some(msg) = last_message {
                    yield_fn(QueryEvent::Message(msg.clone())).await;
                }
            }

            // Skip stop hooks when last message is an API error
            if last_message.map_or(false, is_api_error_message) {
                execute_stop_failure_hooks(last_message.unwrap(), &tool_use_context);
                return Ok(Terminal {
                    reason: "completed".to_string(),
                });
            }

            // Handle stop hooks
            let stop_hook_result = handle_stop_hooks(
                &messages_for_query,
                &assistant_messages,
                &system_prompt,
                &user_context,
                &system_context,
                &tool_use_context,
                &query_source,
                stop_hook_active,
            )
            .await;

            if stop_hook_result.prevent_continuation {
                return Ok(Terminal {
                    reason: "stop_hook_prevented".to_string(),
                });
            }

            if !stop_hook_result.blocking_errors.is_empty() {
                state = QueryState {
                    messages: [
                        messages_for_query.clone(),
                        assistant_messages
                            .iter()
                            .map(|m| Message::Assistant(m.clone()))
                            .collect(),
                        stop_hook_result.blocking_errors,
                    ]
                    .concat(),
                    tool_use_context: tool_use_context.clone(),
                    auto_compact_tracking: tracking.clone(),
                    max_output_tokens_recovery_count: 0,
                    has_attempted_reactive_compact,
                    max_output_tokens_override: None,
                    pending_tool_use_summary: None,
                    stop_hook_active: Some(true),
                    turn_count,
                    transition: Some(Continue {
                        reason: "stop_hook_blocking".to_string(),
                        extra: std::collections::HashMap::new(),
                    }),
                };
                continue;
            }

            // Token budget check (feature-gated)
            // checkTokenBudget(...)

            return Ok(Terminal {
                reason: "completed".to_string(),
            });
        }

        // -- Tool execution --
        let mut should_prevent_continuation = false;
        let mut updated_tool_use_context = (*tool_use_context).clone();

        // Run tools
        let tool_updates = run_tools(
            &tool_use_blocks,
            &assistant_messages,
            &can_use_tool,
            &tool_use_context,
        )
        .await;

        for update in tool_updates {
            if let Some(message) = update.message {
                yield_fn(QueryEvent::Message(message.clone())).await;

                if is_hook_stopped_continuation(&message) {
                    should_prevent_continuation = true;
                }

                tool_results.push(message);
            }
            if let Some(new_context) = update.new_context {
                updated_tool_use_context = new_context;
            }
        }

        // Generate tool use summary for next turn
        let next_pending_tool_use_summary: Option<ToolUseSummaryFuture> =
            if config.gates.emit_tool_use_summaries
                && !tool_use_blocks.is_empty()
                && tool_use_context.abort_signal.is_none()
                && tool_use_context.agent_id.is_none()
            {
                let last_assistant_text = assistant_messages.last().and_then(|msg| {
                    extract_last_assistant_text(&msg.message)
                });

                let tool_info: Vec<serde_json::Value> = tool_use_blocks
                    .iter()
                    .map(|block| {
                        serde_json::json!({
                            "name": block.get("name").and_then(|v| v.as_str()),
                            "input": block.get("input"),
                            "output": null,
                        })
                    })
                    .collect();

                Some(Box::pin(async move {
                    generate_tool_use_summary(
                        tool_info,
                        &tool_use_context,
                        last_assistant_text.as_deref(),
                    )
                    .await
                }) as ToolUseSummaryFuture)
            } else {
                None
            };

        // Handle abort during tool calls
        if tool_use_context.abort_signal.is_some() {
            if tool_use_context.abort_signal.as_ref().map(|s| s != "interrupt").unwrap_or(true)
            {
                yield_fn(QueryEvent::Message(createUserInterruption_message(true)))
                    .await;
            }

            // Check maxTurns before returning when aborted
            let next_turn_count_on_abort = turn_count + 1;
            if let Some(mt) = max_turns {
                if next_turn_count_on_abort > mt {
                    yield_fn(QueryEvent::Message(create_max_turns_attachment(
                        mt,
                        next_turn_count_on_abort,
                    )))
                    .await;
                }
            }
            return Ok(Terminal {
                reason: "aborted_tools".to_string(),
            });
        }

        // Hook indicated to prevent continuation
        if should_prevent_continuation {
            return Ok(Terminal {
                reason: "hook_stopped".to_string(),
            });
        }

        // Update compact tracking counter
        if let Some(ref mut t) = state.auto_compact_tracking {
            if t.compacted {
                t.turn_counter += 1;
                log_event(
                    "tengu_post_autocompact_turn",
                    serde_json::json!({
                        "turnId": t.turn_id,
                        "turnCounter": t.turn_counter,
                        "queryChainId": query_chain_id_for_analytics,
                        "queryDepth": query_tracking.depth,
                    }),
                );
            }
        }

        // Get queued commands and process attachments
        let queued_commands = get_commands_by_max_priority("next");
        let consumed_commands: Vec<_> = queued_commands
            .into_iter()
            .filter(|cmd| {
                cmd.mode.as_deref() == Some("prompt")
                    || cmd.mode.as_deref() == Some("task-notification")
            })
            .collect();

        // Process attachments
        let attachment_messages = get_attachment_messages(
            &updated_tool_use_context,
            &consumed_commands,
            &[
                messages_for_query.clone(),
                assistant_messages
                    .iter()
                    .map(|m| Message::Assistant(m.clone()))
                    .collect(),
                tool_results.clone(),
            ]
            .concat(),
            &query_source,
        )
        .await;

        for attachment in attachment_messages {
            yield_fn(QueryEvent::Message(attachment.clone())).await;
            tool_results.push(attachment);
        }

        // Remove consumed commands from queue
        if !consumed_commands.is_empty() {
            for cmd in &consumed_commands {
                if let Some(uuid) = &cmd.uuid {
                    _consumed_command_uuids.push(uuid.clone());
                }
            }
            remove_from_queue(&consumed_commands);
        }

        // Refresh tools between turns
        if let Some(refresh) = &updated_tool_use_context.options.refresh_tools {
            let refreshed_tools = refresh();
            if refreshed_tools != updated_tool_use_context.options.tools {
                updated_tool_use_context.options.tools = refreshed_tools;
            }
        }

        // Check max turns limit
        let next_turn_count = turn_count + 1;
        if let Some(mt) = max_turns {
            if next_turn_count > mt {
                yield_fn(QueryEvent::Message(create_max_turns_attachment(
                    mt, next_turn_count,
                )))
                .await;
                return Ok(Terminal {
                    reason: "max_turns".to_string(),
                });
            }
        }

        // Recurse to next turn
        state = QueryState {
            messages: [
                messages_for_query,
                assistant_messages
                    .iter()
                    .map(|m| Message::Assistant(m.clone()))
                    .collect(),
                tool_results,
            ]
            .concat(),
            tool_use_context: Arc::new(ToolUseContext {
                query_tracking: Some(query_tracking.clone()),
                ..updated_tool_use_context
            }),
            auto_compact_tracking: tracking,
            turn_count: next_turn_count,
            max_output_tokens_recovery_count: 0,
            has_attempted_reactive_compact: false,
            pending_tool_use_summary: next_pending_tool_use_summary,
            max_output_tokens_override: None,
            stop_hook_active: None,
            transition: Some(Continue {
                reason: "next_turn".to_string(),
                extra: std::collections::HashMap::new(),
            }),
        };
    } // while (true)
}

// ---------------------------------------------------------------------------
// Stub implementations for external dependencies
// ---------------------------------------------------------------------------

fn query_checkpoint(_label: &str) {
    // Profiling checkpoint - no-op in release builds
}

fn log_event(_event_name: &str, _metadata: serde_json::Value) {
    // Analytics logging - stub
}

fn log_error(_error: &AgentError) {
    // Error logging - stub
}

fn get_messages_after_compact_boundary(messages: &[Message]) -> Vec<Message> {
    // Return messages after the last compact boundary marker
    messages.to_vec()
}

fn append_system_context(
    system_prompt: &str,
    system_context: &std::collections::HashMap<String, String>,
) -> String {
    let mut result = system_prompt.to_string();
    for (key, value) in system_context {
        result.push_str(&format!("\n\n{}: {}", key, value));
    }
    result
}

async fn apply_tool_result_budget(
    messages: Vec<Message>,
    _tool_use_context: &Arc<ToolUseContext>,
    _query_source: &str,
) -> Vec<Message> {
    // Enforce per-message budget on aggregate tool result size
    messages
}

fn token_count_with_estimation(messages: &[Message]) -> u64 {
    // Estimate token count for