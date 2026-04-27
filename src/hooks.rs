// Source: /data/home/swei/claudecode/openclaudecode/src/commands/hooks/hooks.tsx
//         /data/home/swei/claudecode/openclaudecode/src/schemas/hooks.ts
//         /data/home/swei/claudecode/openclaudecode/src/types/hooks.ts
//         /data/home/swei/claudecode/openclaudecode/src/utils/hooks/execHttpHook.ts
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::process::Command;
use tokio::time::{Duration, timeout};

/// All supported hook events.
pub const HOOK_EVENTS: &[&str] = &[
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "Notification",
    "UserPromptSubmit",
    "SessionStart",
    "SessionEnd",
    "Stop",
    "StopFailure",
    "SubagentStart",
    "SubagentStop",
    "PreCompact",
    "PostCompact",
    "PermissionRequest",
    "PermissionDenied",
    "Setup",
    "TeammateIdle",
    "TaskCreated",
    "TaskCompleted",
    "Elicitation",
    "ElicitationResult",
    "ConfigChange",
    "WorktreeCreate",
    "WorktreeRemove",
    "InstructionsLoaded",
    "CwdChanged",
    "FileChanged",
];

/// Reasons for session end.
pub const EXIT_REASONS: &[&str] = &[
    "clear",
    "resume",
    "logout",
    "prompt_input_exit",
    "other",
    "bypass_permissions_disabled",
];

/// Reasons for loading instructions.
pub const INSTRUCTIONS_LOAD_REASONS: &[&str] = &[
    "session_start",
    "nested_traversal",
    "path_glob_match",
    "include",
    "compact",
];

/// Types of instructions memory.
pub const INSTRUCTIONS_MEMORY_TYPES: &[&str] = &["User", "Project", "Local", "Managed"];

/// Sources of config changes.
pub const CONFIG_CHANGE_SOURCES: &[&str] = &[
    "user_settings",
    "project_settings",
    "local_settings",
    "policy_settings",
    "skills",
];

/// Default hook timeout in milliseconds (30s for shell, 60s for agent, 10m for HTTP).
pub const DEFAULT_SHELL_TIMEOUT_MS: u64 = 30_000;
pub const DEFAULT_AGENT_TIMEOUT_S: u64 = 60;
pub const DEFAULT_HTTP_TIMEOUT_MS: u64 = 600_000;

/// All supported hook events (enum form).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    Notification,
    UserPromptSubmit,
    SessionStart,
    SessionEnd,
    Stop,
    StopFailure,
    SubagentStart,
    SubagentStop,
    PreCompact,
    PostCompact,
    PermissionRequest,
    PermissionDenied,
    Setup,
    TeammateIdle,
    TaskCreated,
    TaskCompleted,
    Elicitation,
    ElicitationResult,
    ConfigChange,
    WorktreeCreate,
    WorktreeRemove,
    InstructionsLoaded,
    CwdChanged,
    FileChanged,
}

impl HookEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookEvent::PreToolUse => "PreToolUse",
            HookEvent::PostToolUse => "PostToolUse",
            HookEvent::PostToolUseFailure => "PostToolUseFailure",
            HookEvent::Notification => "Notification",
            HookEvent::UserPromptSubmit => "UserPromptSubmit",
            HookEvent::SessionStart => "SessionStart",
            HookEvent::SessionEnd => "SessionEnd",
            HookEvent::Stop => "Stop",
            HookEvent::StopFailure => "StopFailure",
            HookEvent::SubagentStart => "SubagentStart",
            HookEvent::SubagentStop => "SubagentStop",
            HookEvent::PreCompact => "PreCompact",
            HookEvent::PostCompact => "PostCompact",
            HookEvent::PermissionRequest => "PermissionRequest",
            HookEvent::PermissionDenied => "PermissionDenied",
            HookEvent::Setup => "Setup",
            HookEvent::TeammateIdle => "TeammateIdle",
            HookEvent::TaskCreated => "TaskCreated",
            HookEvent::TaskCompleted => "TaskCompleted",
            HookEvent::Elicitation => "Elicitation",
            HookEvent::ElicitationResult => "ElicitationResult",
            HookEvent::ConfigChange => "ConfigChange",
            HookEvent::WorktreeCreate => "WorktreeCreate",
            HookEvent::WorktreeRemove => "WorktreeRemove",
            HookEvent::InstructionsLoaded => "InstructionsLoaded",
            HookEvent::CwdChanged => "CwdChanged",
            HookEvent::FileChanged => "FileChanged",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "PreToolUse" => Some(HookEvent::PreToolUse),
            "PostToolUse" => Some(HookEvent::PostToolUse),
            "PostToolUseFailure" => Some(HookEvent::PostToolUseFailure),
            "Notification" => Some(HookEvent::Notification),
            "UserPromptSubmit" => Some(HookEvent::UserPromptSubmit),
            "SessionStart" => Some(HookEvent::SessionStart),
            "SessionEnd" => Some(HookEvent::SessionEnd),
            "Stop" => Some(HookEvent::Stop),
            "StopFailure" => Some(HookEvent::StopFailure),
            "SubagentStart" => Some(HookEvent::SubagentStart),
            "SubagentStop" => Some(HookEvent::SubagentStop),
            "PreCompact" => Some(HookEvent::PreCompact),
            "PostCompact" => Some(HookEvent::PostCompact),
            "PermissionRequest" => Some(HookEvent::PermissionRequest),
            "PermissionDenied" => Some(HookEvent::PermissionDenied),
            "Setup" => Some(HookEvent::Setup),
            "TeammateIdle" => Some(HookEvent::TeammateIdle),
            "TaskCreated" => Some(HookEvent::TaskCreated),
            "TaskCompleted" => Some(HookEvent::TaskCompleted),
            "Elicitation" => Some(HookEvent::Elicitation),
            "ElicitationResult" => Some(HookEvent::ElicitationResult),
            "ConfigChange" => Some(HookEvent::ConfigChange),
            "WorktreeCreate" => Some(HookEvent::WorktreeCreate),
            "WorktreeRemove" => Some(HookEvent::WorktreeRemove),
            "InstructionsLoaded" => Some(HookEvent::InstructionsLoaded),
            "CwdChanged" => Some(HookEvent::CwdChanged),
            "FileChanged" => Some(HookEvent::FileChanged),
            _ => None,
        }
    }
}

