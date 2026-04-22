// Source: ~/claudecode/openclaudecode/src/utils/hooks/hooksConfigManager.ts
#![allow(dead_code)]

use std::collections::HashMap;

/// Hook event type
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    PermissionDenied,
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
            HookEvent::PermissionDenied => "PermissionDenied",
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
}

/// All hook events as a static slice
pub const HOOK_EVENTS: &[HookEvent] = &[
    HookEvent::PreToolUse,
    HookEvent::PostToolUse,
    HookEvent::PostToolUseFailure,
    HookEvent::PermissionDenied,
    HookEvent::Notification,
    HookEvent::UserPromptSubmit,
    HookEvent::SessionStart,
    HookEvent::SessionEnd,
    HookEvent::Stop,
    HookEvent::StopFailure,
    HookEvent::SubagentStart,
    HookEvent::SubagentStop,
    HookEvent::PreCompact,
    HookEvent::PostCompact,
    HookEvent::PermissionRequest,
    HookEvent::Setup,
    HookEvent::TeammateIdle,
    HookEvent::TaskCreated,
    HookEvent::TaskCompleted,
    HookEvent::Elicitation,
    HookEvent::ElicitationResult,
    HookEvent::ConfigChange,
    HookEvent::WorktreeCreate,
    HookEvent::WorktreeRemove,
    HookEvent::InstructionsLoaded,
    HookEvent::CwdChanged,
    HookEvent::FileChanged,
];

/// Metadata for a hook event matcher
#[derive(Debug, Clone)]
pub struct MatcherMetadata {
    pub field_to_match: String,
    pub values: Vec<String>,
}

/// Metadata for a hook event
#[derive(Debug, Clone)]
pub struct HookEventMetadata {
    pub summary: String,
    pub description: String,
    pub matcher_metadata: Option<MatcherMetadata>,
}

/// Individual hook configuration
#[derive(Debug, Clone)]
pub struct IndividualHookConfig {
    pub event: HookEvent,
    pub config: HookCommand,
    pub matcher: Option<String>,
    pub source: HookSource,
    pub plugin_name: Option<String>,
}

/// Hook command types
#[derive(Debug, Clone)]
pub enum HookCommand {
    Command {
        command: String,
        shell: Option<String>,
        if_condition: Option<String>,
        timeout: Option<u64>,
    },
    Prompt {
        prompt: String,
        if_condition: Option<String>,
        timeout: Option<u64>,
    },
    Agent {
        prompt: String,
        model: Option<String>,
        if_condition: Option<String>,
        timeout: Option<u64>,
    },
    Http {
        url: String,
        if_condition: Option<String>,
        timeout: Option<u64>,
    },
}

/// Hook source
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HookSource {
    UserSettings,
    ProjectSettings,
    LocalSettings,
    PluginHook,
    SessionHook,
    BuiltinHook,
}

impl std::fmt::Display for HookSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookSource::UserSettings => write!(f, "User Settings"),
            HookSource::ProjectSettings => write!(f, "Project Settings"),
            HookSource::LocalSettings => write!(f, "Local Settings"),
            HookSource::PluginHook => write!(f, "Plugin Hooks"),
            HookSource::SessionHook => write!(f, "Session Hooks"),
            HookSource::BuiltinHook => write!(f, "Built-in Hooks"),
        }
    }
}

/// Get display text for a hook
pub fn get_hook_display_text(hook: &HookCommand) -> String {
    match hook {
        HookCommand::Command { command, .. } => command.clone(),
        HookCommand::Prompt { prompt, .. } => prompt.clone(),
        HookCommand::Agent { prompt, .. } => prompt.clone(),
        HookCommand::Http { url, .. } => url.clone(),
    }
}

