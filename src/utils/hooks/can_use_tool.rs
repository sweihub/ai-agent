//! CanUseToolFn type and related types for tool permission checking.
//!
//! This module provides the core function type for checking whether a tool
//! can be used, along with supporting types.

use crate::permission::PermissionDecision;
use crate::types::ToolDefinition;
use crate::utils::messages::{AssistantMessage, AssistantMessageContent};
use serde::{Deserialize, Serialize};

/// Context for tool use, containing information about the current execution context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolUseContext {
    /// Session ID
    pub session_id: String,
    /// Current working directory
    pub cwd: Option<String>,
    /// Whether this is a non-interactive session
    pub is_non_interactive_session: bool,
    /// Additional options
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<ToolUseContextOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolUseContextOptions {
    /// Available tools
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
}

/// Options for permission checking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolPermissionContext {
    /// Permission mode
    pub mode: crate::permission::PermissionMode,
    /// Whether to wait for automated checks before showing dialog
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub await_automated_checks_before_dialog: Option<bool>,
}

/// Function type for checking if a tool can be used.
///
/// This is the core permission check function that determines whether
/// a tool can be executed based on the current permission settings.
///
/// # Type Parameters
/// * `Input` - The tool input type (defaults to a map of string to unknown)
///
/// # Arguments
/// * `tool` - The tool definition
/// * `input` - The input arguments for the tool
/// * `tool_use_context` - Context about the current tool use
/// * `assistant_message` - The assistant message that triggered this tool use
/// * `tool_use_id` - Unique identifier for this tool use
/// * `force_decision` - Optional forced decision (bypasses normal permission checking)
///
/// # Returns
/// A future that resolves to a permission decision
pub type CanUseToolFn<Input = std::collections::HashMap<String, serde_json::Value>> = Box<
    dyn Fn(
            ToolDefinition,
            Input,
            ToolUseContext,
            AssistantMessage,
            String,
            Option<PermissionDecision>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = PermissionDecision> + Send + 'static>,
        > + Send
        + Sync,
>;

/// Simplified CanUseToolFn that works with JSON values
pub type CanUseToolFnJson = Box<
    dyn Fn(
            ToolDefinition,
            serde_json::Value,
            ToolUseContext,
            AssistantMessage,
            String,
            Option<PermissionDecision>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = PermissionDecision> + Send + 'static>,
        > + Send
        + Sync,
>;

/// Build the can_use_tool function signature for documentation
pub const CAN_USE_TOOL_FN_SIGNATURE: &str = r#"
CanUseToolFn<Input> = Fn(
    tool: ToolDefinition,
    input: Input,
    tool_use_context: ToolUseContext,
    assistant_message: AssistantMessage,
    tool_use_id: String,
    force_decision: Option<PermissionDecision>,
) -> impl Future<Output = PermissionDecision>
"#;

/// Helper to create a default CanUseToolFn that uses the permission module
pub fn create_default_can_use_tool_fn(
    permission_context: ToolPermissionContext,
) -> CanUseToolFnJson {
    Box::new(
        move |tool: ToolDefinition,
              input: serde_json::Value,
              _tool_use_context: ToolUseContext,
              _assistant_message: AssistantMessage,
              _tool_use_id: String,
              force_decision: Option<PermissionDecision>| {
            let ctx =
                crate::permission::PermissionContext::new().with_mode(permission_context.mode);

            Box::pin(async move {
                // If force_decision is provided, use it directly
                if let Some(decision) = force_decision {
                    return decision;
                }

                // Check using permission context
                let result = ctx.check_tool(&tool.name, Some(&input));

                // Convert result to decision
                match result {
                    crate::permission::PermissionResult::Allow(allow) => {
                        PermissionDecision::Allow(crate::permission::PermissionAllowDecision {
                            behavior: allow.behavior,
                            updated_input: allow.updated_input,
                            user_modified: allow.user_modified,
                            decision_reason: allow.decision_reason,
                        })
                    }
                    crate::permission::PermissionResult::Ask(ask) => {
                        PermissionDecision::Ask(crate::permission::PermissionAskDecision {
                            behavior: ask.behavior,
                            message: ask.message,
                            updated_input: ask.updated_input,
                            decision_reason: ask.decision_reason,
                            blocked_path: ask.blocked_path,
                        })
                    }
                    crate::permission::PermissionResult::Deny(deny) => {
                        PermissionDecision::Deny(crate::permission::PermissionDenyDecision {
                            behavior: deny.behavior,
                            message: deny.message,
                            decision_reason: deny.decision_reason,
                        })
                    }
                    crate::permission::PermissionResult::Passthrough {
                        message: _,
                        decision_reason,
                    } => {
                        // Passthrough treated as allow with notification
                        PermissionDecision::Allow(crate::permission::PermissionAllowDecision {
                            behavior: crate::permission::PermissionBehavior::Allow,
                            updated_input: None,
                            user_modified: None,
                            decision_reason,
                        })
                    }
                }
            })
        },
    )
}

/// Create a CanUseToolFn that always allows
pub fn create_allow_all_can_use_tool_fn() -> CanUseToolFnJson {
    Box::new(
        |_tool: ToolDefinition,
         input: serde_json::Value,
         _context: ToolUseContext,
         _message: AssistantMessage,
         _tool_use_id: String,
         _force: Option<PermissionDecision>| {
            Box::pin(async move {
                PermissionDecision::Allow(crate::permission::PermissionAllowDecision {
                    behavior: crate::permission::PermissionBehavior::Allow,
                    updated_input: Some(input),
                    user_modified: None,
                    decision_reason: Some(crate::permission::PermissionDecisionReason::Other {
                        reason: "Allowed by default can_use_tool function".to_string(),
                    }),
                })
            })
        },
    )
}

/// Create a CanUseToolFn that always denies
pub fn create_deny_all_can_use_tool_fn() -> CanUseToolFnJson {
    Box::new(
        |tool: ToolDefinition,
         _input: serde_json::Value,
         _context: ToolUseContext,
         _message: AssistantMessage,
         _tool_use_id: String,
         _force: Option<PermissionDecision>| {
            let tool_name = tool.name.clone();
            Box::pin(async move {
                PermissionDecision::Deny(crate::permission::PermissionDenyDecision {
                    behavior: crate::permission::PermissionBehavior::Deny,
                    message: format!("Tool '{}' is denied", tool_name),
                    decision_reason: crate::permission::PermissionDecisionReason::Other {
                        reason: "Denied by default can_use_tool function".to_string(),
                    },
                })
            })
        },
    )
}

/// Create a minimal AssistantMessage for testing
#[cfg(test)]
fn create_test_assistant_message() -> AssistantMessage {
    AssistantMessage {
        message: AssistantMessageContent {
            id: "test-id".to_string(),
            container: None,
            model: "test-model".to_string(),
            role: "assistant".to_string(),
            stop_reason: None,
            stop_sequence: None,
            message_type: "message".to_string(),
            usage: None,
            content: vec![],
            context_management: None,
        },
        request_id: None,
        api_error: None,
        error: None,
        error_details: None,
        is_api_error_message: None,
        is_virtual: None,
        is_meta: None,
        advisor_model: None,
        uuid: "test-uuid".to_string(),
        timestamp: "2024-01-01".to_string(),
        parent_uuid: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_use_context_default() {
        let ctx = ToolUseContext {
            session_id: "test".to_string(),
            cwd: Some("/home".to_string()),
            is_non_interactive_session: false,
            options: None,
        };
        assert_eq!(ctx.session_id, "test");
        assert_eq!(ctx.cwd, Some("/home".to_string()));
    }

    #[test]
    fn test_tool_permission_context_default() {
        let ctx = ToolPermissionContext {
            mode: crate::permission::PermissionMode::Default,
            await_automated_checks_before_dialog: None,
        };
        assert_eq!(ctx.mode, crate::permission::PermissionMode::Default);
    }

    #[tokio::test]
    async fn test_create_default_can_use_tool_fn_allow() {
        let ctx = ToolPermissionContext {
            mode: crate::permission::PermissionMode::Bypass,
            await_automated_checks_before_dialog: None,
        };
        let fn_ptr = create_default_can_use_tool_fn(ctx);

        let tool = ToolDefinition::new(
            "Read",
            "Read files",
            crate::types::ToolInputSchema::default(),
        );
        let input = serde_json::json!({"path": "/test"});

        let result = (fn_ptr)(
            tool,
            input,
            ToolUseContext {
                session_id: "test".to_string(),
                cwd: None,
                is_non_interactive_session: false,
                options: None,
            },
            create_test_assistant_message(),
            "tool-use-1".to_string(),
            None,
        )
        .await;

        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn test_create_default_can_use_tool_fn_deny() {
        let ctx = ToolPermissionContext {
            mode: crate::permission::PermissionMode::DontAsk,
            await_automated_checks_before_dialog: None,
        };
        let fn_ptr = create_default_can_use_tool_fn(ctx);

        let tool = ToolDefinition::new(
            "Bash",
            "Run commands",
            crate::types::ToolInputSchema::default(),
        );
        let input = serde_json::json!({"command": "ls"});

        let result = (fn_ptr)(
            tool,
            input,
            ToolUseContext {
                session_id: "test".to_string(),
                cwd: None,
                is_non_interactive_session: false,
                options: None,
            },
            create_test_assistant_message(),
            "tool-use-1".to_string(),
            None,
        )
        .await;

        assert!(result.is_denied());
    }

    #[tokio::test]
    async fn test_create_allow_all_can_use_tool_fn() {
        let fn_ptr = create_allow_all_can_use_tool_fn();

        let tool = ToolDefinition::new(
            "Bash",
            "Run commands",
            crate::types::ToolInputSchema::default(),
        );
        let input = serde_json::json!({"command": "rm -rf /"});

        let result = (fn_ptr)(
            tool,
            input,
            ToolUseContext {
                session_id: "test".to_string(),
                cwd: None,
                is_non_interactive_session: false,
                options: None,
            },
            create_test_assistant_message(),
            "tool-use-1".to_string(),
            None,
        )
        .await;

        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn test_create_deny_all_can_use_tool_fn() {
        let fn_ptr = create_deny_all_can_use_tool_fn();

        let tool = ToolDefinition::new(
            "Read",
            "Read files",
            crate::types::ToolInputSchema::default(),
        );
        let input = serde_json::json!({"path": "/test"});

        let result = (fn_ptr)(
            tool,
            input,
            ToolUseContext {
                session_id: "test".to_string(),
                cwd: None,
                is_non_interactive_session: false,
                options: None,
            },
            create_test_assistant_message(),
            "tool-use-1".to_string(),
            None,
        )
        .await;

        assert!(result.is_denied());
    }

    #[tokio::test]
    async fn test_force_decision_override() {
        let ctx = ToolPermissionContext {
            mode: crate::permission::PermissionMode::Bypass,
            await_automated_checks_before_dialog: None,
        };
        let fn_ptr = create_default_can_use_tool_fn(ctx);

        let tool = ToolDefinition::new(
            "Bash",
            "Run commands",
            crate::types::ToolInputSchema::default(),
        );
        let input = serde_json::json!({"command": "ls"});

        // Force a deny decision
        let force_deny = PermissionDecision::Deny(crate::permission::PermissionDenyDecision {
            behavior: crate::permission::PermissionBehavior::Deny,
            message: "Forced deny".to_string(),
            decision_reason: crate::permission::PermissionDecisionReason::Other {
                reason: "test".to_string(),
            },
        });

        let result = (fn_ptr)(
            tool,
            input,
            ToolUseContext {
                session_id: "test".to_string(),
                cwd: None,
                is_non_interactive_session: false,
                options: None,
            },
            create_test_assistant_message(),
            "tool-use-1".to_string(),
            Some(force_deny),
        )
        .await;

        assert!(result.is_denied());
    }
}
