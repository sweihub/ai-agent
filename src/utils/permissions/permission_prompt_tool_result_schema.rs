// Source: ~/claudecode/openclaudecode/src/utils/permissions/PermissionPromptToolResultSchema.ts
#![allow(dead_code)]

//! Zod schemas for permission prompt tool results, ported to Rust validation.

use crate::types::permissions::{
    PermissionAllowDecision, PermissionDecision, PermissionDecisionReason, PermissionUpdate,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Input schema for permission prompt tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPromptInput {
    pub tool_name: String,
    pub input: std::collections::HashMap<String, Value>,
    pub tool_use_id: Option<String>,
}

/// Decision classification from SDK host.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionClassification {
    UserTemporary,
    UserPermanent,
    UserReject,
}

/// Permission allow result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionAllowResult {
    pub behavior: String, // "allow"
    #[serde(rename = "updatedInput")]
    pub updated_input: std::collections::HashMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "updatedPermissions")]
    pub updated_permissions: Option<Vec<PermissionUpdate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "decisionClassification")]
    pub decision_classification: Option<DecisionClassification>,
}

/// Permission deny result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDenyResult {
    pub behavior: String, // "deny"
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupt: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "decisionClassification")]
    pub decision_classification: Option<DecisionClassification>,
}

/// Output schema — either allow or deny result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PermissionPromptOutput {
    Allow(PermissionAllowResult),
    Deny(PermissionDenyResult),
}

/// Normalizes the result of a permission prompt tool to a PermissionDecision.
pub fn permission_prompt_tool_result_to_permission_decision(
    result: PermissionPromptOutput,
    tool_name: &str,
    input: &std::collections::HashMap<String, Value>,
) -> PermissionDecision {
    let decision_reason = PermissionDecisionReason::PermissionPromptTool {
        permission_prompt_tool_name: tool_name.to_string(),
        tool_result: serde_json::to_value(&result).unwrap_or_default(),
    };

    match result {
        PermissionPromptOutput::Allow(allow) => {
            // Handle updatedInput — empty object means "use original"
            let updated_input = if allow.updated_input.is_empty() {
                input.clone()
            } else {
                allow.updated_input
            };

            PermissionDecision::Allow(PermissionAllowDecision {
                behavior: "allow".to_string(),
                updated_input: Some(updated_input),
                user_modified: None,
                decision_reason: Some(decision_reason),
                tool_use_id: allow.tool_use_id,
                accept_feedback: None,
                content_blocks: None,
            })
        }
        PermissionPromptOutput::Deny(deny) => {
            if deny.interrupt.unwrap_or(false) {
                log::debug!(
                    "SDK permission prompt deny+interrupt: tool={} message={}",
                    tool_name,
                    deny.message,
                );
            }
            PermissionDecision::Deny(crate::types::permissions::PermissionDenyDecision {
                behavior: "deny".to_string(),
                message: deny.message,
                decision_reason,
                tool_use_id: deny.tool_use_id,
            })
        }
    }
}