/// Get hook event metadata for all events
pub fn get_hook_event_metadata(tool_names: &[String]) -> HashMap<HookEvent, HookEventMetadata> {
    let mut metadata = HashMap::new();

    metadata.insert(
        HookEvent::PreToolUse,
        HookEventMetadata {
            summary: "Before tool execution".to_string(),
            description: "Input to command is JSON of tool call arguments.\nExit code 0 - stdout/stderr not shown\nExit code 2 - show stderr to model and block tool call\nOther exit codes - show stderr to user only but continue with tool call".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "tool_name".to_string(),
                values: tool_names.to_vec(),
            }),
        },
    );

    metadata.insert(
        HookEvent::PostToolUse,
        HookEventMetadata {
            summary: "After tool execution".to_string(),
            description: "Input to command is JSON with fields \"inputs\" (tool call arguments) and \"response\" (tool call response).\nExit code 0 - stdout shown in transcript mode (ctrl+o)\nExit code 2 - show stderr to model immediately\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "tool_name".to_string(),
                values: tool_names.to_vec(),
            }),
        },
    );

    metadata.insert(
        HookEvent::PostToolUseFailure,
        HookEventMetadata {
            summary: "After tool execution fails".to_string(),
            description: "Input to command is JSON with tool_name, tool_input, tool_use_id, error, error_type, is_interrupt, and is_timeout.\nExit code 0 - stdout shown in transcript mode (ctrl+o)\nExit code 2 - show stderr to model immediately\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "tool_name".to_string(),
                values: tool_names.to_vec(),
            }),
        },
    );

    metadata.insert(
        HookEvent::PermissionDenied,
        HookEventMetadata {
            summary: "After auto mode classifier denies a tool call".to_string(),
            description: "Input to command is JSON with tool_name, tool_input, tool_use_id, and reason.\nReturn {\"hookSpecificOutput\":{\"hookEventName\":\"PermissionDenied\",\"retry\":true}} to tell the model it may retry.\nExit code 0 - stdout shown in transcript mode (ctrl+o)\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "tool_name".to_string(),
                values: tool_names.to_vec(),
            }),
        },
    );

    metadata.insert(
        HookEvent::Notification,
        HookEventMetadata {
            summary: "When notifications are sent".to_string(),
            description: "Input to command is JSON with notification message and type.\nExit code 0 - stdout/stderr not shown\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "notification_type".to_string(),
                values: vec![
                    "permission_prompt".to_string(),
                    "idle_prompt".to_string(),
                    "auth_success".to_string(),
                    "elicitation_dialog".to_string(),
                    "elicitation_complete".to_string(),
                    "elicitation_response".to_string(),
                ],
            }),
        },
    );

    metadata.insert(
        HookEvent::UserPromptSubmit,
        HookEventMetadata {
            summary: "When the user submits a prompt".to_string(),
            description: "Input to command is JSON with original user prompt text.\nExit code 0 - stdout shown to Claude\nExit code 2 - block processing, erase original prompt, and show stderr to user only\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: None,
        },
    );

    metadata.insert(
        HookEvent::SessionStart,
        HookEventMetadata {
            summary: "When a new session is started".to_string(),
            description: "Input to command is JSON with session start source.\nExit code 0 - stdout shown to Claude\nBlocking errors are ignored\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "source".to_string(),
                values: vec![
                    "startup".to_string(),
                    "resume".to_string(),
                    "clear".to_string(),
                    "compact".to_string(),
                ],
            }),
        },
    );

    metadata.insert(
        HookEvent::Stop,
        HookEventMetadata {
            summary: "Right before Claude concludes its response".to_string(),
            description: "Exit code 0 - stdout/stderr not shown\nExit code 2 - show stderr to model and continue conversation\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: None,
        },
    );

    metadata.insert(
        HookEvent::StopFailure,
        HookEventMetadata {
            summary: "When the turn ends due to an API error".to_string(),
            description: "Fires instead of Stop when an API error (rate limit, auth failure, etc.) ended the turn. Fire-and-forget — hook output and exit codes are ignored.".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "error".to_string(),
                values: vec![
                    "rate_limit".to_string(),
                    "authentication_failed".to_string(),
                    "billing_error".to_string(),
                    "invalid_request".to_string(),
                    "server_error".to_string(),
                    "max_output_tokens".to_string(),
                    "unknown".to_string(),
                ],
            }),
        },
    );

    metadata.insert(
        HookEvent::SubagentStart,
        HookEventMetadata {
            summary: "When a subagent (Agent tool call) is started".to_string(),
            description: "Input to command is JSON with agent_id and agent_type.\nExit code 0 - stdout shown to subagent\nBlocking errors are ignored\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "agent_type".to_string(),
                values: Vec::new(),
            }),
        },
    );

    metadata.insert(
        HookEvent::SubagentStop,
        HookEventMetadata {
            summary: "Right before a subagent (Agent tool call) concludes its response".to_string(),
            description: "Input to command is JSON with agent_id, agent_type, and agent_transcript_path.\nExit code 0 - stdout/stderr not shown\nExit code 2 - show stderr to subagent and continue having it run\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "agent_type".to_string(),
                values: Vec::new(),
            }),
        },
    );

    metadata.insert(
        HookEvent::PreCompact,
        HookEventMetadata {
            summary: "Before conversation compaction".to_string(),
            description: "Input to command is JSON with compaction details.\nExit code 0 - stdout appended as custom compact instructions\nExit code 2 - block compaction\nOther exit codes - show stderr to user only but continue with compaction".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "trigger".to_string(),
                values: vec!["manual".to_string(), "auto".to_string()],
            }),
        },
    );

    metadata.insert(
        HookEvent::PostCompact,
        HookEventMetadata {
            summary: "After conversation compaction".to_string(),
            description: "Input to command is JSON with compaction details and the summary.\nExit code 0 - stdout shown to user\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "trigger".to_string(),
                values: vec!["manual".to_string(), "auto".to_string()],
            }),
        },
    );

    metadata.insert(
        HookEvent::SessionEnd,
        HookEventMetadata {
            summary: "When a session is ending".to_string(),
            description: "Input to command is JSON with session end reason.\nExit code 0 - command completes successfully\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "reason".to_string(),
                values: vec![
                    "clear".to_string(),
                    "logout".to_string(),
                    "prompt_input_exit".to_string(),
                    "other".to_string(),
                ],
            }),
        },
    );

    metadata.insert(
        HookEvent::PermissionRequest,
        HookEventMetadata {
            summary: "When a permission dialog is displayed".to_string(),
            description: "Input to command is JSON with tool_name, tool_input, and tool_use_id.\nOutput JSON with hookSpecificOutput containing decision to allow or deny.\nExit code 0 - use hook decision if provided\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "tool_name".to_string(),
                values: tool_names.to_vec(),
            }),
        },
    );

    metadata.insert(
        HookEvent::Setup,
        HookEventMetadata {
            summary: "Repo setup hooks for init and maintenance".to_string(),
            description: "Input to command is JSON with trigger (init or maintenance).\nExit code 0 - stdout shown to Claude\nBlocking errors are ignored\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "trigger".to_string(),
                values: vec!["init".to_string(), "maintenance".to_string()],
            }),
        },
    );

    metadata.insert(
        HookEvent::TeammateIdle,
        HookEventMetadata {
            summary: "When a teammate is about to go idle".to_string(),
            description: "Input to command is JSON with teammate_name and team_name.\nExit code 0 - stdout/stderr not shown\nExit code 2 - show stderr to teammate and prevent idle (teammate continues working)\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: None,
        },
    );

    metadata.insert(
        HookEvent::TaskCreated,
        HookEventMetadata {
            summary: "When a task is being created".to_string(),
            description: "Input to command is JSON with task_id, task_subject, task_description, teammate_name, and team_name.\nExit code 0 - stdout/stderr not shown\nExit code 2 - show stderr to model and prevent task creation\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: None,
        },
    );

    metadata.insert(
        HookEvent::TaskCompleted,
        HookEventMetadata {
            summary: "When a task is being marked as completed".to_string(),
            description: "Input to command is JSON with task_id, task_subject, task_description, teammate_name, and team_name.\nExit code 0 - stdout/stderr not shown\nExit code 2 - show stderr to model and prevent task completion\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: None,
        },
    );

    metadata.insert(
        HookEvent::Elicitation,
        HookEventMetadata {
            summary: "When an MCP server requests user input (elicitation)".to_string(),
            description: "Input to command is JSON with mcp_server_name, message, and requested_schema.\nOutput JSON with hookSpecificOutput containing action (accept/decline/cancel) and optional content.\nExit code 0 - use hook response if provided\nExit code 2 - deny the elicitation\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "mcp_server_name".to_string(),
                values: Vec::new(),
            }),
        },
    );

    metadata.insert(
        HookEvent::ElicitationResult,
        HookEventMetadata {
            summary: "After a user responds to an MCP elicitation".to_string(),
            description: "Input to command is JSON with mcp_server_name, action, content, mode, and elicitation_id.\nOutput JSON with hookSpecificOutput containing optional action and content to override the response.\nExit code 0 - use hook response if provided\nExit code 2 - block the response (action becomes decline)\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "mcp_server_name".to_string(),
                values: Vec::new(),
            }),
        },
    );

    metadata.insert(
        HookEvent::ConfigChange,
        HookEventMetadata {
            summary: "When configuration files change during a session".to_string(),
            description: "Input to command is JSON with source (user_settings, project_settings, local_settings, policy_settings, skills) and file_path.\nExit code 0 - allow the change\nExit code 2 - block the change from being applied to the session\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "source".to_string(),
                values: vec![
                    "user_settings".to_string(),
                    "project_settings".to_string(),
                    "local_settings".to_string(),
                    "policy_settings".to_string(),
                    "skills".to_string(),
                ],
            }),
        },
    );

    metadata.insert(
        HookEvent::InstructionsLoaded,
        HookEventMetadata {
            summary: "When an instruction file (CLAUDE.md or rule) is loaded".to_string(),
            description: "Input to command is JSON with file_path, memory_type (User, Project, Local, Managed), load_reason (session_start, nested_traversal, path_glob_match, include, compact), globs (optional — the paths: frontmatter patterns that matched), trigger_file_path (optional — the file Claude touched that caused the load), and parent_file_path (optional — the file that @-included this one).\nExit code 0 - command completes successfully\nOther exit codes - show stderr to user only\nThis hook is observability-only and does not support blocking.".to_string(),
            matcher_metadata: Some(MatcherMetadata {
                field_to_match: "load_reason".to_string(),
                values: vec![
                    "session_start".to_string(),
                    "nested_traversal".to_string(),
                    "path_glob_match".to_string(),
                    "include".to_string(),
                    "compact".to_string(),
                ],
            }),
        },
    );

    metadata.insert(
        HookEvent::WorktreeCreate,
        HookEventMetadata {
            summary: "Create an isolated worktree for VCS-agnostic isolation".to_string(),
            description: "Input to command is JSON with name (suggested worktree slug).\nStdout should contain the absolute path to the created worktree directory.\nExit code 0 - worktree created successfully\nOther exit codes - worktree creation failed".to_string(),
            matcher_metadata: None,
        },
    );

    metadata.insert(
        HookEvent::WorktreeRemove,
        HookEventMetadata {
            summary: "Remove a previously created worktree".to_string(),
            description: "Input to command is JSON with worktree_path (absolute path to worktree).\nExit code 0 - worktree removed successfully\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: None,
        },
    );

    metadata.insert(
        HookEvent::CwdChanged,
        HookEventMetadata {
            summary: "After the working directory changes".to_string(),
            description: "Input to command is JSON with old_cwd and new_cwd.\nCLAUDE_ENV_FILE is set — write bash exports there to apply env to subsequent BashTool commands.\nHook output can include hookSpecificOutput.watchPaths (array of absolute paths) to register with the FileChanged watcher.\nExit code 0 - command completes successfully\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: None,
        },
    );

    metadata.insert(
        HookEvent::FileChanged,
        HookEventMetadata {
            summary: "When a watched file changes".to_string(),
            description: "Input to command is JSON with file_path and event (change, add, unlink).\nCLAUDE_ENV_FILE is set — write bash exports there to apply env to subsequent BashTool commands.\nThe matcher field specifies filenames to watch in the current directory (e.g. \".envrc|.env\").\nHook output can include hookSpecificOutput.watchPaths (array of absolute paths) to dynamically update the watch list.\nExit code 0 - command completes successfully\nOther exit codes - show stderr to user only".to_string(),
            matcher_metadata: None,
        },
    );

    metadata
}