/// Shell types for hook execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HookShell {
    Bash,
    PowerShell,
}

impl Default for HookShell {
    fn default() -> Self {
        HookShell::Bash
    }
}

/// Hook type discriminator (matches TS discriminated union).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HookType {
    Command,
    Prompt,
    Agent,
    Http,
}

impl Default for HookType {
    fn default() -> Self {
        HookType::Command
    }
}

/// Permission behavior for hook decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionBehavior {
    Ask,
    Deny,
    Allow,
    Passthrough,
}

/// Event-specific hook output sub-schema (mirrors TS hookSpecificOutput).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "hookEventName")]
pub enum HookSpecificOutput {
    #[serde(rename = "PreToolUse")]
    PreToolUse {
        #[serde(skip_serializing_if = "Option::is_none")]
        permission_decision: Option<PermissionBehavior>,
        #[serde(skip_serializing_if = "Option::is_none")]
        permission_decision_reason: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        updated_input: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        additional_context: Option<String>,
    },
    #[serde(rename = "UserPromptSubmit")]
    UserPromptSubmit {
        #[serde(skip_serializing_if = "Option::is_none")]
        additional_context: Option<String>,
    },
    #[serde(rename = "SessionStart")]
    SessionStart {
        #[serde(skip_serializing_if = "Option::is_none")]
        additional_context: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        initial_user_message: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        watch_paths: Option<Vec<String>>,
    },
    #[serde(rename = "Setup")]
    Setup {
        #[serde(skip_serializing_if = "Option::is_none")]
        additional_context: Option<String>,
    },
    #[serde(rename = "SubagentStart")]
    SubagentStart {
        #[serde(skip_serializing_if = "Option::is_none")]
        additional_context: Option<String>,
    },
    #[serde(rename = "PostToolUse")]
    PostToolUse {
        #[serde(skip_serializing_if = "Option::is_none")]
        additional_context: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        updated_mcp_tool_output: Option<serde_json::Value>,
    },
    #[serde(rename = "PostToolUseFailure")]
    PostToolUseFailure {
        #[serde(skip_serializing_if = "Option::is_none")]
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
        additional_context: Option<String>,
    },
    #[serde(rename = "PermissionRequest")]
    PermissionRequest {
        #[serde(skip_serializing_if = "Option::is_none")]
        decision: Option<PermissionRequestDecision>,
    },
    #[serde(rename = "Elicitation")]
    Elicitation {
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<serde_json::Value>,
    },
    #[serde(rename = "ElicitationResult")]
    ElicitationResult {
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<serde_json::Value>,
    },
    #[serde(rename = "CwdChanged")]
    CwdChanged {
        #[serde(skip_serializing_if = "Option::is_none")]
        watch_paths: Option<Vec<String>>,
    },
    #[serde(rename = "FileChanged")]
    FileChanged {
        #[serde(skip_serializing_if = "Option::is_none")]
        watch_paths: Option<Vec<String>>,
    },
    #[serde(rename = "WorktreeCreate")]
    WorktreeCreate {
        worktree_path: String,
    },
}

/// Permission request decision from hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum PermissionRequestDecision {
    Allow {
        behavior: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        updated_input: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        updated_permissions: Option<Vec<PermissionUpdate>>,
    },
    Deny {
        behavior: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        interrupt: Option<bool>,
    },
}

/// Full hook output (mirrors TS syncHookResponseSchema).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_execution: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_specific_output: Option<HookSpecificOutput>,
    // Legacy fields for backwards compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_update: Option<PermissionUpdate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification: Option<Notification>,
}

/// Async hook output -- hook signals it wants to run in background.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsyncHookOutput {
    #[serde(rename = "async")]
    pub async_run: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub async_timeout: Option<u64>,
}

/// Hook definition (supports all TS hook types: command, prompt, agent, http).
/// Translated from HookCommand discriminated union in schemas/hooks.ts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum HookCommand {
    #[serde(rename = "command")]
    Command(CommandHookParams),
    #[serde(rename = "prompt")]
    Prompt(PromptHookParams),
    #[serde(rename = "agent")]
    Agent(AgentHookParams),
    #[serde(rename = "http")]
    Http(HttpHookParams),
}

/// Parameters for a shell command hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandHookParams {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#if: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<HookShell>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub once: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub async_run: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub async_rewake: Option<bool>,
}

/// Parameters for an LLM prompt hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptHookParams {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#if: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub once: Option<bool>,
}

/// Parameters for an agentic verifier hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentHookParams {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#if: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub once: Option<bool>,
}

/// Parameters for an HTTP webhook hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpHookParams {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#if: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_env_vars: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub once: Option<bool>,
}

/// Legacy hook definition -- kept for backwards compatibility with code
/// that constructs HookDefinition directly.
#[derive(Debug, Clone)]
pub struct HookDefinition {
    /// Shell command to execute
    pub command: Option<String>,
    /// Timeout in milliseconds
    pub timeout: Option<u64>,
    /// Tool name matcher (regex pattern)
    pub matcher: Option<String>,
}

impl From<HookCommand> for HookDefinition {
    fn from(cmd: HookCommand) -> Self {
        match cmd {
            HookCommand::Command(p) => HookDefinition {
                command: Some(p.command),
                timeout: p.timeout,
                matcher: None,
            },
            HookCommand::Prompt(_) | HookCommand::Agent(_) | HookCommand::Http(_) => {
                HookDefinition {
                    command: None,
                    timeout: None,
                    matcher: None,
                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for HookDefinition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct HookDef {
            command: Option<String>,
            timeout: Option<u64>,
            matcher: Option<String>,
        }

        let def = HookDef::deserialize(deserializer)?;
        Ok(HookDefinition {
            command: def.command,
            timeout: def.timeout.or(Some(DEFAULT_SHELL_TIMEOUT_MS)),
            matcher: def.matcher,
        })
    }
}

/// Hook matcher configuration (from TS HookMatcherSchema).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookMatcher {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matcher: Option<String>,
    pub hooks: Vec<HookCommand>,
}

/// Hook configuration (from settings) -- HashMap<event, matchers>.
pub type HookConfig = HashMap<String, Vec<HookDefinition>>;
pub type HookMatcherConfig = HashMap<String, Vec<HookMatcher>>;

/// Hook input passed to handlers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookInput {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_output: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    // Event-specific fields (mirrors TS event-specific HookInput types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_server_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_source: Option<String>,
}

impl HookInput {
    pub fn new(event: &str) -> Self {
        Self {
            event: event.to_string(),
            tool_name: None,
            tool_input: None,
            tool_output: None,
            tool_use_id: None,
            session_id: None,
            cwd: None,
            error: None,
            source: None,
            reason: None,
            final_text: None,
            agent_id: None,
            agent_type: None,
            trigger: None,
            old_cwd: None,
            file_path: None,
            file_event: None,
            mcp_server_name: None,
            requested_schema: None,
            config_source: None,
        }
    }
}

/// Permission update for hook output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionUpdate {
    pub tool: String,
    pub behavior: String,
}

