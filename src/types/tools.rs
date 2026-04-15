// Source: ~/claudecode/openclaudecode/src/types/tools.ts

use serde::{Deserialize, Serialize};

/// Base tool progress data with flexible extra fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProgressData {
    #[serde(rename = "kind", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// Progress types for various tools, all sharing ToolProgressData structure.
pub type ShellProgress = ToolProgressData;
pub type BashProgress = ToolProgressData;
pub type PowerShellProgress = ToolProgressData;
pub type McpProgress = ToolProgressData;
pub type SkillToolProgress = ToolProgressData;
pub type TaskOutputProgress = ToolProgressData;
pub type WebSearchProgress = ToolProgressData;
pub type AgentToolProgress = ToolProgressData;
pub type ReplToolProgress = ToolProgressData;
pub type SdkWorkflowProgress = ToolProgressData;