/// Group hooks by event and matcher
pub fn group_hooks_by_event_and_matcher(
    tool_names: &[String],
) -> HashMap<HookEvent, HashMap<String, Vec<IndividualHookConfig>>> {
    let mut grouped: HashMap<HookEvent, HashMap<String, Vec<IndividualHookConfig>>> =
        HashMap::new();

    // Initialize all events with empty maps
    for event in HOOK_EVENTS {
        grouped.insert(event.clone(), HashMap::new());
    }

    let metadata = get_hook_event_metadata(tool_names);

    // In a real implementation, this would gather hooks from:
    // 1. Settings files (user, project, local)
    // 2. Registered hooks (e.g., plugin hooks)
    // 3. Session hooks

    grouped
}

/// Sort matchers by priority for a specific event
pub fn sort_matchers_by_priority(
    matchers: &[String],
    _hooks_by_event_and_matcher: &HashMap<HookEvent, HashMap<String, Vec<IndividualHookConfig>>>,
    _event: &HookEvent,
) -> Vec<String> {
    // In the TS version, this sorts by source priority:
    // userSettings > projectSettings > localSettings > pluginHook
    // For now, just return sorted alphabetically
    let mut sorted = matchers.to_vec();
    sorted.sort();
    sorted
}

/// Get hooks for a specific event and matcher
pub fn get_hooks_for_matcher(
    hooks_by_event_and_matcher: &HashMap<HookEvent, HashMap<String, Vec<IndividualHookConfig>>>,
    event: &HookEvent,
    matcher: Option<&str>,
) -> Vec<IndividualHookConfig> {
    let matcher_key = matcher.unwrap_or("");
    hooks_by_event_and_matcher
        .get(event)
        .and_then(|event_map| event_map.get(matcher_key))
        .cloned()
        .unwrap_or_default()
}

