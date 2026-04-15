// Source: ~/claudecode/openclaudecode/src/types/permissions.ts

//! Pure permission type definitions extracted to break import cycles.
//! This file contains only type definitions and constants with no runtime dependencies.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Permission Modes
// ============================================================================

/// External permission modes (user-addressable)
pub const EXTERNAL_PERMISSION_MODES: &[&str] = &[
    "acceptEdits",
    "bypassPermissions",
    "default",
    "dontAsk",
    "plan",
];

/// External permission mode type.
pub type ExternalPermissionMode = String;

/// Internal permission mode includes external modes plus 'auto' and 'bubble'.
pub type InternalPermissionMode = String;

/// Union of all permission modes.
pub type PermissionMode = String;

/// Runtime validation set: modes that are user-addressable.
pub const INTERNAL_PERMISSION_MODES: &[&str] = &[
    "acceptEdits",
    "bypassPermissions",
    "default",
    "dontAsk",
    "plan",
    "auto",
];

/// All permission modes.
pub const PERMISSION_MODES: &[&str] = INTERNAL_PERMISSION_MODES;

// ============================================================================
// Permission Behaviors
// ============================================================================

/// Permission behavior enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionBehavior {
    Allow,
    Deny,
    Ask,
}

// ============================================================================
// Permission Rules
// ============================================================================

/// Where a permission rule originated from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum PermissionRuleSource {
    UserSettings,
    ProjectSettings,
    LocalSettings,
    FlagSettings,
    PolicySettings,
    CliArg,
    Command,
    Session,
}

impl PermissionRuleSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            PermissionRuleSource::UserSettings => "userSettings",
            PermissionRuleSource::ProjectSettings => "projectSettings",
            PermissionRuleSource::LocalSettings => "localSettings",
            PermissionRuleSource::FlagSettings => "flagSettings",
            PermissionRuleSource::PolicySettings => "policySettings",
            PermissionRuleSource::CliArg => "cliArg",
            PermissionRuleSource::Command => "command",
            PermissionRuleSource::Session => "session",
        }
    }
}

/// The value of a permission rule - specifies which tool and optional content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRuleValue {
    #[serde(rename = "toolName")]
    pub tool_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ruleContent")]
    pub rule_content: Option<String>,
}

/// A permission rule with its source and behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    pub source: PermissionRuleSource,
    #[serde(rename = "ruleBehavior")]
    pub rule_behavior: PermissionBehavior,
    #[serde(rename = "ruleValue")]
    pub rule_value: PermissionRuleValue,
}

// ============================================================================
// Permission Updates
// ============================================================================

/// Where a permission update should be persisted.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PermissionUpdateDestination {
    UserSettings,
    ProjectSettings,
    LocalSettings,
    Session,
    CliArg,
}

/// Update operations for permission configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PermissionUpdate {
    #[serde(rename = "addRules")]
    AddRules {
        destination: PermissionUpdateDestination,
        rules: Vec<PermissionRuleValue>,
        behavior: PermissionBehavior,
    },
    #[serde(rename = "replaceRules")]
    ReplaceRules {
        destination: PermissionUpdateDestination,
        rules: Vec<PermissionRuleValue>,
        behavior: PermissionBehavior,
    },
    #[serde(rename = "removeRules")]
    RemoveRules {
        destination: PermissionUpdateDestination,
        rules: Vec<PermissionRuleValue>,
        behavior: PermissionBehavior,
    },
    #[serde(rename = "setMode")]
    SetMode {
        destination: PermissionUpdateDestination,
        mode: ExternalPermissionMode,
    },
    #[serde(rename = "addDirectories")]
    AddDirectories {
        destination: PermissionUpdateDestination,
        directories: Vec<String>,
    },
    #[serde(rename = "removeDirectories")]
    RemoveDirectories {
        destination: PermissionUpdateDestination,
        directories: Vec<String>,
    },
}

/// Source of an additional working directory permission.
pub type WorkingDirectorySource = PermissionRuleSource;

/// An additional directory included in permission scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditionalWorkingDirectory {
    pub path: String,
    pub source: WorkingDirectorySource,
}

// ============================================================================
// Permission Decisions & Results
// ============================================================================

/// Minimal command shape for permission metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCommandMetadata {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Allow additional properties for forward compatibility
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Metadata attached to permission decisions.
pub type PermissionMetadata = Option<PermissionCommandMetadata>;

