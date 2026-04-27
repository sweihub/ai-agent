// Source: ~/claudecode/openclaudecode/src/utils/hooks/hooksSettings.ts
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::Path;

use serde::de;

use crate::utils::hooks::session_hooks::get_session_hooks;
use crate::utils::settings::{get_settings_file_path_for_source, read_settings_file};

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

/// Editable setting sources - re-export from settings module.
pub use crate::utils::settings::EditableSettingSource;

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
///
/// Serialized/deserialized with `#[serde(tag = "type")]` to match the
/// TypeScript discriminated union: command | prompt | agent | http.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum HookCommand {
    #[serde(rename = "command")]
    Command {
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        shell: Option<String>,
        #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
        if_condition: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<u64>,
        #[serde(rename = "statusMessage", skip_serializing_if = "Option::is_none")]
        status_message: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        once: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        r#async: Option<bool>,
        #[serde(rename = "asyncRewake", skip_serializing_if = "Option::is_none")]
        async_rewake: Option<bool>,
    },
    #[serde(rename = "prompt")]
    Prompt {
        prompt: String,
        #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
        if_condition: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        #[serde(rename = "statusMessage", skip_serializing_if = "Option::is_none")]
        status_message: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        once: Option<bool>,
    },
    #[serde(rename = "agent")]
    Agent {
        prompt: String,
        #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
        if_condition: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        #[serde(rename = "statusMessage", skip_serializing_if = "Option::is_none")]
        status_message: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        once: Option<bool>,
    },
    #[serde(rename = "http")]
    Http {
        url: String,
        #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
        if_condition: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
        #[serde(rename = "allowedEnvVars", skip_serializing_if = "Option::is_none")]
        allowed_env_vars: Option<Vec<String>>,
        #[serde(rename = "statusMessage", skip_serializing_if = "Option::is_none")]
        status_message: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        once: Option<bool>,
    },
}

impl<'de> de::Deserialize<'de> for HookCommand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let map: serde_json::Map<String, serde_json::Value> =
            serde_json::Map::deserialize(deserializer)?;

        let type_str = map
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| de::Error::missing_field("type"))?;

        // Helper to extract optional string from map
        let opt_str = |m: &serde_json::Map<String, serde_json::Value>, key: &str| -> Option<String> {
            m.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
        };
        let opt_u64 =
            |m: &serde_json::Map<String, serde_json::Value>, key: &str| -> Option<u64> {
                m.get(key).and_then(|v| v.as_u64())
            };
        let opt_bool =
            |m: &serde_json::Map<String, serde_json::Value>, key: &str| -> Option<bool> {
                m.get(key).and_then(|v| v.as_bool())
            };

        match type_str {
            "command" => {
                let command = opt_str(&map, "command")
                    .ok_or_else(|| de::Error::missing_field("command"))?;
                Ok(HookCommand::Command {
                    command,
                    shell: opt_str(&map, "shell"),
                    if_condition: opt_str(&map, "if"),
                    timeout: opt_u64(&map, "timeout"),
                    status_message: opt_str(&map, "statusMessage"),
                    once: opt_bool(&map, "once"),
                    r#async: opt_bool(&map, "async"),
                    async_rewake: opt_bool(&map, "asyncRewake"),
                })
            }
            "prompt" => {
                let prompt = opt_str(&map, "prompt")
                    .ok_or_else(|| de::Error::missing_field("prompt"))?;
                Ok(HookCommand::Prompt {
                    prompt,
                    if_condition: opt_str(&map, "if"),
                    timeout: opt_u64(&map, "timeout"),
                    model: opt_str(&map, "model"),
                    status_message: opt_str(&map, "statusMessage"),
                    once: opt_bool(&map, "once"),
                })
            }
            "agent" => {
                let prompt = opt_str(&map, "prompt")
                    .ok_or_else(|| de::Error::missing_field("prompt"))?;
                Ok(HookCommand::Agent {
                    prompt,
                    if_condition: opt_str(&map, "if"),
                    timeout: opt_u64(&map, "timeout"),
                    model: opt_str(&map, "model"),
                    status_message: opt_str(&map, "statusMessage"),
                    once: opt_bool(&map, "once"),
                })
            }
            "http" => {
                let url = opt_str(&map, "url")
                    .ok_or_else(|| de::Error::missing_field("url"))?;
                let headers = map
                    .get("headers")
                    .and_then(|v| v.as_object())
                    .map(|m| {
                        m.iter()
                            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                            .collect()
                    });
                let allowed_env_vars = map
                    .get("allowedEnvVars")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    });
                Ok(HookCommand::Http {
                    url,
                    if_condition: opt_str(&map, "if"),
                    timeout: opt_u64(&map, "timeout"),
                    headers,
                    allowed_env_vars,
                    status_message: opt_str(&map, "statusMessage"),
                    once: opt_bool(&map, "once"),
                })
            }
            other => Err(de::Error::unknown_variant(
                other,
                &["command", "prompt", "agent", "http"],
            )),
        }
    }
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