/// Get metadata for a specific event's matcher
pub fn get_matcher_metadata(event: &HookEvent, tool_names: &[String]) -> Option<MatcherMetadata> {
    let metadata = get_hook_event_metadata(tool_names);
    metadata.get(event).and_then(|m| m.matcher_metadata.clone())
}

/// Hook source description display string
pub fn hook_source_description_display_string(source: &HookSource) -> String {
    match source {
        HookSource::UserSettings => "User settings (~/.claude/settings.json)".to_string(),
        HookSource::ProjectSettings => "Project settings (.claude/settings.json)".to_string(),
        HookSource::LocalSettings => "Local settings (.claude/settings.local.json)".to_string(),
        HookSource::PluginHook => "Plugin hooks (~/.claude/plugins/*/hooks/hooks.json)".to_string(),
        HookSource::SessionHook => "Session hooks (in-memory, temporary)".to_string(),
        HookSource::BuiltinHook => {
            "Built-in hook (registered internally by Claude Code)".to_string()
        }
    }
}

/// Hook source header display string
pub fn hook_source_header_display_string(source: &HookSource) -> String {
    match source {
        HookSource::UserSettings => "User Settings".to_string(),
        HookSource::ProjectSettings => "Project Settings".to_string(),
        HookSource::LocalSettings => "Local Settings".to_string(),
        HookSource::PluginHook => "Plugin Hooks".to_string(),
        HookSource::SessionHook => "Session Hooks".to_string(),
        HookSource::BuiltinHook => "Built-in Hooks".to_string(),
    }
}

