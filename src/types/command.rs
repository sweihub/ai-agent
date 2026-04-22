// Source: ~/claudecode/openclaudecode/src/types/command.ts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::types::ids::AgentId;
use crate::types::logs::LogOption;
use crate::types::message::Message;

/// Local command result type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LocalCommandResult {
    #[serde(rename = "text")]
    Text { value: String },
    #[serde(rename = "compact")]
    Compact {
        #[serde(rename = "compactionResult")]
        compaction_result: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "displayText")]
        display_text: Option<String>,
    },
    #[serde(rename = "skip")]
    Skip,
}

/// Setting source enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SettingSource {
    UserSettings,
    ProjectSettings,
    LocalSettings,
    PolicySettings,
}

/// Prompt command configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCommand {
    #[serde(rename = "type")]
    pub command_type: String, // "prompt"
    #[serde(rename = "progressMessage")]
    pub progress_message: String,
    #[serde(rename = "contentLength")]
    pub content_length: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "argNames")]
    pub arg_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "allowedTools")]
    pub allowed_tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub source: CommandSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "pluginInfo")]
    pub plugin_info: Option<PluginInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "disableNonInteractive")]
    pub disable_non_interactive: Option<bool>,
    /// Hooks to register when this skill is invoked
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<serde_json::Value>,
    /// Base directory for skill resources
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "skillRoot")]
    pub skill_root: Option<String>,
    /// Execution context: 'inline' (default) or 'fork' (run as sub-agent)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<ExecutionContext>,
    /// Agent type to use when forked
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Effort level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    /// Glob patterns for file paths this skill applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<String>>,
}

/// Where a command was loaded from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandSource {
    UserSettings,
    ProjectSettings,
    LocalSettings,
    PolicySettings,
    Builtin,
    Mcp,
    Plugin,
}

/// Plugin info for a command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    #[serde(rename = "pluginManifest")]
    pub plugin_manifest: serde_json::Value,
    pub repository: String,
}

/// Execution context for commands.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionContext {
    Inline,
    Fork,
}

/// Local command module shape.
pub type LocalCommandModule = serde_json::Value;

/// Local command definition.
pub struct LocalCommand {
    pub command_type: String, // "local"
    pub supports_non_interactive: bool,
    pub load: Box<
        dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = LocalCommandModule> + Send>>
            + Send
            + Sync,
    >,
}

/// Resume entry point.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResumeEntrypoint {
    CliFlag,
    SlashCommandPicker,
    SlashCommandSessionId,
    SlashCommandTitle,
    Fork,
}

/// Command result display enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CommandResultDisplay {
    Skip,
    System,
    User,
}

/// Options for command completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandCompleteOptions {
    /// How to display the result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<CommandResultDisplay>,
    /// If true, send messages to the model after command completes
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "shouldQuery")]
    pub should_query: Option<bool>,
    /// Additional messages to insert as isMeta
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "metaMessages")]
    pub meta_messages: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "nextInput")]
    pub next_input: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "submitNextInput")]
    pub submit_next_input: Option<bool>,
}

/// Callback when a command completes.
pub type LocalJsxCommandOnDone =
    Box<dyn Fn(Option<String>, Option<CommandCompleteOptions>) + Send + Sync>;

/// Local JSX command module shape.
pub type LocalJsxCommandModule = serde_json::Value;

/// Local JSX command definition.
pub struct LocalJsxCommand {
    pub command_type: String, // "local-jsx"
    pub load: Box<
        dyn Fn()
                -> std::pin::Pin<Box<dyn std::future::Future<Output = LocalJsxCommandModule> + Send>>
            + Send
            + Sync,
    >,
}

/// Callback when a command declares who can use it (auth/provider requirement, static).
/// This is separate from `isEnabled()`:
///   - `availability` = who can use this (auth/provider requirement, static)
///   - `isEnabled()`  = is this turned on right now (GrowthBook, platform, env vars)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CommandAvailability {
    /// claude.ai OAuth subscriber (Pro/Max/Team/Enterprise via claude.ai)
    ClaudeAi,
    /// Console API key user (direct api.anthropic.com, not via claude.ai OAuth)
    Console,
}

/// Base command shared by all command types.
pub struct CommandBase {
    pub availability: Option<Vec<CommandAvailability>>,
    pub description: String,
    pub has_user_specified_description: Option<bool>,
    /// Defaults to true. Only set when the command has conditional enablement.
    pub is_enabled: Option<Box<dyn Fn() -> bool + Send + Sync>>,
    /// Defaults to false. Only set when the command should be hidden from typeahead/help.
    pub is_hidden: Option<bool>,
    pub name: String,
    pub aliases: Option<Vec<String>>,
    pub is_mcp: Option<bool>,
    pub argument_hint: Option<String>,
    /// Detailed usage scenarios for when to use this command
    pub when_to_use: Option<String>,
    /// Version of the command/skill
    pub version: Option<String>,
    /// Whether to disable this command from being invoked by models
    pub disable_model_invocation: Option<bool>,
    /// Whether users can invoke this skill by typing /skill-name
    pub user_invocable: Option<bool>,
    /// Where the command was loaded from
    pub loaded_from: Option<CommandLoadSource>,
    /// Distinguishes workflow-backed commands (badged in autocomplete)
    pub kind: Option<CommandKind>,
    /// If true, command executes immediately without waiting for a stop point
    pub immediate: Option<bool>,
    /// If true, args are redacted from the conversation history
    pub is_sensitive: Option<bool>,
}

/// Where the command was loaded from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandLoadSource {
    CommandsDeprecated,
    Skills,
    Plugin,
    Managed,
    Bundled,
    Mcp,
}

/// Command kind enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CommandKind {
    Workflow,
}

/// Resolves the user-visible name, falling back to `cmd.name` when not overridden.
pub fn get_command_name(cmd: &CommandBase) -> &str {
    &cmd.name
}

/// Resolves whether the command is enabled, defaulting to true.
pub fn is_command_enabled(is_enabled: Option<&Box<dyn Fn() -> bool + Send + Sync>>) -> bool {
    match is_enabled {
        Some(f) => f(),
        None => true,
    }
}

/// Unified command enum combining base with specific implementation types.
pub enum Command {
    Prompt {
        base: CommandBase,
        prompt: PromptCommand,
    },
    Local {
        base: CommandBase,
        local: LocalCommand,
    },
    LocalJsx {
        base: CommandBase,
        local_jsx: LocalJsxCommand,
    },
}