/// Notification for hook output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub title: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
}

/// The result of a single hook execution.
#[derive(Debug, Clone)]
pub struct HookResult {
    pub outcome: HookOutcome,
    pub output: Option<HookOutput>,
    pub blocking_error: Option<String>,
    pub prevent_continuation: Option<bool>,
    pub stop_reason: Option<String>,
    pub additional_context: Option<String>,
}

/// Hook execution outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookOutcome {
    Success,
    Blocking,
    NonBlockingError,
    Cancelled,
}

/// Hook registry for managing and executing hooks.
#[derive(Debug, Default, Clone)]
pub struct HookRegistry {
    /// Legacy shell-command hooks
    hooks: HashMap<String, Vec<HookDefinition>>,
    /// Typed hooks (command, prompt, agent, http)
    typed_hooks: HashMap<String, Vec<HookMatcher>>,
}

impl HookRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
            typed_hooks: HashMap::new(),
        }
    }

    /// Register hooks from legacy configuration.
    pub fn register_from_config(&mut self, config: HookConfig) {
        for (event, definitions) in config {
            if !HOOK_EVENTS.contains(&event.as_str()) {
                continue;
            }
            let existing = self.hooks.entry(event).or_insert_with(Vec::new);
            existing.extend(definitions);
        }
    }

    /// Register typed hooks from matcher configuration.
    pub fn register_from_matcher_config(&mut self, config: HookMatcherConfig) {
        for (event, matchers) in config {
            if !HOOK_EVENTS.contains(&event.as_str()) {
                continue;
            }
            let existing = self.typed_hooks.entry(event).or_insert_with(Vec::new);
            existing.extend(matchers);
        }
    }

    /// Register a single legacy hook.
    pub fn register(&mut self, event: &str, definition: HookDefinition) {
        if !HOOK_EVENTS.contains(&event) {
            return;
        }
        let existing = self.hooks.entry(event.to_string()).or_insert_with(Vec::new);
        existing.push(definition);
    }

    /// Register a typed hook matcher.
    pub fn register_matcher(&mut self, event: &str, matcher: HookMatcher) {
        if !HOOK_EVENTS.contains(&event) {
            return;
        }
        let existing = self.typed_hooks.entry(event.to_string()).or_insert_with(Vec::new);
        existing.push(matcher);
    }

    /// Execute hooks for an event (runs both legacy and typed hooks in parallel).
    pub async fn execute(&self, event: &str, mut input: HookInput) -> Vec<HookOutput> {
        input.event = event.to_string();

        // Collect all async hook futures as pinned boxed trait objects
        let mut futures: Vec<Pin<Box<dyn futures_util::Future<Output = Option<HookOutput>> + Send>>> =
            Vec::new();

        // Legacy shell-command hooks
        if let Some(definitions) = self.hooks.get(event) {
            for def in definitions {
                if let Some(matcher) = &def.matcher {
                    if let Some(tool_name) = &input.tool_name {
                        if let Ok(re) = regex::Regex::new(matcher) {
                            if !re.is_match(tool_name) {
                                continue;
                            }
                        }
                    }
                }

                if let Some(command) = &def.command {
                    let fut = execute_hook_def(def.clone(), &input);
                    futures.push(Box::pin(fut));
                }
            }
        }

        // Typed hooks (command, prompt, agent, http)
        if let Some(matchers) = self.typed_hooks.get(event) {
            for matcher in matchers {
                if let Some(matcher_pattern) = &matcher.matcher {
                    if let Some(tool_name) = &input.tool_name {
                        if !tool_name.contains(matcher_pattern.as_str()) {
                            continue;
                        }
                    }
                }

                for hook_cmd in &matcher.hooks {
                    if let Some(cond) = hook_cmd.if_condition() {
                        if !check_if_condition(cond, &input) {
                            continue;
                        }
                    }

                    let fut = execute_hook_command(hook_cmd.clone(), &input);
                    futures.push(Box::pin(fut));
                }
            }
        }

        // Execute all hooks in parallel
        let results = futures_util::future::join_all(futures).await;
        results.into_iter().flatten().collect()
    }

    /// Check if any hooks are registered for an event.
    pub fn has_hooks(&self, event: &str) -> bool {
        self.hooks
            .get(event)
            .map(|h| !h.is_empty())
            .unwrap_or(false)
            || self.typed_hooks.get(event).map(|h| !h.is_empty()).unwrap_or(false)
    }

    /// Clear all hooks.
    pub fn clear(&mut self) {
        self.hooks.clear();
        self.typed_hooks.clear();
    }
}

/// Get the `if` condition from a HookCommand (if any).
impl HookCommand {
    pub fn if_condition(&self) -> Option<&str> {
        match self {
            HookCommand::Command(p) => p.r#if.as_deref(),
            HookCommand::Prompt(p) => p.r#if.as_deref(),
            HookCommand::Agent(p) => p.r#if.as_deref(),
            HookCommand::Http(p) => p.r#if.as_deref(),
        }
    }

    pub fn status_message(&self) -> Option<&str> {
        match self {
            HookCommand::Command(p) => p.status_message.as_deref(),
            HookCommand::Prompt(p) => p.status_message.as_deref(),
            HookCommand::Agent(p) => p.status_message.as_deref(),
            HookCommand::Http(p) => p.status_message.as_deref(),
        }
    }