/// Hook source inline display string
pub fn hook_source_inline_display_string(source: &HookSource) -> String {
    match source {
        HookSource::UserSettings => "User".to_string(),
        HookSource::ProjectSettings => "Project".to_string(),
        HookSource::LocalSettings => "Local".to_string(),
        HookSource::PluginHook => "Plugin".to_string(),
        HookSource::SessionHook => "Session".to_string(),
        HookSource::BuiltinHook => "Built-in".to_string(),
    }
}

/// Check if two hooks are equal (comparing only command/prompt content, not timeout)
pub fn is_hook_equal(a: &HookCommand, b: &HookCommand) -> bool {
    // We only compare command/prompt content, not timeout
    // `if` is part of identity: same command with different `if` conditions
    // are distinct hooks
    match (a, b) {
        (
            HookCommand::Command {
                command: cmd_a,
                shell: shell_a,
                if_condition: if_a,
                ..
            },
            HookCommand::Command {
                command: cmd_b,
                shell: shell_b,
                if_condition: if_b,
                ..
            },
        ) => {
            cmd_a == cmd_b
                && (shell_a.clone().unwrap_or_else(|| "bash".to_string())
                    == shell_b.clone().unwrap_or_else(|| "bash".to_string()))
                && (if_a.clone().unwrap_or_default() == if_b.clone().unwrap_or_default())
        }
        (
            HookCommand::Prompt {
                prompt: p_a,
                if_condition: if_a,
                ..
            },
            HookCommand::Prompt {
                prompt: p_b,
                if_condition: if_b,
                ..
            },
        ) => p_a == p_b && (if_a.clone().unwrap_or_default() == if_b.clone().unwrap_or_default()),
        (
            HookCommand::Agent {
                prompt: p_a,
                if_condition: if_a,
                ..
            },
            HookCommand::Agent {
                prompt: p_b,
                if_condition: if_b,
                ..
            },
        ) => p_a == p_b && (if_a.clone().unwrap_or_default() == if_b.clone().unwrap_or_default()),
        (
            HookCommand::Http {
                url: u_a,
                if_condition: if_a,
                ..
            },
            HookCommand::Http {
                url: u_b,
                if_condition: if_b,
                ..
            },
        ) => u_a == u_b && (if_a.clone().unwrap_or_default() == if_b.clone().unwrap_or_default()),
        _ => false,
    }
}