/// Result when permission is granted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionAllowDecision {
    pub behavior: String, // "allow"
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "updatedInput")]
    pub updated_input: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "userModified")]
    pub user_modified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "decisionReason")]
    pub decision_reason: Option<PermissionDecisionReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "toolUseID")]
    pub tool_use_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "acceptFeedback")]
    pub accept_feedback: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "contentBlocks")]
    pub content_blocks: Option<Vec<serde_json::Value>>,
}

/// Metadata for a pending classifier check that will run asynchronously.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingClassifierCheck {
    pub command: String,
    pub cwd: String,
    pub descriptions: Vec<String>,
}

/// Result when user should be prompted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionAskDecision {
    pub behavior: String, // "ask"
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "updatedInput")]
    pub updated_input: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "decisionReason")]
    pub decision_reason: Option<PermissionDecisionReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestions: Option<Vec<PermissionUpdate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "blockedPath")]
    pub blocked_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<PermissionMetadata>,
    /// If true, triggered by a bashCommandIsSafe security check
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isBashSecurityCheckForMisparsing")]
    pub is_bash_security_check_for_misparsing: Option<bool>,
    /// If set, an allow classifier check should be run asynchronously
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "pendingClassifierCheck")]
    pub pending_classifier_check: Option<PendingClassifierCheck>,
    /// Optional content blocks (e.g., images) to include
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "contentBlocks")]
    pub content_blocks: Option<Vec<serde_json::Value>>,
}

/// Result when permission is denied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDenyDecision {
    pub behavior: String, // "deny"
    pub message: String,
    #[serde(rename = "decisionReason")]
    pub decision_reason: PermissionDecisionReason,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "toolUseID")]
    pub tool_use_id: Option<String>,
}

/// A permission decision - allow, ask, or deny.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "behavior")]
pub enum PermissionDecision {
    #[serde(rename = "allow")]
    Allow(PermissionAllowDecision),
    #[serde(rename = "ask")]
    Ask(PermissionAskDecision),
    #[serde(rename = "deny")]
    Deny(PermissionDenyDecision),
}

/// Permission result with additional passthrough option.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "behavior")]
pub enum PermissionResult {
    #[serde(rename = "allow")]
    Allow(PermissionAllowDecision),
    #[serde(rename = "ask")]
    Ask(PermissionAskDecision),
    #[serde(rename = "deny")]
    Deny(PermissionDenyDecision),
    #[serde(rename = "passthrough")]
    Passthrough {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "decisionReason")]
        decision_reason: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        suggestions: Option<Vec<PermissionUpdate>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "blockedPath")]
        blocked_path: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "pendingClassifierCheck")]
        pending_classifier_check: Option<PendingClassifierCheck>,
    },
}

/// Explanation of why a permission decision was made.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PermissionDecisionReason {
    #[serde(rename = "rule")]
    Rule { rule: PermissionRule },
    #[serde(rename = "mode")]
    Mode { mode: PermissionMode },
    #[serde(rename = "subcommandResults")]
    SubcommandResults {
        reasons: HashMap<String, PermissionResult>,
    },
    #[serde(rename = "permissionPromptTool")]
    PermissionPromptTool {
        #[serde(rename = "permissionPromptToolName")]
        permission_prompt_tool_name: String,
        #[serde(rename = "toolResult")]
        tool_result: serde_json::Value,
    },
    #[serde(rename = "hook")]
    Hook {
        #[serde(rename = "hookName")]
        hook_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "hookSource")]
        hook_source: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    #[serde(rename = "asyncAgent")]
    AsyncAgent { reason: String },
    #[serde(rename = "sandboxOverride")]
    SandboxOverride {
        reason: SandboxOverrideReason,
    },
    #[serde(rename = "classifier")]
    Classifier {
        classifier: String,
        reason: String,
    },
    #[serde(rename = "workingDir")]
    WorkingDir { reason: String },
    #[serde(rename = "safetyCheck")]
    SafetyCheck {
        reason: String,
        /// When true, auto mode lets the classifier evaluate this instead of forcing a prompt
        #[serde(rename = "classifierApprovable")]
        classifier_approvable: bool,
    },
    #[serde(rename = "other")]
    Other { reason: String },
}

/// Sandbox override reason.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SandboxOverrideReason {
    ExcludedCommand,
    DangerouslyDisableSandbox,
}

// ============================================================================
// Bash Classifier Types
// ============================================================================

/// Classifier result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifierResult {
    pub matches: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "matchedDescription")]
    pub matched_description: Option<String>,
    pub confidence: ClassifierConfidence,
    pub reason: String,
}

/// Classifier confidence level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClassifierConfidence {
    High,
    Medium,
    Low,
}

