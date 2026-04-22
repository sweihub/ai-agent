// Source: ~/claudecode/openclaudecode/src/utils/hooks/hooksSettings.ts
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::path::Path;

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

/// All hook events
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

/// Editable setting sources
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EditableSettingSource {
    UserSettings,
    ProjectSettings,
    LocalSettings,
}

/// Setting source priority order (lower index = higher priority)
pub const SOURCES: &[EditableSettingSource] = &[
    EditableSettingSource::UserSettings,
    EditableSettingSource::ProjectSettings,
    EditableSettingSource::LocalSettings,
];

/// Hook source
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HookSource {
    Editable(EditableSettingSource),
    PolicySettings,
    PluginHook,
    SessionHook,
    BuiltinHook,
}

impl std::fmt::Display for HookSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookSource::Editable(s) => match s {
                EditableSettingSource::UserSettings => write!(f, "User"),
                EditableSettingSource::ProjectSettings => write!(f, "Project"),
                EditableSettingSource::LocalSettings => write!(f, "Local"),
            },
            HookSource::PolicySettings => write!(f, "Policy"),
            HookSource::PluginHook => write!(f, "Plugin"),
            HookSource::SessionHook => write!(f, "Session"),
            HookSource::BuiltinHook => write!(f, "Built-in"),
        }
    }
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

/// Default hook shell
pub const DEFAULT_HOOK_SHELL: &str = "bash";

