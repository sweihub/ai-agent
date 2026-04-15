// Source: /data/home/swei/claudecode/openclaudecode/src/commands.ts
//! Plugin commands - ported from ~/claudecode/openclaudecode/src/utils/plugins/loadPluginCommands.ts
//!
//! This module provides the plugin command registry and execution functionality.

use crate::error::AgentError;
use crate::plugin::types::LoadedPlugin;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, OnceLock, RwLock};

/// Frontmatter data parsed from command markdown files
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CommandFrontmatter {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    #[serde(rename = "allowed-tools")]
    pub allowed_tools: Option<serde_json::Value>,
    #[serde(default)]
    #[serde(rename = "argument-hint")]
    pub argument_hint: Option<String>,
    #[serde(default)]
    #[serde(rename = "arguments")]
    pub arguments: Option<serde_json::Value>,
    #[serde(default)]
    #[serde(rename = "when_to_use")]
    pub when_to_use: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub effort: Option<String>,
    #[serde(default)]
    #[serde(rename = "disable-model-invocation")]
    pub disable_model_invocation: Option<bool>,
    #[serde(default)]
    #[serde(rename = "user-invocable")]
    pub user_invocable: Option<bool>,
    #[serde(default)]
    pub shell: Option<serde_json::Value>,
}

/// A plugin command definition
#[derive(Debug, Clone)]
pub struct PluginCommand {
    /// Unique command name (e.g., "my-plugin:my-command")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Tools allowed when executing this command
    pub allowed_tools: Vec<String>,
    /// Hint for command arguments
    pub argument_hint: Option<String>,
    /// Argument names parsed from frontmatter
    pub arg_names: Vec<String>,
    /// When to use this command
    pub when_to_use: Option<String>,
    /// Command version
    pub version: Option<String>,
    /// Model to use for this command
    pub model: Option<String>,
    /// Effort level
    pub effort: Option<u8>,
    /// Whether to disable model invocation
    pub disable_model_invocation: bool,
    /// Whether the command can be invoked by user
    pub user_invocable: bool,
    /// The command content/prompt
    pub content: String,
    /// Path to the command file
    pub source_path: Option<String>,
    /// Plugin info
    pub plugin_name: String,
    pub plugin_source: String,
    /// Whether this is a skill (loaded from skills directory)
    pub is_skill: bool,
    /// Content length for optimization
    pub content_length: usize,
}

/// Command execution context
#[derive(Debug, Clone, Default)]
pub struct CommandContext {
    /// Arguments to substitute into the command
    pub args: HashMap<String, String>,
    /// Additional context variables
    pub variables: HashMap<String, String>,
}

/// Result of command execution
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult {
    pub success: bool,
    pub content: String,
    pub error: Option<String>,
}

/// Parse simple YAML-like frontmatter from markdown content
pub fn parse_frontmatter(content: &str) -> (CommandFrontmatter, String) {
    let mut frontmatter = CommandFrontmatter::default();
    let trimmed = content.trim();

    if !trimmed.starts_with("---") {
        return (frontmatter, content.to_string());
    }

    // Find the closing ---
    if let Some(end_pos) = trimmed[3..].find("---") {
        let frontmatter_str = &trimmed[3..end_pos + 3];

        // Parse key: value pairs
        for line in frontmatter_str.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 1..].trim().to_string();

                match key.as_str() {
                    "description" => frontmatter.description = Some(value),
                    "name" => frontmatter.name = Some(value),
                    "allowed-tools" => {
                        if value.is_empty() {
                            frontmatter.allowed_tools = Some(serde_json::json!([]));
                        } else {
                            let tools: Vec<String> = value
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                            frontmatter.allowed_tools = Some(serde_json::json!(tools));
                        }
                    }
                    "argument-hint" => frontmatter.argument_hint = Some(value),
                    "arguments" => frontmatter.arguments = Some(serde_json::json!(value)),
                    "when_to_use" => frontmatter.when_to_use = Some(value),
                    "version" => frontmatter.version = Some(value),
                    "model" => frontmatter.model = Some(value),
                    "effort" => frontmatter.effort = Some(value),
                    "disable-model-invocation" => {
                        frontmatter.disable_model_invocation =
                            Some(value.parse::<bool>().ok().unwrap_or(false));
                    }
                    "user-invocable" => {
                        frontmatter.user_invocable =
                            Some(value.parse::<bool>().ok().unwrap_or(true));
                    }
                    _ => {}
                }
            }
        }

        let body = trimmed[end_pos + 6..].trim_start().to_string();
        return (frontmatter, body);
    }

    (frontmatter, content.to_string())
}

