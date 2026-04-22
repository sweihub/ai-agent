// Source: ~/claudecode/openclaudecode/src/types/hooks.ts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::message::Message;

/// Hook events enum matching HOOK_EVENTS from the SDK.
pub type HookEvent = String;

/// Hook input type from the SDK.
pub type HookInput = HashMap<String, serde_json::Value>;

/// Permission update type.
pub type PermissionUpdate = HashMap<String, serde_json::Value>;

/// Hook JSON output type.
pub type HookJSONOutput = serde_json::Value;

/// Async hook JSON output type.
pub type AsyncHookJSONOutput = serde_json::Value;

/// Sync hook JSON output type.
pub type SyncHookJSONOutput = serde_json::Value;

/// Check if a value is a valid HookEvent.
pub fn is_hook_event(value: &str) -> bool {
    // Hook events list from SDK
    let events = [
        "PreToolUse",
        "UserPromptSubmit",
        "SessionStart",
        "Setup",
        "SubagentStart",
        "PostToolUse",
        "PostToolUseFailure",
        "PermissionDenied",
        "Notification",
        "PermissionRequest",
        "Elicitation",
        "ElicitationResult",
        "CwdChanged",
        "FileChanged",
        "WorktreeCreate",
    ];
    events.contains(&value)
}

/// Prompt elicitation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptRequest {
    /// Request id
    pub prompt: String,
    pub message: String,
    pub options: Vec<PromptOption>,
}

/// An option in a prompt request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptOption {
    pub key: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Response to a prompt request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResponse {
    /// Request id
    #[serde(rename = "prompt_response")]
    pub prompt_response: String,
    pub selected: String,
}

/// Sync hook response - whether to continue after hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHookResponse {
    /// Whether Claude should continue after hook (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_flag: Option<bool>,
    /// Hide stdout from transcript (default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "suppressOutput")]
    pub suppress_output: Option<bool>,
    /// Message shown when continue is false
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stopReason")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<HookDecision>,
    /// Explanation for the decision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Warning message shown to the user
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "systemMessage")]
    pub system_message: Option<String>,
    /// Hook-specific output
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hookSpecificOutput")]
    pub hook_specific_output: Option<HookSpecificOutput>,
}

/// Hook decision enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HookDecision {
    Approve,
    Block,
}

/// Hook-specific output discriminated by hook event name.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "hookEventName")]
pub enum HookSpecificOutput {
    #[serde(rename = "PreToolUse")]
    PreToolUse {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "permissionDecision")]
        permission_decision: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "permissionDecisionReason")]
        permission_decision_reason: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "updatedInput")]
        updated_input: Option<HashMap<String, serde_json::Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "additionalContext")]
        additional_context: Option<String>,
    },
    #[serde(rename = "UserPromptSubmit")]
    UserPromptSubmit {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "additionalContext")]
        additional_context: Option<String>,
    },
    #[serde(rename = "SessionStart")]
    SessionStart {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "additionalContext")]
        additional_context: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "initialUserMessage")]
        initial_user_message: Option<String>,
        /// Absolute paths to watch for FileChanged hooks
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "watchPaths")]
        watch_paths: Option<Vec<String>>,
    },
    #[serde(rename = "Setup")]
    Setup {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "additionalContext")]
        additional_context: Option<String>,
    },
    #[serde(rename = "SubagentStart")]
    SubagentStart {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "additionalContext")]
        additional_context: Option<String>,
    },
    #[serde(rename = "PostToolUse")]
    PostToolUse {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "additionalContext")]
        additional_context: Option<String>,
        /// Updates the output for MCP tools
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "updatedMCPToolOutput")]
        updated_mcp_tool_output: Option<serde_json::Value>,
    },
    #[serde(rename = "PostToolUseFailure")]
    PostToolUseFailure {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "additionalContext")]
        additional_context: Option<String>,
    },
    #[serde(rename = "PermissionDenied")]
    PermissionDenied {
        #[serde(skip_serializing_if = "Option::is_none")]
        retry: Option<bool>,
    },
    #[serde(rename = "Notification")]
    Notification {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "additionalContext")]
        additional_context: Option<String>,
    },
    #[serde(rename = "PermissionRequest")]
    PermissionRequest {
        #[serde(flatten)]
        decision: PermissionRequestDecision,
    },
    #[serde(rename = "Elicitation")]
    Elicitation {
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<ElicitationAction>,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<HashMap<String, serde_json::Value>>,
    },
    #[serde(rename = "ElicitationResult")]
    ElicitationResult {
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<ElicitationAction>,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<HashMap<String, serde_json::Value>>,
    },
    #[serde(rename = "CwdChanged")]
    CwdChanged {
        /// Absolute paths to watch for FileChanged hooks
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "watchPaths")]
        watch_paths: Option<Vec<String>>,
    },
    #[serde(rename = "FileChanged")]
    FileChanged {
        /// Absolute paths to watch for FileChanged hooks
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "watchPaths")]
        watch_paths: Option<Vec<String>>,
    },
    #[serde(rename = "WorktreeCreate")]
    WorktreeCreate {
        #[serde(rename = "worktreePath")]
        worktree_path: String,
    },
}