/// Check if two hooks are equal (comparing only command/prompt content, not timeout)
pub fn is_hook_equal(a: &HookCommand, b: &HookCommand) -> bool {
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
                && (shell_a
                    .clone()
                    .unwrap_or_else(|| DEFAULT_HOOK_SHELL.to_string())
                    == shell_b
                        .clone()
                        .unwrap_or_else(|| DEFAULT_HOOK_SHELL.to_string()))
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

/// Get the display text for a hook
pub fn get_hook_display_text(hook: &HookCommand) -> String {
    match hook {
        HookCommand::Command { command, .. } => command.clone(),
        HookCommand::Prompt { prompt, .. } => prompt.clone(),
        HookCommand::Agent { prompt, .. } => prompt.clone(),
        HookCommand::Http { url, .. } => url.clone(),
    }
}

/// Get all hooks from all sources
pub fn get_all_hooks() -> Vec<IndividualHookConfig> {
    let mut hooks: Vec<IndividualHookConfig> = Vec::new();

    // Check if restricted to managed hooks only
    // (would check policy settings)
    let restricted_to_managed_only = false;

    if !restricted_to_managed_only {
        // Get hooks from all editable sources
        let sources = [
            EditableSettingSource::UserSettings,
            EditableSettingSource::ProjectSettings,
            EditableSettingSource::LocalSettings,
        ];

        // Track which setting files we've already processed to avoid duplicates
        let mut seen_files: HashSet<String> = HashSet::new();

        for source in sources {
            let file_path = get_settings_file_path_for_source(&source);

            if let Some(ref path) = file_path {
                let resolved_path = Path::new(path)
                    .canonicalize()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| path.clone());

                if seen_files.contains(&resolved_path) {
                    continue;
                }
                seen_files.insert(resolved_path);
            }

            // Get hooks from this source's settings
            if let Some(source_hooks) = get_settings_for_source(&source) {
                for (event_str, matchers) in source_hooks {
                    if let Ok(event) = parse_hook_event(&event_str) {
                        for matcher in matchers {
                            for hook_command in matcher.hooks {
                                hooks.push(IndividualHookConfig {
                                    event: event.clone(),
                                    config: hook_command,
                                    matcher: matcher.matcher.clone(),
                                    source: HookSource::Editable(source.clone()),
                                    plugin_name: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Get session hooks (would call get_session_hooks)
    // Session hooks are handled separately

    hooks
}

/// Get hooks for a specific event
pub fn get_hooks_for_event(event: &HookEvent) -> Vec<IndividualHookConfig> {
    get_all_hooks()
        .into_iter()
        .filter(|hook| &hook.event == event)
        .collect()
}

/// Parse a hook event from a string
fn parse_hook_event(s: &str) -> Result<HookEvent, String> {
    match s {
        "PreToolUse" => Ok(HookEvent::PreToolUse),
        "PostToolUse" => Ok(HookEvent::PostToolUse),
        "PostToolUseFailure" => Ok(HookEvent::PostToolUseFailure),
        "PermissionDenied" => Ok(HookEvent::PermissionDenied),
        "Notification" => Ok(HookEvent::Notification),
        "UserPromptSubmit" => Ok(HookEvent::UserPromptSubmit),
        "SessionStart" => Ok(HookEvent::SessionStart),
        "SessionEnd" => Ok(HookEvent::SessionEnd),
        "Stop" => Ok(HookEvent::Stop),
        "StopFailure" => Ok(HookEvent::StopFailure),
        "SubagentStart" => Ok(HookEvent::SubagentStart),
        "SubagentStop" => Ok(HookEvent::SubagentStop),
        "PreCompact" => Ok(HookEvent::PreCompact),
        "PostCompact" => Ok(HookEvent::PostCompact),
        "PermissionRequest" => Ok(HookEvent::PermissionRequest),
        "Setup" => Ok(HookEvent::Setup),
        "TeammateIdle" => Ok(HookEvent::TeammateIdle),
        "TaskCreated" => Ok(HookEvent::TaskCreated),
        "TaskCompleted" => Ok(HookEvent::TaskCompleted),
        "Elicitation" => Ok(HookEvent::Elicitation),
        "ElicitationResult" => Ok(HookEvent::ElicitationResult),
        "ConfigChange" => Ok(HookEvent::ConfigChange),
        "WorktreeCreate" => Ok(HookEvent::WorktreeCreate),
        "WorktreeRemove" => Ok(HookEvent::WorktreeRemove),
        "InstructionsLoaded" => Ok(HookEvent::InstructionsLoaded),
        "CwdChanged" => Ok(HookEvent::CwdChanged),
        "FileChanged" => Ok(HookEvent::FileChanged),
        _ => Err(format!("Unknown hook event: {}", s)),
    }
}

/// Matcher structure from settings
pub struct HookMatcher {
    pub matcher: Option<String>,
    pub hooks: Vec<HookCommand>,
}

/// Get settings file path for a source
fn get_settings_file_path_for_source(source: &EditableSettingSource) -> Option<String> {
    match source {
        EditableSettingSource::UserSettings => {
            // ~/.claude/settings.json
            dirs::home_dir().map(|home| {
                home.join(".claude")
                    .join("settings.json")
                    .to_string_lossy()
                    .to_string()
            })
        }
        EditableSettingSource::ProjectSettings => {
            // .claude/settings.json in cwd
            let cwd = std::env::current_dir().ok()?;
            Some(
                cwd.join(".claude")
                    .join("settings.json")
                    .to_string_lossy()
                    .to_string(),
            )
        }
        EditableSettingSource::LocalSettings => {
            // .claude/settings.local.json in cwd
            let cwd = std::env::current_dir().ok()?;
            Some(
                cwd.join(".claude")
                    .join("settings.local.json")
                    .to_string_lossy()
                    .to_string(),
            )
        }
    }
}

/// Get settings for a source (simplified - would read from files)
fn get_settings_for_source(
    _source: &EditableSettingSource,
) -> Option<HashMap<String, Vec<HookMatcher>>> {
    // This would read the settings JSON file for the given source
    // and extract the hooks section
    None
}

/// Hook source description display string
pub fn hook_source_description_display_string(source: &HookSource) -> String {
    match source {
        HookSource::Editable(s) => match s {
            EditableSettingSource::UserSettings => {
                "User settings (~/.claude/settings.json)".to_string()
            }
            EditableSettingSource::ProjectSettings => {
                "Project settings (.claude/settings.json)".to_string()
            }
            EditableSettingSource::LocalSettings => {
                "Local settings (.claude/settings.local.json)".to_string()
            }
        },
        HookSource::PolicySettings => "Policy settings".to_string(),
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
        HookSource::Editable(s) => match s {
            EditableSettingSource::UserSettings => "User Settings".to_string(),
            EditableSettingSource::ProjectSettings => "Project Settings".to_string(),
            EditableSettingSource::LocalSettings => "Local Settings".to_string(),
        },
        HookSource::PolicySettings => "Policy Settings".to_string(),
        HookSource::PluginHook => "Plugin Hooks".to_string(),
        HookSource::SessionHook => "Session Hooks".to_string(),
        HookSource::BuiltinHook => "Built-in Hooks".to_string(),
    }
}

/// Hook source inline display string
pub fn hook_source_inline_display_string(source: &HookSource) -> String {
    match source {
        HookSource::Editable(s) => match s {
            EditableSettingSource::UserSettings => "User".to_string(),
            EditableSettingSource::ProjectSettings => "Project".to_string(),
            EditableSettingSource::LocalSettings => "Local".to_string(),
        },
        HookSource::PolicySettings => "Policy".to_string(),
        HookSource::PluginHook => "Plugin".to_string(),
        HookSource::SessionHook => "Session".to_string(),
        HookSource::BuiltinHook => "Built-in".to_string(),
    }
}

/// Sort matchers by priority for a specific event.
/// Priority is based on source order: userSettings > projectSettings > localSettings.
/// Plugin hooks get lowest priority.
pub fn sort_matchers_by_priority(
    matchers: &[String],
    hooks_by_event_and_matcher: &HashMap<HookEvent, HashMap<String, Vec<IndividualHookConfig>>>,
    selected_event: &HookEvent,
) -> Vec<String> {
    // Create a priority map based on SOURCES order (lower index = higher priority)
    let source_priority: HashMap<EditableSettingSource, usize> = SOURCES
        .iter()
        .enumerate()
        .map(|(i, s)| (s.clone(), i))
        .collect();

    let mut sorted = matchers.to_vec();
    sorted.sort_by(|a, b| {
        let a_hooks = hooks_by_event_and_matcher
            .get(selected_event)
            .and_then(|m| m.get(a))
            .cloned()
            .unwrap_or_default();
        let b_hooks = hooks_by_event_and_matcher
            .get(selected_event)
            .and_then(|m| m.get(b))
            .cloned()
            .unwrap_or_default();

        let a_sources: HashSet<&HookSource> = a_hooks.iter().map(|h| &h.source).collect();
        let b_sources: HashSet<&HookSource> = b_hooks.iter().map(|h| &h.source).collect();

        // Sort by highest priority source first (lowest priority number)
        // Plugin hooks get lowest priority (highest number)
        let get_source_priority = |source: &&HookSource| -> usize {
            match *source {
                HookSource::PluginHook | HookSource::BuiltinHook => 999,
                HookSource::Editable(s) => *source_priority.get(s).unwrap_or(&999),
                HookSource::PolicySettings => 0, // Highest priority
                HookSource::SessionHook => 100,
            }
        };

        let a_highest_priority = a_sources
            .iter()
            .map(get_source_priority)
            .min()
            .unwrap_or(999);
        let b_highest_priority = b_sources
            .iter()
            .map(get_source_priority)
            .min()
            .unwrap_or(999);

        if a_highest_priority != b_highest_priority {
            return a_highest_priority.cmp(&b_highest_priority);
        }

        // If same priority, sort by matcher name
        a.cmp(b)
    });

    sorted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hook_equal_same_command() {
        let a = HookCommand::Command {
            command: "echo hello".to_string(),
            shell: None,
            if_condition: None,
            timeout: None,
        };
        let b = HookCommand::Command {
            command: "echo hello".to_string(),
            shell: None,
            if_condition: None,
            timeout: Some(30), // Different timeout doesn't matter
        };
        assert!(is_hook_equal(&a, &b));
    }

    #[test]
    fn test_is_hook_equal_different_command() {
        let a = HookCommand::Command {
            command: "echo hello".to_string(),
            shell: None,
            if_condition: None,
            timeout: None,
        };
        let b = HookCommand::Command {
            command: "echo world".to_string(),
            shell: None,
            if_condition: None,
            timeout: None,
        };
        assert!(!is_hook_equal(&a, &b));
    }

    #[test]
    fn test_is_hook_equal_different_types() {
        let a = HookCommand::Command {
            command: "echo hello".to_string(),
            shell: None,
            if_condition: None,
            timeout: None,
        };
        let b = HookCommand::Prompt {
            prompt: "echo hello".to_string(),
            if_condition: None,
            timeout: None,
        };
        assert!(!is_hook_equal(&a, &b));
    }

    #[test]
    fn test_get_hook_display_text() {
        let hook = HookCommand::Command {
            command: "echo hello".to_string(),
            shell: None,
            if_condition: None,
            timeout: None,
        };
        assert_eq!(get_hook_display_text(&hook), "echo hello");
    }

    #[test]
    fn test_parse_hook_event() {
        assert_eq!(parse_hook_event("Stop").unwrap(), HookEvent::Stop);
        assert_eq!(
            parse_hook_event("PreToolUse").unwrap(),
            HookEvent::PreToolUse
        );
        assert!(parse_hook_event("Unknown").is_err());
    }

    #[test]
    fn test_hook_source_display_strings() {
        let source = HookSource::Editable(EditableSettingSource::UserSettings);
        assert_eq!(
            hook_source_description_display_string(&source),
            "User settings (~/.claude/settings.json)"
        );
        assert_eq!(hook_source_header_display_string(&source), "User Settings");
        assert_eq!(hook_source_inline_display_string(&source), "User");
    }
}