/// Parse argument names from arguments field
pub fn parse_argument_names(arguments: &Option<serde_json::Value>) -> Vec<String> {
    match arguments {
        Some(serde_json::Value::String(s)) => s
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

/// Parse allowed tools from frontmatter
pub fn parse_allowed_tools(allowed_tools: &Option<serde_json::Value>) -> Vec<String> {
    match allowed_tools {
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
            .collect(),
        Some(serde_json::Value::String(s)) => s
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

/// Parse effort value from string
pub fn parse_effort_value(effort: &Option<String>) -> Option<u8> {
    match effort {
        Some(s) => {
            // Try parsing as number first
            if let Ok(num) = s.parse::<u8>() {
                return Some(num);
            }
            // Try keyword mappings
            match s.to_lowercase().as_str() {
                "minimal" => Some(1),
                "low" => Some(2),
                "medium" => Some(3),
                "high" => Some(5),
                "maximum" => Some(8),
                _ => None,
            }
        }
        None => None,
    }
}

/// Load a single plugin command from a markdown file
pub fn load_command_from_file(
    file_path: &Path,
    plugin_name: &str,
    plugin_source: &str,
    is_skill: bool,
) -> Result<PluginCommand, AgentError> {
    let content = fs::read_to_string(file_path).map_err(|e| AgentError::Io(e))?;

    let (frontmatter, body) = parse_frontmatter(&content);

    // Extract description
    let description = frontmatter
        .description
        .clone()
        .unwrap_or_else(|| extract_description_from_markdown(&body));

    // Parse allowed tools
    let allowed_tools = parse_allowed_tools(&frontmatter.allowed_tools);

    // Parse argument names
    let arg_names = parse_argument_names(&frontmatter.arguments);

    // Parse effort
    let effort = parse_effort_value(&frontmatter.effort);

    // Determine user invocable (default true)
    let user_invocable = frontmatter.user_invocable.unwrap_or(true);

    // Get command name from file path
    let command_name = if is_skill {
        // For skills, use the parent directory name
        file_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|n| format!("{}:{}", plugin_name, n))
            .unwrap_or_else(|| format!("{}:unknown", plugin_name))
    } else {
        // For regular commands, use filename without extension
        file_path
            .file_stem()
            .and_then(|n| n.to_str())
            .map(|n| format!("{}:{}", plugin_name, n))
            .unwrap_or_else(|| format!("{}:unknown", plugin_name))
    };

    Ok(PluginCommand {
        name: command_name,
        description,
        allowed_tools,
        argument_hint: frontmatter.argument_hint.clone(),
        arg_names,
        when_to_use: frontmatter.when_to_use.clone(),
        version: frontmatter.version.clone(),
        model: frontmatter.model.clone(),
        effort,
        disable_model_invocation: frontmatter.disable_model_invocation.unwrap_or(false),
        user_invocable,
        content: body.clone(),
        source_path: Some(file_path.to_string_lossy().to_string()),
        plugin_name: plugin_name.to_string(),
        plugin_source: plugin_source.to_string(),
        is_skill,
        content_length: body.len(),
    })
}

/// Extract description from markdown content
fn extract_description_from_markdown(content: &str) -> String {
    // Try to get the first paragraph or heading
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Return first non-empty line, truncated
        return if trimmed.len() > 200 {
            format!("{}...", &trimmed[..200])
        } else {
            trimmed.to_string()
        };
    }
    "No description".to_string()
}

/// Load commands from a directory
pub fn load_commands_from_directory(
    dir_path: &Path,
    plugin_name: &str,
    plugin_source: &str,
    is_skill_mode: bool,
) -> Result<Vec<PluginCommand>, AgentError> {
    if !dir_path.exists() {
        return Ok(Vec::new());
    }

    let mut commands = Vec::new();

    let entries = fs::read_dir(dir_path).map_err(|e| AgentError::Io(e))?;

    for entry in entries {
        let entry = entry.map_err(|e| AgentError::Io(e))?;
        let path = entry.path();

        if path.is_dir() {
            // For skill mode, check if directory contains SKILL.md
            if is_skill_mode {
                let skill_file = path.join("SKILL.md");
                if skill_file.exists() {
                    if let Ok(cmd) =
                        load_command_from_file(&skill_file, plugin_name, plugin_source, true)
                    {
                        commands.push(cmd);
                    }
                }
            } else {
                // Recursively load from subdirectories
                match load_commands_from_directory(&path, plugin_name, plugin_source, false) {
                    Ok(sub_commands) => commands.extend(sub_commands),
                    Err(e) => {
                        log::warn!("Failed to load commands from {:?}: {}", path, e);
                    }
                }
            }
        } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
            // Skip SKILL.md files in non-skill mode (they're loaded as skills)
            if !is_skill_mode
                && path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map_or(false, |s| s.to_lowercase() == "skill.md")
            {
                continue;
            }

            if let Ok(cmd) = load_command_from_file(&path, plugin_name, plugin_source, false) {
                commands.push(cmd);
            }
        }
    }

    Ok(commands)
}

/// Substitute arguments in command content
pub fn substitute_arguments(content: &str, args: &HashMap<String, String>) -> String {
    let mut result = content.to_string();
    for (key, value) in args {
        let placeholder = format!("${{{}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

/// Command handler function type
pub type CommandHandler = Arc<
    dyn Fn(HashMap<String, String>, &CommandContext) -> Result<CommandResult, AgentError>
        + Send
        + Sync,
>;

/// Plugin command with handler
#[derive(Clone)]
pub struct ExecutablePluginCommand {
    pub command: PluginCommand,
    pub handler: Option<CommandHandler>,
}

impl ExecutablePluginCommand {
    /// Execute the command with given arguments
    pub fn execute(
        &self,
        args: HashMap<String, String>,
        context: &CommandContext,
    ) -> Result<CommandResult, AgentError> {
        // If there's a custom handler, use it
        if let Some(handler) = &self.handler {
            return handler(args, context);
        }

        // Otherwise, return the command content as a prompt
        let content = substitute_arguments(&self.command.content, &args);
        Ok(CommandResult {
            success: true,
            content,
            error: None,
        })
    }

    /// Get the prompt for this command
    pub fn get_prompt(&self, args: &HashMap<String, String>) -> String {
        substitute_arguments(&self.command.content, args)
    }
}

/// Global command registry
pub struct CommandRegistry {
    commands: RwLock<HashMap<String, ExecutablePluginCommand>>,
    /// Map of plugin names to their commands (for quick lookup by plugin)
    by_plugin: RwLock<HashMap<String, Vec<String>>>,
}

impl CommandRegistry {
    /// Create a new command registry
    pub fn new() -> Self {
        Self {
            commands: RwLock::new(HashMap::new()),
            by_plugin: RwLock::new(HashMap::new()),
        }
    }

    /// Get global command registry instance
    pub fn global() -> &'static CommandRegistry {
        static REGISTRY: OnceLock<CommandRegistry> = OnceLock::new();
        REGISTRY.get_or_init(|| CommandRegistry::new())
    }

    /// Register a command
    pub fn register(&self, command: PluginCommand) {
        let name = command.name.clone();
        let plugin_name = command.plugin_name.clone();

        let executable = ExecutablePluginCommand {
            command,
            handler: None,
        };

        // Add to commands map
        {
            let mut commands = self.commands.write().unwrap();
            commands.insert(name.clone(), executable);
        }

        // Add to by_plugin map
        {
            let mut by_plugin = self.by_plugin.write().unwrap();
            by_plugin
                .entry(plugin_name.clone())
                .or_insert_with(Vec::new)
                .push(name.clone());
        }

        log::debug!("Registered plugin command: {}", name);
    }

    /// Register a command with a custom handler
    pub fn register_with_handler(&self, command: PluginCommand, handler: CommandHandler) {
        let name = command.name.clone();
        let plugin_name = command.plugin_name.clone();

        let executable = ExecutablePluginCommand {
            command,
            handler: Some(handler),
        };

        {
            let mut commands = self.commands.write().unwrap();
            commands.insert(name.clone(), executable);
        }

        {
            let mut by_plugin = self.by_plugin.write().unwrap();
            by_plugin
                .entry(plugin_name)
                .or_insert_with(Vec::new)
                .push(name.clone());
        }

        log::debug!("Registered plugin command with handler: {}", name);
    }

    /// Get a command by name
    pub fn get(&self, name: &str) -> Option<ExecutablePluginCommand> {
        let commands = self.commands.read().unwrap();
        commands.get(name).cloned()
    }

    /// Get all registered command names
    pub fn all_commands(&self) -> Vec<String> {
        let commands = self.commands.read().unwrap();
        commands.keys().cloned().collect()
    }

    /// Get commands for a specific plugin
    pub fn get_by_plugin(&self, plugin_name: &str) -> Vec<ExecutablePluginCommand> {
        let commands = self.commands.read().unwrap();
        let by_plugin = self.by_plugin.read().unwrap();

        by_plugin
            .get(plugin_name)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|n| commands.get(n).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a command exists
    pub fn contains(&self, name: &str) -> bool {
        let commands = self.commands.read().unwrap();
        commands.contains_key(name)
    }

    /// Parse a slash command and extract plugin name, command name, and arguments
    /// Format: /plugin-name:command arg1=value1 arg2=value2
    pub fn parse_slash_command(input: &str) -> Option<(String, String, HashMap<String, String>)> {
        let input = input.trim();

        // Must start with /
        if !input.starts_with('/') {
            return None;
        }

        let input = &input[1..]; // Remove leading /

        // Find the : separator between plugin and command
        let colon_pos = input.find(':')?;

        let plugin_name = input[..colon_pos].to_string();
        let rest = &input[colon_pos + 1..];

        // Parse command name and arguments
        let (command_name, args) = if let Some(space_pos) = rest.find(' ') {
            let cmd_name = rest[..space_pos].to_string();
            let args_str = &rest[space_pos + 1..];
            let args = Self::parse_arguments(args_str);
            (cmd_name, args)
        } else {
            (rest.to_string(), HashMap::new())
        };

        Some((plugin_name, command_name, args))
    }

    /// Parse command arguments from string
    /// Format: arg1=value1 arg2="value with spaces" arg3='quoted'
    fn parse_arguments(args_str: &str) -> HashMap<String, String> {
        let mut args = HashMap::new();
        let mut current_key = String::new();
        let mut current_value = String::new();
        let mut in_key = true;
        let mut in_quotes = false;
        let mut quote_char = '\0';

        for ch in args_str.chars() {
            if in_key {
                if ch == '=' {
                    in_key = false;
                } else if !ch.is_whitespace() {
                    current_key.push(ch);
                }
            } else {
                if in_quotes {
                    if ch == quote_char {
                        in_quotes = false;
                    } else {
                        current_value.push(ch);
                    }
                } else if ch == '"' || ch == '\'' {
                    in_quotes = true;
                    quote_char = ch;
                } else if ch.is_whitespace() && !current_key.is_empty() && !current_value.is_empty()
                {
                    // End of argument
                    args.insert(current_key.clone(), current_value.clone());
                    current_key.clear();
                    current_value.clear();
                    in_key = true;
                } else {
                    current_value.push(ch);
                }
            }
        }

        // Add last argument if present
        if !current_key.is_empty() {
            args.insert(current_key, current_value);
        }

        args
    }

    /// Execute a slash command
    pub fn execute_slash_command(
        &self,
        input: &str,
        context: &CommandContext,
    ) -> Result<CommandResult, AgentError> {
        let (plugin_name, command_name, args) =
            Self::parse_slash_command(input).ok_or_else(|| {
                AgentError::Command(format!("Invalid slash command format: {}", input))
            })?;

        let full_name = format!("{}:{}", plugin_name, command_name);

        let cmd = self
            .get(&full_name)
            .ok_or_else(|| AgentError::Command(format!("Command not found: {}", full_name)))?;

        cmd.execute(args, context)
    }

    /// Clear all registered commands
    pub fn clear(&self) {
        let mut commands = self.commands.write().unwrap();
        commands.clear();

        let mut by_plugin = self.by_plugin.write().unwrap();
        by_plugin.clear();
    }

    /// Get the number of registered commands
    pub fn len(&self) -> usize {
        let commands = self.commands.read().unwrap();
        commands.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Load commands from a plugin
pub fn load_plugin_commands(plugin: &LoadedPlugin) -> Result<Vec<PluginCommand>, AgentError> {
    let mut commands = Vec::new();
    let plugin_name = &plugin.name;
    let plugin_source = &plugin.source;

    // Load from commands_path
    if let Some(commands_path) = &plugin.commands_path {
        let path = Path::new(commands_path);
        match load_commands_from_directory(path, plugin_name, plugin_source, false) {
            Ok(cmds) => commands.extend(cmds),
            Err(e) => {
                log::warn!("Failed to load commands from {}: {}", commands_path, e);
            }
        }
    }

    // Load from additional commands_paths
    if let Some(commands_paths) = &plugin.commands_paths {
        for command_path in commands_paths {
            let path = Path::new(command_path);
            if path.is_dir() {
                match load_commands_from_directory(path, plugin_name, plugin_source, false) {
                    Ok(cmds) => commands.extend(cmds),
                    Err(e) => {
                        log::warn!("Failed to load commands from {}: {}", command_path, e);
                    }
                }
            } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                match load_command_from_file(path, plugin_name, plugin_source, false) {
                    Ok(cmd) => commands.push(cmd),
                    Err(e) => {
                        log::warn!("Failed to load command from {}: {}", command_path, e);
                    }
                }
            }
        }
    }

    // Load skills from skills_path
    if let Some(skills_path) = &plugin.skills_path {
        let path = Path::new(skills_path);
        match load_commands_from_directory(path, plugin_name, plugin_source, true) {
            Ok(cmds) => commands.extend(cmds),
            Err(e) => {
                log::warn!("Failed to load skills from {}: {}", skills_path, e);
            }
        }
    }

    // Load from additional skills_paths
    if let Some(skills_paths) = &plugin.skills_paths {
        for skill_path in skills_paths {
            let path = Path::new(skill_path);
            if path.is_dir() {
                match load_commands_from_directory(path, plugin_name, plugin_source, true) {
                    Ok(cmds) => commands.extend(cmds),
                    Err(e) => {
                        log::warn!("Failed to load skills from {}: {}", skill_path, e);
                    }
                }
            }
        }
    }

    Ok(commands)
}

/// Register all commands from a plugin
pub fn register_plugin_commands(plugin: &LoadedPlugin) -> Result<usize, AgentError> {
    let commands = load_plugin_commands(plugin)?;
    let registry = CommandRegistry::global();

    let count = commands.len();
    for command in commands {
        registry.register(command);
    }

    log::info!("Registered {} commands from plugin {}", count, plugin.name);

    Ok(count)
}

/// Get all registered plugin commands
pub fn get_all_plugin_commands() -> Vec<ExecutablePluginCommand> {
    let registry = CommandRegistry::global();
    let names = registry.all_commands();
    names.iter().filter_map(|n| registry.get(n)).collect()
}

/// Get a command by name (matches TypeScript getCommand)
pub fn get_command(name: &str) -> Option<ExecutablePluginCommand> {
    let registry = CommandRegistry::global();
    registry.get(name)
}

/// Check if a command exists (matches TypeScript hasCommand)
pub fn has_command(name: &str) -> bool {
    let registry = CommandRegistry::global();
    registry.contains(name)
}

/// Get all skill tool commands (matches TypeScript getSkillToolCommands)
/// Returns commands that are marked as skills
pub fn get_skill_tool_commands() -> Vec<ExecutablePluginCommand> {
    get_all_plugin_commands()
        .into_iter()
        .filter(|cmd| cmd.command.is_skill)
        .collect()
}

/// Register a command directly (convenience function)
pub fn register_command(command: PluginCommand) {
    let registry = CommandRegistry::global();
    registry.register(command);
}

/// Clear all registered commands (useful for testing)
pub fn clear_commands() {
    let registry = CommandRegistry::global();
    registry.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
description: Test command
allowed-tools: Bash,Read
argument-hint: <name>
---

This is the command content.
"#;
        let (fm, body) = parse_frontmatter(content);
        assert_eq!(fm.description, Some("Test command".to_string()));
        assert_eq!(fm.argument_hint, Some("<name>".to_string()));
        assert_eq!(body, "This is the command content.");
    }

    #[test]
    fn test_parse_argument_names() {
        let args = Some(serde_json::json!("arg1, arg2, arg3"));
        let names = parse_argument_names(&args);
        assert_eq!(names, vec!["arg1", "arg2", "arg3"]);
    }

    #[test]
    fn test_parse_allowed_tools() {
        let tools = Some(serde_json::json!(["Bash", "Read"]));
        let parsed = parse_allowed_tools(&tools);
        assert_eq!(parsed, vec!["Bash", "Read"]);
    }

    #[test]
    fn test_parse_effort_value() {
        assert_eq!(parse_effort_value(&Some("3".to_string())), Some(3));
        assert_eq!(parse_effort_value(&Some("medium".to_string())), Some(3));
        assert_eq!(parse_effort_value(&Some("high".to_string())), Some(5));
        assert_eq!(parse_effort_value(&None), None);
        assert_eq!(parse_effort_value(&Some("invalid".to_string())), None);
    }

    #[test]
    fn test_parse_slash_command() {
        let (plugin, cmd, args) =
            CommandRegistry::parse_slash_command("/my-plugin:hello arg1=value1").unwrap();
        assert_eq!(plugin, "my-plugin");
        assert_eq!(cmd, "hello");
        assert_eq!(args.get("arg1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_parse_slash_command_with_quoted_args() {
        let (plugin, cmd, args) =
            CommandRegistry::parse_slash_command("/my-plugin:hello name=\"John Doe\"").unwrap();
        assert_eq!(plugin, "my-plugin");
        assert_eq!(cmd, "hello");
        assert_eq!(args.get("name"), Some(&"John Doe".to_string()));
    }

    #[test]
    fn test_substitute_arguments() {
        let content = "Hello ${name}, your score is ${score}";
        let mut args = HashMap::new();
        args.insert("name".to_string(), "Alice".to_string());
        args.insert("score".to_string(), "100".to_string());

        let result = substitute_arguments(content, &args);
        assert_eq!(result, "Hello Alice, your score is 100");
    }

    #[test]
    fn test_command_registry_register_and_get() {
        let registry = CommandRegistry::new();
        registry.clear();

        let command = PluginCommand {
            name: "test:cmd".to_string(),
            description: "Test command".to_string(),
            allowed_tools: vec!["Bash".to_string()],
            argument_hint: None,
            arg_names: vec![],
            when_to_use: None,
            version: None,
            model: None,
            effort: None,
            disable_model_invocation: false,
            user_invocable: true,
            content: "Test content".to_string(),
            source_path: None,
            plugin_name: "test".to_string(),
            plugin_source: "test".to_string(),
            is_skill: false,
            content_length: 12,
        };

        registry.register(command);

        let retrieved = registry.get("test:cmd");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().command.name, "test:cmd");
    }

    #[test]
    fn test_command_registry_execute() {
        let registry = CommandRegistry::new();
        registry.clear();

        let command = PluginCommand {
            name: "test:hello".to_string(),
            description: "Test command".to_string(),
            allowed_tools: vec![],
            argument_hint: None,
            arg_names: vec!["name".to_string()],
            when_to_use: None,
            version: None,
            model: None,
            effort: None,
            disable_model_invocation: false,
            user_invocable: true,
            content: "Hello ${name}".to_string(),
            source_path: None,
            plugin_name: "test".to_string(),
            plugin_source: "test".to_string(),
            is_skill: false,
            content_length: 10,
        };

        registry.register(command);

        let result =
            registry.execute_slash_command("/test:hello name=World", &CommandContext::default());
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.content, "Hello World");
    }

    #[test]
    fn test_command_registry_by_plugin() {
        let registry = CommandRegistry::new();
        registry.clear();

        let cmd1 = PluginCommand {
            name: "my-plugin:cmd1".to_string(),
            description: "Command 1".to_string(),
            allowed_tools: vec![],
            argument_hint: None,
            arg_names: vec![],
            when_to_use: None,
            version: None,
            model: None,
            effort: None,
            disable_model_invocation: false,
            user_invocable: true,
            content: "Content 1".to_string(),
            source_path: None,
            plugin_name: "my-plugin".to_string(),
            plugin_source: "my-plugin".to_string(),
            is_skill: false,
            content_length: 9,
        };

        let cmd2 = PluginCommand {
            name: "my-plugin:cmd2".to_string(),
            description: "Command 2".to_string(),
            allowed_tools: vec![],
            argument_hint: None,
            arg_names: vec![],
            when_to_use: None,
            version: None,
            model: None,
            effort: None,
            disable_model_invocation: false,
            user_invocable: true,
            content: "Content 2".to_string(),
            source_path: None,
            plugin_name: "my-plugin".to_string(),
            plugin_source: "my-plugin".to_string(),
            is_skill: false,
            content_length: 9,
        };

        registry.register(cmd1);
        registry.register(cmd2);

        let by_plugin = registry.get_by_plugin("my-plugin");
        assert_eq!(by_plugin.len(), 2);
    }
}