/// A hook matcher as it appears in settings JSON.
///
/// Matches TypeScript `HookMatcher` from `schemas/hooks.ts`:
/// `{ matcher?: string, hooks: HookCommand[] }`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HookMatcher {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matcher: Option<String>,
    pub hooks: Vec<HookCommand>,
}

/// Hooks settings extracted from a settings.json `hooks` key.
///
/// Matches TypeScript `HooksSettings = Partial<Record<HookEvent, HookMatcher[]>>`.
pub type ParsedHooksSettings = HashMap<String, Vec<HookMatcher>>;

/// Parse hooks from a settings JSON value.
/// Returns the `hooks` section as a map from event name to matchers.
fn parse_hooks_from_settings(settings: &serde_json::Value) -> Option<ParsedHooksSettings> {
    let hooks_obj = settings.get("hooks")?;
    let hooks_map = hooks_obj.as_object()?;
    let mut parsed = HashMap::new();
    for (event_name, matchers_value) in hooks_map {
        if let Ok(matchers) =
            serde_json::from_value::<Vec<HookMatcher>>(matchers_value.clone())
        {
            if !matchers.is_empty() {
                parsed.insert(event_name.clone(), matchers);
            }
        }
    }
    if parsed.is_empty() {
        None
    } else {
        Some(parsed)
    }
}