    pub fn timeout_ms(&self) -> u64 {
        match self {
            HookCommand::Command(p) => p.timeout.unwrap_or(DEFAULT_SHELL_TIMEOUT_MS),
            HookCommand::Prompt(p) => p.timeout.unwrap_or(DEFAULT_SHELL_TIMEOUT_MS),
            HookCommand::Agent(p) => {
                p.timeout.unwrap_or(DEFAULT_AGENT_TIMEOUT_S) * 1000
            }
            HookCommand::Http(p) => p.timeout.unwrap_or(DEFAULT_HTTP_TIMEOUT_MS),
        }
    }

    pub fn is_once(&self) -> bool {
        match self {
            HookCommand::Command(p) => p.once.unwrap_or(false),
            HookCommand::Prompt(p) => p.once.unwrap_or(false),
            HookCommand::Agent(p) => p.once.unwrap_or(false),
            HookCommand::Http(p) => p.once.unwrap_or(false),
        }
    }

    pub fn is_async(&self) -> bool {
        match self {
            HookCommand::Command(p) => p.async_run.unwrap_or(false),
            _ => false,
        }
    }
}

/// Check an `if` condition against hook input.
/// Uses simple permission-rule-style matching: "ToolName(pattern)" or "ToolName".
/// Translated from prepareIfConditionMatcher in TS.
fn check_if_condition(cond: &str, input: &HookInput) -> bool {
    let cond = cond.trim();
    if cond.is_empty() {
        return true;
    }

    // Parse "ToolName(pattern)" or just "ToolName"
    if let Some(paren_start) = cond.find('(') {
        let paren_end = cond.rfind(')');
        if let Some(paren_end) = paren_end {
            let tool_part = &cond[..paren_start];
            let pattern = &cond[paren_start + 1..paren_end];

            // Check tool name
            if let Some(tool_name) = &input.tool_name {
                if !tool_name.contains(tool_part) {
                    return false;
                }
            } else {
                return false;
            }

            // Check input pattern (simple glob/wildcard match)
            if let Some(tool_input) = &input.tool_input {
                let input_str = tool_input.to_string();
                if !matches_pattern(pattern, &input_str) {
                    return false;
                }
            }
            true
        } else {
            // Malformed condition -- run anyway
            true
        }
    } else {
        // Just a tool name prefix
        if let Some(tool_name) = &input.tool_name {
            tool_name.contains(cond)
        } else {
            false
        }
    }
}

/// Simple pattern matching: "*" matches anything, other patterns are substring match.
fn matches_pattern(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    // Split pattern by '*' and check each segment appears in order
    let segments: Vec<&str> = pattern.split('*').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return true;
    }
    let mut pos = 0;
    for segment in &segments {
        if let Some(found) = text[pos..].find(*segment) {
            pos = pos + found + segment.len();
        } else {
            return false;
        }
    }
    true
}

/// Execute a legacy HookDefinition.
async fn execute_hook_def(
    def: HookDefinition,
    input: &HookInput,
) -> Option<HookOutput> {
    if let Some(command) = &def.command {
        let shell = HookShell::Bash;
        execute_shell_hook(&command, &shell, input, def.timeout.unwrap_or(DEFAULT_SHELL_TIMEOUT_MS))
            .await
            .ok()
            .flatten()
    } else {
        None
    }
}

/// Execute a typed HookCommand.
async fn execute_hook_command(cmd: HookCommand, input: &HookInput) -> Option<HookOutput> {
    match cmd {
        HookCommand::Command(params) => {
            let shell = params.shell.clone().unwrap_or_default();
            execute_shell_hook(&params.command, &shell, input, params.timeout.unwrap_or(DEFAULT_SHELL_TIMEOUT_MS))
                .await
                .ok()
                .flatten()
        }
        HookCommand::Http(params) => {
            execute_http_hook(&params, input).await.ok().flatten()
        }
HookCommand::Prompt(params) => {
            let hook = crate::utils::hooks::PromptHook {
                prompt: params.prompt.clone(),
                timeout: params.timeout,
                model: params.model.clone(),
            };
            let tool_use_context = std::sync::Arc::new(
                crate::utils::hooks::can_use_tool::ToolUseContext {
                    session_id: input.session_id.clone().unwrap_or_default(),
                    cwd: input.cwd.clone(),
                    is_non_interactive_session: false,
                    options: None,
                }
            );
            let (_signal_tx, signal_rx) = tokio::sync::watch::channel(false);
            let hook_name = format!("prompt:{}", input.event);
            let json_input = serde_json::to_string(input).unwrap_or_default();

            match crate::utils::hooks::exec_prompt_hook(
                &hook,
                &hook_name,
                &input.event,
                &json_input,
                signal_rx,
                tool_use_context,
                None,
                input.tool_use_id.clone(),
            )
            .await
            {
                crate::utils::hooks::ExecPromptHookResult::Success { .. } => {
                    Some(HookOutput {
                        continue_execution: Some(true),
                        suppress_output: Some(true),
                        stop_reason: None,
                        decision: Some("allow".to_string()),
                        reason: None,
                        system_message: None,
                        hook_specific_output: None,
                        message: None,
                        permission_update: None,
                        notification: None,
                        block: None,
                    })
                }
                crate::utils::hooks::ExecPromptHookResult::Blocking {
                    blocking_error, ..
                } => {
                    Some(HookOutput {
                        continue_execution: Some(false),
                        suppress_output: Some(false),
                        stop_reason: Some(blocking_error),
                        decision: Some("deny".to_string()),
                        reason: None,
                        system_message: None,
                        hook_specific_output: None,
                        message: None,
                        permission_update: None,
                        notification: None,
                        block: Some(true),
                    })
                }
                crate::utils::hooks::ExecPromptHookResult::Cancelled => {
                    Some(HookOutput {
                        continue_execution: Some(true),
                        suppress_output: Some(false),
                        stop_reason: None,
                        decision: None,
                        reason: Some("hook timed out".to_string()),
                        system_message: None,
                        hook_specific_output: None,
                        message: Some("Prompt hook cancelled/timeout".to_string()),
                        permission_update: None,
                        notification: None,
                        block: None,
                    })
                }
                crate::utils::hooks::ExecPromptHookResult::NonBlockingError { stderr, .. } => {
                    Some(HookOutput {
                        continue_execution: Some(true),
                        suppress_output: Some(false),
                        stop_reason: None,
                        decision: None,
                        reason: Some(stderr),
                        system_message: None,
                        hook_specific_output: None,
                        message: None,
                        permission_update: None,
                        notification: None,
                        block: None,
                    })
                }
            }
        }
HookCommand::Agent(params) => {
            let hook = crate::utils::hooks::exec_agent_hook::AgentHook {
                prompt: params.prompt.clone(),
                timeout: params.timeout,
                model: params.model.clone(),
            };
            let tool_use_context = std::sync::Arc::new(
                crate::utils::hooks::can_use_tool::ToolUseContext {
                    session_id: input.session_id.clone().unwrap_or_default(),
                    cwd: input.cwd.clone(),
                    is_non_interactive_session: false,
                    options: None,
                }
            );
            let (_signal_tx, signal_rx) = tokio::sync::watch::channel(false);
            let hook_name = format!("agent:{}", input.event);
            let json_input = serde_json::to_string(input).unwrap_or_default();

            match crate::utils::hooks::exec_agent_hook(
                &hook,
                &hook_name,
                &input.event,
                &json_input,
                signal_rx,
                tool_use_context,
                None,
                &[],
                None,
            )
            .await
            {
                crate::utils::hooks::ExecAgentHookResult::Success { .. } => {
                    Some(HookOutput {
                        continue_execution: Some(true),
                        suppress_output: Some(true),
                        stop_reason: None,
                        decision: Some("allow".to_string()),
                        reason: None,
                        system_message: None,
                        hook_specific_output: None,
                        message: None,
                        permission_update: None,
                        notification: None,
                        block: None,
                    })
                }
                crate::utils::hooks::ExecAgentHookResult::Blocking {
                    blocking_error, ..
                } => {
                    Some(HookOutput {
                        continue_execution: Some(false),
                        suppress_output: Some(false),
                        stop_reason: Some(blocking_error),
                        decision: Some("deny".to_string()),
                        reason: None,
                        system_message: None,
                        hook_specific_output: None,
                        message: None,
                        permission_update: None,
                        notification: None,
                        block: Some(true),
                    })
                }
                crate::utils::hooks::ExecAgentHookResult::Cancelled => {
                    Some(HookOutput {
                        continue_execution: Some(true),
                        suppress_output: Some(false),
                        stop_reason: None,
                        decision: None,
                        reason: Some("hook cancelled".to_string()),
                        system_message: None,
                        hook_specific_output: None,
                        message: Some("Agent hook cancelled".to_string()),
                        permission_update: None,
                        notification: None,
                        block: None,
                    })
                }
                crate::utils::hooks::ExecAgentHookResult::NonBlockingError { stderr, .. } => {
                    Some(HookOutput {
                        continue_execution: Some(true),
                        suppress_output: Some(false),
                        stop_reason: None,
                        decision: None,
                        reason: Some(stderr),
                        system_message: None,
                        hook_specific_output: None,
                        message: None,
                        permission_update: None,
                        notification: None,
                        block: None,
                    })
                }
            }
        }
    }
}