/// Classifier behavior.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClassifierBehavior {
    Deny,
    Ask,
    Allow,
}

/// Classifier token usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifierUsage {
    #[serde(rename = "inputTokens")]
    pub input_tokens: i64,
    #[serde(rename = "outputTokens")]
    pub output_tokens: i64,
    #[serde(rename = "cacheReadInputTokens")]
    pub cache_read_input_tokens: i64,
    #[serde(rename = "cacheCreationInputTokens")]
    pub cache_creation_input_tokens: i64,
}

/// YOLO classifier result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloClassifierResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(rename = "shouldBlock")]
    pub should_block: bool,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unavailable: Option<bool>,
    /// API returned "prompt is too long"
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "transcriptTooLong")]
    pub transcript_too_long: Option<bool>,
    /// The model used for this classifier call
    pub model: String,
    /// Token usage from the classifier API call
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ClassifierUsage>,
    /// Duration of the classifier API call in ms
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "durationMs")]
    pub duration_ms: Option<u64>,
    /// Character lengths of the prompt components sent to the classifier
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "promptLengths")]
    pub prompt_lengths: Option<ClassifierPromptLengths>,
    /// Path where error prompts were dumped
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "errorDumpPath")]
    pub error_dump_path: Option<String>,
    /// Which classifier stage produced the final decision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<ClassifierStage>,
    /// Token usage from stage 1 when stage 2 was also run
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stage1Usage")]
    pub stage1_usage: Option<ClassifierUsage>,
    /// Duration of stage 1 in ms when stage 2 was also run
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stage1DurationMs")]
    pub stage1_duration_ms: Option<u64>,
    /// API request_id for stage 1
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stage1RequestId")]
    pub stage1_request_id: Option<String>,
    /// API message id for stage 1
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stage1MsgId")]
    pub stage1_msg_id: Option<String>,
    /// Token usage from stage 2 when stage 2 was run
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stage2Usage")]
    pub stage2_usage: Option<ClassifierUsage>,
    /// Duration of stage 2 in ms when stage 2 was run
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stage2DurationMs")]
    pub stage2_duration_ms: Option<u64>,
    /// API request_id for stage 2
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stage2RequestId")]
    pub stage2_request_id: Option<String>,
    /// API message id for stage 2
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stage2MsgId")]
    pub stage2_msg_id: Option<String>,
}

/// Classifier stage enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClassifierStage {
    Fast,
    Thinking,
}

/// Character lengths of prompt components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifierPromptLengths {
    #[serde(rename = "systemPrompt")]
    pub system_prompt: usize,
    #[serde(rename = "toolCalls")]
    pub tool_calls: usize,
    #[serde(rename = "userPrompts")]
    pub user_prompts: usize,
}

// ============================================================================
// Permission Explainer Types
// ============================================================================

/// Risk level enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Permission explanation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionExplanation {
    #[serde(rename = "riskLevel")]
    pub risk_level: RiskLevel,
    pub explanation: String,
    pub reasoning: String,
    pub risk: String,
}

// ============================================================================
// Tool Permission Context
// ============================================================================

/// Mapping of permission rules by their source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermissionRulesBySource {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "userSettings")]
    pub user_settings: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "projectSettings")]
    pub project_settings: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "localSettings")]
    pub local_settings: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "flagSettings")]
    pub flag_settings: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "policySettings")]
    pub policy_settings: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "cliArg")]
    pub cli_arg: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<Vec<String>>,
}

/// Context needed for permission checking in tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermissionContext {
    pub mode: PermissionMode,
    #[serde(rename = "additionalWorkingDirectories")]
    pub additional_working_directories: HashMap<String, AdditionalWorkingDirectory>,
    #[serde(rename = "alwaysAllowRules")]
    pub always_allow_rules: ToolPermissionRulesBySource,
    #[serde(rename = "alwaysDenyRules")]
    pub always_deny_rules: ToolPermissionRulesBySource,
    #[serde(rename = "alwaysAskRules")]
    pub always_ask_rules: ToolPermissionRulesBySource,
    #[serde(rename = "isBypassPermissionsModeAvailable")]
    pub is_bypass_permissions_mode_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "strippedDangerousRules")]
    pub stripped_dangerous_rules: Option<ToolPermissionRulesBySource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "shouldAvoidPermissionPrompts")]
    pub should_avoid_permission_prompts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "awaitAutomatedChecksBeforeDialog")]
    pub await_automated_checks_before_dialog: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "prePlanMode")]
    pub pre_plan_mode: Option<PermissionMode>,
}