/// Parse a hook event from a string (matches TypeScript enum names).
pub fn parse_hook_event(s: &str) -> Result<HookEvent, String> {
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

/// Check if only managed hooks should run.
/// Returns true when policy settings or env vars set allowManagedHooksOnly.
fn is_restricted_to_managed_only() -> bool {
    // Check environment variable (localized from CLAUDE_CODE_ to AI_)
    if std::env::var("AI_CODE_ALLOW_MANAGED_HOOKS_ONLY")
        .ok()
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
    {
        return true;
    }

    // Check policy settings for allowManagedHooksOnly
    // (would need policy settings integration)
    false
}

/// Get all hooks from all sources in priority order.
///
/// Priority (highest first):
/// 1. User settings (~/.ai/settings.json)
/// 2. Project settings (.ai/settings.json in CWD)
/// 3. Local settings (.ai/settings.local.json)
/// 4. Session hooks (transient, in-memory)
///
/// Matches TypeScript `getAllHooks(appState)` from hooksSettings.ts lines 92-161.
pub fn get_all_hooks(session_id: &str) -> Vec<IndividualHookConfig> {
    let mut hooks: Vec<IndividualHookConfig> = Vec::new();

    // Check if restricted to managed hooks only
    // (would check policy settings and env vars)
    let restricted_to_managed_only = is_restricted_to_managed_only();

    if !restricted_to_managed_only {
        // Get hooks from all editable sources in priority order
        let sources = [
            EditableSettingSource::UserSettings,
            EditableSettingSource::ProjectSettings,
            EditableSettingSource::LocalSettings,
        ];

        // Track which setting files we've already processed to avoid duplicates
        // (e.g., when running from home directory, userSettings and projectSettings
        // both resolve to ~/.ai/settings.json)
        let mut seen_files: HashSet<String> = HashSet::new();

        for source in &sources {
            let file_path = get_settings_file_path_for_source(source);

            if let Some(ref path) = file_path {
                let resolved_path = path
                    .canonicalize()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| path.to_string_lossy().to_string());

                if seen_files.contains(&resolved_path) {
                    continue;
                }
                seen_files.insert(resolved_path);
            }

            // Get hooks from this source's settings file
            if let Some(source_hooks) = get_hooks_for_source(source) {
                for (event_str, matchers) in source_hooks {
                    if let Ok(event) = parse_hook_event(&event_str) {
                        for matcher in &matchers {
                            for hook_command in &matcher.hooks {
                                hooks.push(IndividualHookConfig {
                                    event: event.clone(),
                                    config: hook_command.clone(),
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

    // Get session hooks and add them to the list
    let session_hooks_map = get_session_hooks(session_id, None);
    for (event, matchers) in session_hooks_map {
        for matcher in matchers {
            for hook_command in &matcher.hooks {
                hooks.push(IndividualHookConfig {
                    event: event.clone(),
                    config: hook_command.clone(),
                    matcher: Some(matcher.matcher.clone()),
                    source: HookSource::SessionHook,
                    plugin_name: None,
                });
            }
        }
    }

    hooks
}

/// Get hooks for a specific event
pub fn get_hooks_for_event(session_id: &str, event: &HookEvent) -> Vec<IndividualHookConfig> {
    get_all_hooks(session_id)
        .into_iter()
        .filter(|hook| &hook.event == event)
        .collect()
}

/// Read hooks from a specific editable settings source.
///
/// Reads the settings file for the source, extracts the `hooks` section,
/// and parses it into a map of event name -> matchers.
pub fn get_hooks_for_source(source: &EditableSettingSource) -> Option<ParsedHooksSettings> {
    let path = get_settings_file_path_for_source(source)?;
    let settings = read_settings_file(&path)?;
    parse_hooks_from_settings(&settings)
}

/// Hook source description display string
pub fn hook_source_description_display_string(source: &HookSource) -> String {
    match source {
        HookSource::Editable(s) => match s {
            EditableSettingSource::UserSettings => {
                "User settings (~/.ai/settings.json)".to_string()
            }
            EditableSettingSource::ProjectSettings => {
                "Project settings (.ai/settings.json)".to_string()
            }
            EditableSettingSource::LocalSettings => {
                "Local settings (.ai/settings.local.json)".to_string()
            }
        },
        HookSource::PolicySettings => "Policy settings".to_string(),
        HookSource::PluginHook => "Plugin hooks (~/.ai/plugins/*/hooks/hooks.json)".to_string(),
        HookSource::SessionHook => "Session hooks (in-memory, temporary)".to_string(),
        HookSource::BuiltinHook => {
            "Built-in hook (registered internally by AI Code)".to_string()
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
            status_message: None,
            once: None,
            r#async: None,
            async_rewake: None,
        };
        let b = HookCommand::Command {
            command: "echo hello".to_string(),
            shell: None,
            if_condition: None,
            timeout: Some(30), // Different timeout doesn't matter
            status_message: None,
            once: None,
            r#async: None,
            async_rewake: None,
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
            status_message: None,
            once: None,
            r#async: None,
            async_rewake: None,
        };
        let b = HookCommand::Command {
            command: "echo world".to_string(),
            shell: None,
            if_condition: None,
            timeout: None,
            status_message: None,
            once: None,
            r#async: None,
            async_rewake: None,
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
            status_message: None,
            once: None,
            r#async: None,
            async_rewake: None,
        };
        let b = HookCommand::Prompt {
            prompt: "echo hello".to_string(),
            if_condition: None,
            timeout: None,
            model: None,
            status_message: None,
            once: None,
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
            status_message: None,
            once: None,
            r#async: None,
            async_rewake: None,
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
            "User settings (~/.ai/settings.json)"
        );
        assert_eq!(hook_source_header_display_string(&source), "User Settings");
        assert_eq!(hook_source_inline_display_string(&source), "User");
    }

    #[test]
    fn test_deserialize_hook_command_command() {
        let json = serde_json::json!({
            "type": "command",
            "command": "echo hello",
            "shell": "bash",
            "if": "Bash(git *)"
        });
        let hook: HookCommand = serde_json::from_value(json).unwrap();
        match hook {
            HookCommand::Command {
                command, shell, if_condition, ..
            } => {
                assert_eq!(command, "echo hello");
                assert_eq!(shell, Some("bash".to_string()));
                assert_eq!(if_condition, Some("Bash(git *)".to_string()));
            }
            _ => panic!("Expected Command variant"),
        }
    }

    #[test]
    fn test_deserialize_hook_command_prompt() {
        let json = serde_json::json!({
            "type": "prompt",
            "prompt": "Review the code",
            "model": "claude-sonnet-4-6"
        });
        let hook: HookCommand = serde_json::from_value(json).unwrap();
        match hook {
            HookCommand::Prompt { prompt, model, .. } => {
                assert_eq!(prompt, "Review the code");
                assert_eq!(model, Some("claude-sonnet-4-6".to_string()));
            }
            _ => panic!("Expected Prompt variant"),
        }
    }

    #[test]
    fn test_deserialize_hook_command_agent() {
        let json = serde_json::json!({
            "type": "agent",
            "prompt": "Verify tests pass"
        });
        let hook: HookCommand = serde_json::from_value(json).unwrap();
        match hook {
            HookCommand::Agent { prompt, .. } => {
                assert_eq!(prompt, "Verify tests pass");
            }
            _ => panic!("Expected Agent variant"),
        }
    }

    #[test]
    fn test_deserialize_hook_command_http() {
        let json = serde_json::json!({
            "type": "http",
            "url": "https://example.com/hook",
            "if": "Bash(npm *)"
        });
        let hook: HookCommand = serde_json::from_value(json).unwrap();
        match hook {
            HookCommand::Http { url, if_condition, .. } => {
                assert_eq!(url, "https://example.com/hook");
                assert_eq!(if_condition, Some("Bash(npm *)".to_string()));
            }
            _ => panic!("Expected Http variant"),
        }
    }

    #[test]
    fn test_deserialize_hook_matcher() {
        let json = serde_json::json!({
            "matcher": "Bash(git *)",
            "hooks": [
                {"type": "command", "command": "git status"},
                {"type": "prompt", "prompt": "Check git state"}
            ]
        });
        let matcher: HookMatcher = serde_json::from_value(json).unwrap();
        assert_eq!(matcher.matcher, Some("Bash(git *)".to_string()));
        assert_eq!(matcher.hooks.len(), 2);
    }

    #[test]
    fn test_deserialize_hook_matcher_no_matcher() {
        let json = serde_json::json!({
            "hooks": [
                {"type": "command", "command": "echo hi"}
            ]
        });
        let matcher: HookMatcher = serde_json::from_value(json).unwrap();
        assert_eq!(matcher.matcher, None);
        assert_eq!(matcher.hooks.len(), 1);
    }

    #[test]
    fn test_parse_hooks_from_settings() {
        let settings = serde_json::json!({
            "hooks": {
                "Stop": [
                    {
                        "hooks": [
                            {"type": "command", "command": "echo stopped"}
                        ]
                    }
                ],
                "PreToolUse": [
                    {
                        "matcher": "Bash(git *)",
                        "hooks": [
                            {"type": "command", "command": "git status"}
                        ]
                    }
                ]
            },
            "model": "claude-sonnet-4-6"
        });
        let parsed = parse_hooks_from_settings(&settings).unwrap();
        assert_eq!(parsed.len(), 2);
        assert!(parsed.contains_key("Stop"));
        assert!(parsed.contains_key("PreToolUse"));
        assert_eq!(parsed["Stop"].len(), 1);
        assert_eq!(parsed["PreToolUse"][0].matcher, Some("Bash(git *)".to_string()));
    }

    #[test]
    fn test_parse_hooks_from_settings_no_hooks() {
        let settings = serde_json::json!({
            "model": "claude-sonnet-4-6"
        });
        let parsed = parse_hooks_from_settings(&settings);
        assert!(parsed.is_none());
    }

    #[test]
    fn test_hook_command_serialization() {
        let hook = HookCommand::Command {
            command: "echo hello".to_string(),
            shell: Some("bash".to_string()),
            if_condition: Some("Bash(git *)".to_string()),
            timeout: Some(30),
            status_message: Some("Running git check".to_string()),
            once: Some(false),
            r#async: Some(true),
            async_rewake: Some(false),
        };
        let json = serde_json::to_value(&hook).unwrap();
        assert_eq!(json["type"], "command");
        assert_eq!(json["command"], "echo hello");
        assert_eq!(json["shell"], "bash");
        assert_eq!(json["if"], "Bash(git *)");
        assert_eq!(json["timeout"], 30);
        assert_eq!(json["statusMessage"], "Running git check");
        assert_eq!(json["async"], true);
        assert_eq!(json["asyncRewake"], false);
    }
}