/// Execute a shell command as a hook.
async fn execute_shell_hook(
    command: &str,
    shell: &HookShell,
    input: &HookInput,
    timeout_ms: u64,
) -> Result<Option<HookOutput>, crate::error::AgentError> {
    let input_json = serde_json::to_string(input).map_err(crate::error::AgentError::Json)?;

    // Clone data needed in the blocking task
    let cmd_str = command.to_string();
    let event = input.event.clone();
    let tool_name = input.tool_name.clone();
    let session_id = input.session_id.clone();
    let cwd = input.cwd.clone();
    let project_dir = crate::utils::get_original_cwd()
        .to_string_lossy()
        .to_string();
    let shell = shell.clone();

    let result = timeout(
        Duration::from_millis(timeout_ms),
        tokio::task::spawn_blocking(move || {
            let (prog, args) = match shell {
                HookShell::Bash => ("bash", vec!["-c".to_string(), cmd_str.clone()]),
                HookShell::PowerShell => ("pwsh", vec![
                    "-NoProfile".to_string(),
                    "-NonInteractive".to_string(),
                    "-Command".to_string(),
                    cmd_str.clone(),
                ]),
            };

            let mut cmd = Command::new(prog);
            cmd.args(&args)
                .env("HOOK_EVENT", &event)
                .env("HOOK_TOOL_NAME", tool_name.as_deref().unwrap_or(""))
                .env("HOOK_SESSION_ID", session_id.as_deref().unwrap_or(""))
                .env("HOOK_CWD", cwd.as_deref().unwrap_or(""))
                .env("HOOK_PROJECT_DIR", &project_dir)
                .env("HOOK_INPUT", &input_json)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            let mut child = cmd.spawn()?;

            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(input_json.as_bytes())?;
            }

            let output = child.wait_with_output()?;

            // Exit code semantics (mirrors TS):
            // 0 -> success
            // 2 -> blocking error
            // other -> non-blocking error
            if !output.status.success() {
                let stderr_msg = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let exit_code = if output.status.code().is_some() {
                    output.status.code().unwrap()
                } else {
                    -1
                };
                if exit_code == 2 {
                    // Blocking error
                    return Ok(Some(HookOutput {
                        continue_execution: Some(false),
                        suppress_output: None,
                        stop_reason: Some(stderr_msg.clone()),
                        decision: None,
                        reason: None,
                        system_message: None,
                        hook_specific_output: None,
                        message: Some(stderr_msg),
                        block: Some(true),
                        permission_update: None,
                        notification: None,
                    }));
                }
                // Non-blocking error -- still return output
                return Ok(Some(HookOutput {
                    continue_execution: None,
                    suppress_output: None,
                    stop_reason: None,
                    decision: None,
                    reason: None,
                    system_message: None,
                    hook_specific_output: None,
                    message: Some(stderr_msg),
                    block: None,
                    permission_update: None,
                    notification: None,
                }));
            }

            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if stdout.is_empty() {
                return Ok(None);
            }

            // Check for async protocol: {"async": true}
            if let Ok(async_out) = serde_json::from_str::<AsyncHookOutput>(&stdout) {
                if async_out.async_run {
                    return Ok(Some(HookOutput {
                        continue_execution: Some(true),
                        suppress_output: Some(true),
                        stop_reason: None,
                        decision: None,
                        reason: None,
                        system_message: None,
                        hook_specific_output: None,
                        message: Some("Hook running in background".to_string()),
                        block: None,
                        permission_update: None,
                        notification: None,
                    }));
                }
            }

            // Try to parse as JSON
            if let Ok(hook_output) = serde_json::from_str::<HookOutput>(&stdout) {
                Ok(Some(hook_output))
            } else {
                // Non-JSON output treated as message
                Ok(Some(HookOutput {
                    message: Some(stdout),
                    permission_update: None,
                    block: None,
                    notification: None,
                    continue_execution: None,
                    suppress_output: None,
                    stop_reason: None,
                    decision: None,
                    reason: None,
                    system_message: None,
                    hook_specific_output: None,
                }))
            }
        }),
    )
    .await;

    match result {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => {
            let err = std::io::Error::new(std::io::ErrorKind::Other, e.to_string());
            Err(crate::error::AgentError::Io(err))
        }
        Err(_) => {
            let err = std::io::Error::new(std::io::ErrorKind::TimedOut, "Hook timeout");
            Err(crate::error::AgentError::Io(err))
        }
    }
}

