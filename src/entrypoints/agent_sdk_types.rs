// Source: ~/claudecode/openclaudecode/src/entrypoints/agentSdkTypes.ts
use serde::{Deserialize, Serialize};

pub const HOOK_EVENTS: &[&str] = &[
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "UserPromptSubmit",
    "Notification",
    "PermissionRequest",
    "PermissionDenied",
    "Elicitation",
    "ElicitationResult",
    "SessionStart",
    "Setup",
    "SubagentStart",
    "CwdChanged",
    "FileChanged",
    "WorktreeCreate",
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct HookEvent;

impl HookEvent {
    pub fn as_str(&self) -> &'static str {
        "HookEvent"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookInput {
    pub tool_name: String,
    pub tool_input: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionUpdate {
    pub permission: String,
    pub allow: bool,
}