/// Permission request decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "behavior")]
pub enum PermissionRequestDecision {
    #[serde(rename = "allow")]
    Allow {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "updatedInput")]
        updated_input: Option<HashMap<String, serde_json::Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "updatedPermissions")]
        updated_permissions: Option<Vec<PermissionUpdate>>,
    },
    #[serde(rename = "deny")]
    Deny {
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        interrupt: Option<bool>,
    },
}

/// Elicitation action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ElicitationAction {
    Accept,
    Decline,
    Cancel,
}

/// Type guard to check if response is sync.
pub fn is_sync_hook_json_output(json: &HookJSONOutput) -> bool {
    !json.get("async").is_some_and(|v| v.as_bool() == Some(true))
}

/// Type guard to check if response is async.
pub fn is_async_hook_json_output(json: &HookJSONOutput) -> bool {
    json.get("async").is_some_and(|v| v.as_bool() == Some(true))
}

/// Context passed to callback hooks for state access.
pub struct HookCallbackContext {
    pub get_app_state: Box<dyn Fn() -> Box<dyn std::any::Any> + Send + Sync>,
    pub update_attribution_state:
        Box<dyn Fn(Box<dyn std::any::Any>) -> Box<dyn std::any::Any> + Send + Sync>,
}

/// Hook that is a callback.
pub struct HookCallback {
    pub callback_type: String, // "callback"
    pub callback: Box<
        dyn Fn(
                HookInput,
                Option<String>,                             // toolUseID
                Option<tokio::sync::oneshot::Receiver<()>>, // abort signal
                Option<usize>,                              // hookIndex
                Option<HookCallbackContext>,
            )
                -> std::pin::Pin<Box<dyn std::future::Future<Output = HookJSONOutput> + Send>>
            + Send
            + Sync,
    >,
    /// Timeout in seconds for this hook
    pub timeout: Option<u64>,
    /// Internal hooks excluded from tengu_run_hook metrics
    pub internal: Option<bool>,
}

/// Hook callback matcher.
pub struct HookCallbackMatcher {
    pub matcher: Option<String>,
    pub hooks: Vec<HookCallback>,
    pub plugin_name: Option<String>,
}

/// Hook progress message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookProgress {
    #[serde(rename = "type")]
    pub entry_type: String, // "hook_progress"
    #[serde(rename = "hookEvent")]
    pub hook_event: HookEvent,
    #[serde(rename = "hookName")]
    pub hook_name: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "promptText")]
    pub prompt_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "statusMessage")]
    pub status_message: Option<String>,
}

/// Hook blocking error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookBlockingError {
    #[serde(rename = "blockingError")]
    pub blocking_error: String,
    pub command: String,
}

/// Permission request result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "behavior")]
pub enum PermissionRequestResult {
    #[serde(rename = "allow")]
    Allow {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "updatedInput")]
        updated_input: Option<HashMap<String, serde_json::Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "updatedPermissions")]
        updated_permissions: Option<Vec<PermissionUpdate>>,
    },
    #[serde(rename = "deny")]
    Deny {
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        interrupt: Option<bool>,
    },
}

/// Result from running a hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "systemMessage")]
    pub system_message: Option<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "blockingError")]
    pub blocking_error: Option<HookBlockingError>,
    pub outcome: HookOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "preventContinuation")]
    pub prevent_continuation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stopReason")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "permissionBehavior")]
    pub permission_behavior: Option<PermissionBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hookPermissionDecisionReason")]
    pub hook_permission_decision_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "additionalContext")]
    pub additional_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "initialUserMessage")]
    pub initial_user_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "updatedInput")]
    pub updated_input: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "updatedMCPToolOutput")]
    pub updated_mcp_tool_output: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "permissionRequestResult")]
    pub permission_request_result: Option<PermissionRequestResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<bool>,
}

/// Hook outcome enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HookOutcome {
    Success,
    Blocking,
    NonBlockingError,
    Cancelled,
}

/// Permission behavior enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionBehavior {
    Ask,
    Deny,
    Allow,
    Passthrough,
}

/// Aggregated result from running multiple hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedHookResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "blockingErrors")]
    pub blocking_errors: Option<Vec<HookBlockingError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "preventContinuation")]
    pub prevent_continuation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stopReason")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hookPermissionDecisionReason")]
    pub hook_permission_decision_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "permissionBehavior")]
    pub permission_behavior: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "additionalContexts")]
    pub additional_contexts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "initialUserMessage")]
    pub initial_user_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "updatedInput")]
    pub updated_input: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "updatedMCPToolOutput")]
    pub updated_mcp_tool_output: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "permissionRequestResult")]
    pub permission_request_result: Option<PermissionRequestResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<bool>,
}