/// Execute an HTTP hook (POST to webhook URL).
/// Translated from execHttpHook in TypeScript.
async fn execute_http_hook(
    params: &HttpHookParams,
    input: &HookInput,
) -> Result<Option<HookOutput>, crate::error::AgentError> {
    let mut url = params.url.clone();

    // Sanitize CRLF header injection
    if url.contains('\r') || url.contains('\n') {
        return Err(crate::error::AgentError::Internal(format!(
            "HTTP hook URL contains disallowed characters: {}",
            &url
        )));
    }

    // Basic URL validation
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(crate::error::AgentError::Internal(format!(
            "HTTP hook URL must start with http:// or https://: {}",
            url
        )));
    }

    let body = serde_json::to_string(input).map_err(crate::error::AgentError::Json)?;

    // Build headers
    let mut header_map = reqwest::header::HeaderMap::new();
    header_map.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    if let Some(custom_headers) = &params.headers {
        for (key, val) in custom_headers {
            // Interpolate environment variables
            let interpolated = interpolate_env_vars(val, &params.allowed_env_vars);
            // Sanitize CRLF injection in headers
            if interpolated.contains('\r') || interpolated.contains('\n') {
                continue;
            }
            if let Ok(header_val) = reqwest::header::HeaderValue::from_str(&interpolated) {
                if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
                    header_map.insert(header_name, header_val);
                }
            }
        }
    }

    let timeout_s = params.timeout.unwrap_or(DEFAULT_HTTP_TIMEOUT_MS / 1000) as u64;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(if timeout_s == 0 { 600 } else { timeout_s }))
        .build()
        .map_err(|e| crate::error::AgentError::Internal(format!("Failed to build HTTP client: {}", e)))?;

    let response = client
        .post(&url)
        .headers(header_map)
        .body(body)
        .send()
        .await
        .map_err(|e| crate::error::AgentError::Internal(format!("HTTP hook request failed: {}", e)))?;

    let status = response.status();
    let body = response.text().await.map_err(|e| {
        crate::error::AgentError::Internal(format!("Failed to read HTTP hook response: {}", e))
    })?;

    if !status.is_success() {
        return Ok(Some(HookOutput {
            message: Some(format!("HTTP hook returned status {}: {}", status, body)),
            block: Some(status.as_u16() >= 500),
            continue_execution: None,
            suppress_output: None,
            stop_reason: None,
            decision: None,
            reason: None,
            system_message: None,
            hook_specific_output: None,
            permission_update: None,
            notification: None,
        }));
    }

    // Try to parse response as HookOutput
    if let Ok(output) = serde_json::from_str::<HookOutput>(&body) {
        Ok(Some(output))
    } else if !body.trim().is_empty() {
        Ok(Some(HookOutput {
            message: Some(body),
            block: None,
            continue_execution: None,
            suppress_output: None,
            stop_reason: None,
            decision: None,
            reason: None,
            system_message: None,
            hook_specific_output: None,
            permission_update: None,
            notification: None,
        }))
    } else {
        Ok(None)
    }
}

/// Interpolate environment variables in header values.
/// Only resolves variables listed in `allowed_env_vars`.
fn interpolate_env_vars(
    value: &str,
    allowed_env_vars: &Option<Vec<String>>,
) -> String {
    // If no allowed vars, don't interpolate anything
    if allowed_env_vars.is_none() || allowed_env_vars.as_ref().unwrap().is_empty() {
        return value.to_string();
    }

    let mut result = value.to_string();
    for var in allowed_env_vars.as_ref().unwrap() {
        // Match $VAR_NAME or ${VAR_NAME}
        let dollar_var = format!("${}", var);
        let brace_var = format!("${{{}}}", var);
        if let Ok(env_val) = std::env::var(var) {
            result = result.replace(&dollar_var, &env_val).replace(&brace_var, &env_val);
        } else {
            result = result.replace(&dollar_var, "").replace(&brace_var, "");
        }
    }
    result
}

/// Create a default hook registry.
pub fn create_hook_registry(config: Option<HookConfig>) -> HookRegistry {
    let mut registry = HookRegistry::new();
    if let Some(c) = config {
        registry.register_from_config(c);
    }
    registry
}

/// Result of running Stop hooks.
#[derive(Debug, Default)]
pub struct StopHookResult {
    pub prevent_continuation: bool,
    pub blocking_errors: Vec<String>,
}

/// Free function: Run PreToolUse hooks from a registry.
/// Returns Ok(true) if any hook blocked, Ok(false) otherwise.
pub async fn run_pre_tool_use_hooks(
    registry: &HookRegistry,
    tool_name: &str,
    tool_input: &serde_json::Value,
    tool_use_id: &str,
    cwd: &str,
) -> Result<bool, crate::error::AgentError> {
    if !registry.has_hooks("PreToolUse") {
        return Ok(false);
    }
    let input = HookInput {
        event: "PreToolUse".to_string(),
        tool_name: Some(tool_name.to_string()),
        tool_input: Some(tool_input.clone()),
        tool_output: None,
        tool_use_id: Some(tool_use_id.to_string()),
        session_id: None,
        cwd: Some(cwd.to_string()),
        error: None,
        source: None,
        reason: None,
        final_text: None,
        agent_id: None,
        agent_type: None,
        trigger: None,
        old_cwd: None,
        file_path: None,
        file_event: None,
        mcp_server_name: None,
        requested_schema: None,
        config_source: None,
    };
    let results = registry.execute("PreToolUse", input).await;
    for output in results {
        if output.block == Some(true) {
            return Err(crate::error::AgentError::Tool(format!(
                "Tool '{}' blocked by PreToolUse hook",
                tool_name
            )));
        }
    }
    Ok(false)
}

