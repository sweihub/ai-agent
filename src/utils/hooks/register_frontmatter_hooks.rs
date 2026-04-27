// Source: ~/claudecode/openclaudecode/src/utils/hooks/registerFrontmatterHooks.ts
#![allow(dead_code)]

use std::collections::HashMap;

use crate::utils::hooks::hooks_settings::{HookCommand, HookEvent};
use crate::utils::hooks::session_hooks::add_session_hook;

/// Hooks settings structure (simplified)
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HooksSettings {
    #[serde(flatten)]
    pub events: HashMap<String, Vec<HookMatcher>>,
}

/// A hook matcher groups hooks by matching criteria
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HookMatcher {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matcher: Option<String>,
    pub hooks: Vec<serde_json::Value>,
}

/// All hook events as strings (for iteration)
const HOOK_EVENT_NAMES: &[&str] = &[
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "PermissionDenied",
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

/// Parse a hook event from a string
fn parse_hook_event(s: &str) -> Option<HookEvent> {
    match s {
        "PreToolUse" => Some(HookEvent::PreToolUse),
        "PostToolUse" => Some(HookEvent::PostToolUse),
        "PostToolUseFailure" => Some(HookEvent::PostToolUseFailure),
        "PermissionDenied" => Some(HookEvent::PermissionDenied),
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

/// Register hooks from frontmatter (agent or skill) into session-scoped hooks.
/// These hooks will be active for the duration of the session/agent and cleaned up
/// when the session/agent ends.
///
/// # Arguments
/// * `set_app_state` - Function to update app state
/// * `session_id` - Session ID to scope the hooks (agent ID for agents, session ID for skills)
/// * `hooks` - The hooks settings from frontmatter
/// * `source_name` - Human-readable source name for logging (e.g., "agent 'my-agent'")
/// * `is_agent` - If true, converts Stop hooks to SubagentStop (since subagents trigger SubagentStop, not Stop)
pub fn register_frontmatter_hooks(
    set_app_state: &dyn Fn(&dyn Fn(&mut serde_json::Value)),
    session_id: &str,
    hooks: &HooksSettings,
    source_name: &str,
    is_agent: bool,
) {
    if hooks.events.is_empty() {
        return;
    }

    let mut hook_count = 0;

    for event_name in HOOK_EVENT_NAMES {
        let matchers = match hooks.events.get(*event_name) {
            Some(m) => m,
            None => continue,
        };
        if matchers.is_empty() {
            continue;
        }

        // Parse the event
        let event = match parse_hook_event(event_name) {
            Some(e) => e,
            None => continue,
        };

        // For agents, convert Stop hooks to SubagentStop since that's what fires when an agent completes
        // (executeStopHooks uses SubagentStop when called with an agentId)
        let target_event = if is_agent && event == HookEvent::Stop {
            log_for_debugging(&format!(
                "Converting Stop hook to SubagentStop for {} (subagents trigger SubagentStop)",
                source_name
            ));
            HookEvent::SubagentStop
        } else {
            event
        };

        for matcher_config in matchers {
            let matcher = matcher_config.matcher.clone().unwrap_or_default();

            // Parse hooks from JSON values
            for hook_json in &matcher_config.hooks {
                if let Ok(hook_command) = parse_hook_command(hook_json) {
                    add_session_hook_inner(
                        set_app_state,
                        session_id,
                        &target_event,
                        &matcher,
                        hook_command,
                    );
                    hook_count += 1;
                }
            }
        }
    }

    if hook_count > 0 {
        log_for_debugging(&format!(
            "Registered {} frontmatter hook(s) from {} for session {}",
            hook_count, source_name, session_id
        ));
    }
}

/// Parse a hook command from a JSON value
fn parse_hook_command(value: &serde_json::Value) -> Result<HookCommand, String> {
    // Try to parse as different hook types
    if let Some(command) = value.get("command").and_then(|v| v.as_str()) {
        return Ok(HookCommand::Command {
            command: command.to_string(),
            shell: value
                .get("shell")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            if_condition: value
                .get("if")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            timeout: value.get("timeout").and_then(|v| v.as_u64()),
            status_message: value
                .get("statusMessage")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            once: value.get("once").and_then(|v| v.as_bool()),
            r#async: value.get("async").and_then(|v| v.as_bool()),
            async_rewake: value
                .get("asyncRewake")
                .and_then(|v| v.as_bool()),
        });
    }

    if let Some(prompt) = value.get("prompt").and_then(|v| v.as_str()) {
        // Check if it's an agent hook (has model field)
        if value.get("model").is_some() {
            return Ok(HookCommand::Agent {
                prompt: prompt.to_string(),
                model: value
                    .get("model")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                if_condition: value
                    .get("if")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                timeout: value.get("timeout").and_then(|v| v.as_u64()),
                status_message: value
                    .get("statusMessage")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                once: value.get("once").and_then(|v| v.as_bool()),
            });
        }

        return Ok(HookCommand::Prompt {
            prompt: prompt.to_string(),
            if_condition: value
                .get("if")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            timeout: value.get("timeout").and_then(|v| v.as_u64()),
            model: value
                .get("model")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            status_message: value
                .get("statusMessage")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            once: value.get("once").and_then(|v| v.as_bool()),
        });
    }

    if let Some(url) = value.get("url").and_then(|v| v.as_str()) {
        return Ok(HookCommand::Http {
            url: url.to_string(),
            if_condition: value
                .get("if")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            timeout: value.get("timeout").and_then(|v| v.as_u64()),
            headers: value
                .get("headers")
                .and_then(|v| v.as_object())
                .map(|m| {
                    m.iter()
                        .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                        .collect()
                }),
            allowed_env_vars: value
                .get("allowedEnvVars")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                }),
            status_message: value
                .get("statusMessage")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            once: value.get("once").and_then(|v| v.as_bool()),
        });
    }

    Err("Could not parse hook command from JSON".to_string())
}

/// Add a session hook (internal helper)
fn add_session_hook_inner(
    set_app_state: &dyn Fn(&dyn Fn(&mut serde_json::Value)),
    session_id: &str,
    event: &HookEvent,
    matcher: &str,
    hook: HookCommand,
) {
    add_session_hook(
        set_app_state,
        session_id,
        event,
        matcher,
        hook,
        None, // on_hook_success
        None, // skill_root
    );
}

/// Log for debugging
fn log_for_debugging(msg: &str) {
    log::debug!("{}", msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hook_event() {
        assert_eq!(parse_hook_event("Stop"), Some(HookEvent::Stop));
        assert_eq!(parse_hook_event("PreToolUse"), Some(HookEvent::PreToolUse));
        assert_eq!(parse_hook_event("Unknown"), None);
    }

    #[test]
    fn test_parse_hook_command_command_type() {
        let json = serde_json::json!({
            "command": "echo hello",
            "shell": "bash",
            "if": "Bash(*)",
            "timeout": 30
        });
        let result = parse_hook_command(&json);
        assert!(result.is_ok());
        if let HookCommand::Command {
            command,
            shell,
            if_condition,
            timeout,
            ..
        } = result.unwrap()
        {
            assert_eq!(command, "echo hello");
            assert_eq!(shell, Some("bash".to_string()));
            assert_eq!(if_condition, Some("Bash(*)".to_string()));
            assert_eq!(timeout, Some(30));
        } else {
            panic!("Expected Command variant");
        }
    }

    #[test]
    fn test_parse_hook_command_prompt_type() {
        let json = serde_json::json!({
            "prompt": "Check if X is done"
        });
        let result = parse_hook_command(&json);
        assert!(result.is_ok());
        if let HookCommand::Prompt { prompt, .. } = result.unwrap() {
            assert_eq!(prompt, "Check if X is done");
        } else {
            panic!("Expected Prompt variant");
        }
    }

    #[test]
    fn test_parse_hook_command_agent_type() {
        let json = serde_json::json!({
            "prompt": "Verify the condition",
            "model": "claude-3-haiku-20240307"
        });
        let result = parse_hook_command(&json);
        assert!(result.is_ok());
        if let HookCommand::Agent { prompt, model, .. } = result.unwrap() {
            assert_eq!(prompt, "Verify the condition");
            assert_eq!(model, Some("claude-3-haiku-20240307".to_string()));
        } else {
            panic!("Expected Agent variant");
        }
    }

    #[test]
    fn test_parse_hook_command_http_type() {
        let json = serde_json::json!({
            "url": "https://example.com/hook"
        });
        let result = parse_hook_command(&json);
        assert!(result.is_ok());
        if let HookCommand::Http { url, .. } = result.unwrap() {
            assert_eq!(url, "https://example.com/hook");
        } else {
            panic!("Expected Http variant");
        }
    }

    #[test]
    fn test_register_frontmatter_hooks_empty() {
        let hooks = HooksSettings::default();
        let call_count = std::cell::Cell::new(0usize);
        let set_app_state = |_: &dyn Fn(&mut serde_json::Value)| {
            call_count.set(call_count.get() + 1);
        };

        register_frontmatter_hooks(&set_app_state, "test-session", &hooks, "test-agent", false);

        assert_eq!(call_count.get(), 0);
    }
}