/// Free function: Run PostToolUse hooks from a registry.
pub async fn run_post_tool_use_hooks(
    registry: &HookRegistry,
    tool_name: &str,
    tool_output: &crate::types::ToolResult,
    tool_use_id: &str,
    cwd: &str,
) {
    if !registry.has_hooks("PostToolUse") {
        return;
    }
    let input = HookInput {
        event: "PostToolUse".to_string(),
        tool_name: Some(tool_name.to_string()),
        tool_input: None,
        tool_output: Some(serde_json::json!({
            "result_type": tool_output.result_type,
            "content": tool_output.content,
            "is_error": tool_output.is_error,
        })),
        tool_use_id: Some(tool_use_id.to_string()),
        session_id: None,
        cwd: Some(cwd.to_string()),
        error: None,
        source: None,
        reason: None,
        final_text: None,
        agent_id: None,
        agent_type: None,
        trigger: None,
        old_cwd: None,
        file_path: None,
        file_event: None,
        mcp_server_name: None,
        requested_schema: None,
        config_source: None,
    };
    let _ = registry.execute("PostToolUse", input).await;
}

/// Free function: Run PostToolUseFailure hooks from a registry.
pub async fn run_post_tool_use_failure_hooks(
    registry: &HookRegistry,
    tool_name: &str,
    error: &str,
    tool_use_id: &str,
    cwd: &str,
) {
    if !registry.has_hooks("PostToolUseFailure") {
        return;
    }
    let input = HookInput {
        event: "PostToolUseFailure".to_string(),
        tool_name: Some(tool_name.to_string()),
        tool_input: None,
        tool_output: None,
        tool_use_id: Some(tool_use_id.to_string()),
        session_id: None,
        cwd: Some(cwd.to_string()),
        error: Some(error.to_string()),
        source: None,
        reason: None,
        final_text: None,
        agent_id: None,
        agent_type: None,
        trigger: None,
        old_cwd: None,
        file_path: None,
        file_event: None,
        mcp_server_name: None,
        requested_schema: None,
        config_source: None,
    };
    let _ = registry.execute("PostToolUseFailure", input).await;
}

/// Free function: Run Stop hooks from a registry.
/// Returns prevent_continuation and any blocking error messages.
pub async fn run_stop_hooks(
    registry: &HookRegistry,
    cwd: &str,
    final_text: &str,
) -> StopHookResult {
    if !registry.has_hooks("Stop") {
        return StopHookResult::default();
    }
    let input = HookInput {
        event: "Stop".to_string(),
        tool_name: None,
        tool_input: None,
        tool_output: Some(serde_json::json!({ "text": final_text })),
        tool_use_id: None,
        session_id: None,
        cwd: Some(cwd.to_string()),
        error: None,
        source: None,
        reason: None,
        final_text: Some(final_text.to_string()),
        agent_id: None,
        agent_type: None,
        trigger: None,
        old_cwd: None,
        file_path: None,
        file_event: None,
        mcp_server_name: None,
        requested_schema: None,
        config_source: None,
    };
    let results = registry.execute("Stop", input).await;
    let mut blocking_errors = Vec::new();
    for output in results {
        if output.block == Some(true) {
            if let Some(msg) = output.message {
                blocking_errors.push(msg);
            }
        }
    }
    StopHookResult {
        prevent_continuation: blocking_errors.is_empty(),
        blocking_errors,
    }
}

/// Free function: Run StopFailure hooks (fire-and-forget).
pub async fn run_stop_failure_hooks(
    registry: &HookRegistry,
    error: &str,
    cwd: &str,
) {
    if !registry.has_hooks("StopFailure") {
        return;
    }
    let input = HookInput {
        event: "StopFailure".to_string(),
        tool_name: None,
        tool_input: None,
        tool_output: None,
        tool_use_id: None,
        session_id: None,
        cwd: Some(cwd.to_string()),
        error: Some(error.to_string()),
        source: None,
        reason: None,
        final_text: None,
        agent_id: None,
        agent_type: None,
        trigger: None,
        old_cwd: None,
        file_path: None,
        file_event: None,
        mcp_server_name: None,
        requested_schema: None,
        config_source: None,
    };
    let _ = registry.execute("StopFailure", input).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_event_as_str() {
        assert_eq!(HookEvent::PreToolUse.as_str(), "PreToolUse");
        assert_eq!(HookEvent::PostToolUse.as_str(), "PostToolUse");
        assert_eq!(HookEvent::SessionStart.as_str(), "SessionStart");
    }

    #[test]
    fn test_hook_event_from_str() {
        assert_eq!(
            HookEvent::from_str("PreToolUse"),
            Some(HookEvent::PreToolUse)
        );
        assert_eq!(HookEvent::from_str("Invalid"), None);
    }

    #[test]
    fn test_hook_events_constant() {
        assert!(HOOK_EVENTS.contains(&"PreToolUse"));
        assert!(HOOK_EVENTS.contains(&"PostToolUse"));
        assert!(HOOK_EVENTS.contains(&"SessionStart"));
    }

    #[test]
    fn test_hook_registry_new() {
        let registry = HookRegistry::new();
        assert!(!registry.has_hooks("PreToolUse"));
    }

    #[test]
    fn test_hook_registry_register() {
        let mut registry = HookRegistry::new();
        registry.register(
            "PreToolUse",
            HookDefinition {
                command: Some("echo test".to_string()),
                timeout: Some(5000),
                matcher: Some("Read.*".to_string()),
            },
        );
        assert!(registry.has_hooks("PreToolUse"));
    }

    #[test]
    fn test_hook_registry_clear() {
        let mut registry = HookRegistry::new();
        registry.register(
            "PreToolUse",
            HookDefinition {
                command: Some("echo test".to_string()),
                timeout: None,
                matcher: None,
            },
        );
        registry.clear();
        assert!(!registry.has_hooks("PreToolUse"));
    }

    #[test]
    fn test_hook_input_new() {
        let input = HookInput::new("PreToolUse");
        assert_eq!(input.event, "PreToolUse");
    }

    #[test]
    fn test_hook_output_serialization() {
        let output = HookOutput {
            message: Some("test message".to_string()),
            permission_update: None,
            block: Some(true),
            notification: None,
            continue_execution: None,
            suppress_output: None,
            stop_reason: None,
            decision: None,
            reason: None,
            system_message: None,
            hook_specific_output: None,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("test message"));
    }

    #[test]
    fn test_create_hook_registry() {
        let registry = create_hook_registry(None);
        assert!(!registry.has_hooks("PreToolUse"));
    }

    #[tokio::test]
    async fn test_execute_no_hooks() {
        let registry = HookRegistry::new();
        let input = HookInput::new("PreToolUse");
        let results = registry.execute("PreToolUse", input).await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_execute_with_invalid_event() {
        let registry = HookRegistry::new();
        let input = HookInput::new("InvalidEvent");
        let results = registry.execute("InvalidEvent", input).await;
        assert!(results.is_empty());
    }

    #[test]
    fn test_check_if_condition_exact_tool() {
        let input = HookInput {
            tool_name: Some("Bash".to_string()),
            ..HookInput::new("PreToolUse")
        };
        assert!(check_if_condition("Bash", &input));
        assert!(!check_if_condition("Read", &input));
    }

    #[test]
    fn test_check_if_condition_with_pattern() {
        let input = HookInput {
            tool_name: Some("Bash".to_string()),
            tool_input: Some(serde_json::json!({"command": "git status"})),
            ..HookInput::new("PreToolUse")
        };
        assert!(check_if_condition("Bash(git)", &input));
        assert!(!check_if_condition("Bash(npm)", &input));
    }

    #[test]
    fn test_check_if_condition_wildcard() {
        let input = HookInput {
            tool_name: Some("Bash".to_string()),
            tool_input: Some(serde_json::json!({"command": "anything"})),
            ..HookInput::new("PreToolUse")
        };
        assert!(check_if_condition("Bash(*)", &input));
    }

    #[test]
    fn test_matches_pattern_glob() {
        assert!(matches_pattern("*.ts", "foo.ts"));
        assert!(!matches_pattern("*.ts", "foo.js"));
        assert!(matches_pattern("*test*", "my_test_file"));
        assert!(matches_pattern("*", "anything"));
    }

    #[test]
    fn test_hook_shell_default() {
        let shell = HookShell::default();
        assert_eq!(shell, HookShell::Bash);
    }

    #[test]
    fn test_hook_type_default() {
        let ty = HookType::default();
        assert_eq!(ty, HookType::Command);
    }

    #[test]
    fn test_hook_command_if_condition() {
        let cmd: HookCommand = serde_json::from_str(
            r#"{"type":"command","command":"echo hi","if":"Bash(git)"}"#,
        )
        .unwrap();
        assert_eq!(cmd.if_condition(), Some("Bash(git)"));
    }

    #[test]
    fn test_http_hook_params_deserialize() {
        let params: HttpHookParams = serde_json::from_str(
            r#"{"url":"https://example.com/webhook","timeout":30,"headers":{"Authorization":"Bearer $TOKEN"},"allowedEnvVars":["TOKEN"]}"#,
        )
        .unwrap();
        assert_eq!(params.url, "https://example.com/webhook");
        assert!(params.headers.is_some());
    }

    #[test]
    fn test_interpolate_env_vars() {
        unsafe {
            std::env::set_var("TEST_HOOK_VAR", "secret123");
        }
        let result =
            interpolate_env_vars("Bearer $TEST_HOOK_VAR", &Some(vec!["TEST_HOOK_VAR".to_string()]));
        assert_eq!(result, "Bearer secret123");

        // Unallowed var should not be interpolated
        unsafe {
            std::env::set_var("UNALLOWED_VAR", "leaked");
        }
        let result = interpolate_env_vars("$UNALLOWED_VAR", &Some(vec!["OTHER".to_string()]));
        assert_eq!(result, "$UNALLOWED_VAR");

        unsafe {
            std::env::remove_var("TEST_HOOK_VAR");
            std::env::remove_var("UNALLOWED_VAR");
        }
    }

    #[test]
    fn test_interpolate_env_vars_brace_syntax() {
        unsafe {
            std::env::set_var("MY_TOKEN", "abc");
        }
        let result =
            interpolate_env_vars("Bearer ${MY_TOKEN}", &Some(vec!["MY_TOKEN".to_string()]));
        assert_eq!(result, "Bearer abc");
        unsafe {
            std::env::remove_var("MY_TOKEN");
        }
    }

    #[test]
    fn test_hook_specific_output() {
        let output = HookSpecificOutput::PreToolUse {
            permission_decision: Some(PermissionBehavior::Allow),
            permission_decision_reason: None,
            updated_input: None,
            additional_context: Some("approved".to_string()),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("PreToolUse"));
        assert!(json.contains("allow"));
    }

    #[test]
    fn test_hook_matcher_deserialize() {
        let matcher: HookMatcher = serde_json::from_str(
            r#"{"matcher":"Bash","hooks":[{"type":"command","command":"echo bash"}]}"#,
        )
        .unwrap();
        assert_eq!(matcher.matcher.as_deref(), Some("Bash"));
        assert_eq!(matcher.hooks.len(), 1);
    }

    #[test]
    fn test_async_hook_output() {
        let json = r#"{"async": true, "asyncTimeout": 60}"#;
        let output: AsyncHookOutput = serde_json::from_str(json).unwrap();
        assert!(output.async_run);
        assert_eq!(output.async_timeout, Some(60));
    }

    #[test]
    fn test_hook_command_timeout_ms() {
        let cmd: HookCommand = serde_json::from_str(
            r#"{"type":"command","command":"echo hi","timeout":5}"#,
        )
        .unwrap();
        assert_eq!(cmd.timeout_ms(), 5);

        let agent_cmd: HookCommand = serde_json::from_str(
            r#"{"type":"agent","prompt":"verify"}"#,
        )
        .unwrap();
        // Default agent timeout is 60s = 60000ms
        assert_eq!(agent_cmd.timeout_ms(), 60_000);
    }
}
