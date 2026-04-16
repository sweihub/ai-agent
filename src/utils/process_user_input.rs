//! Process user input utilities - translates processUserInput.ts from TypeScript
//!
//! This module handles processing user input, including text prompts, bash commands,
//! slash commands, and attachments.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::Message;

/// Prompt input mode
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PromptInputMode {
    #[default]
    Prompt,
    Bash,
    Print,
    Continue,
}

/// Process user input context - combines ToolUseContext and LocalJSXCommandContext
/// (Extended to match TypeScript's rich context with memory/skill tracking)
#[derive(Debug, Clone)]
pub struct ProcessUserInputContext {
    /// Session ID
    pub session_id: String,
    /// Current working directory
    pub cwd: String,
    /// Agent ID if set
    pub agent_id: Option<String>,
    /// Query tracking information
    pub query_tracking: Option<QueryTracking>,
    /// Context options
    pub options: ProcessUserInputContextOptions,
    /// Track nested memory paths loaded via memory attachment triggers
    pub loaded_nested_memory_paths: std::collections::HashSet<String>,
    /// Track discovered skill names (feeds was_discovered on skill_tool_invocation)
    pub discovered_skill_names: std::collections::HashSet<String>,
    /// Trigger directories for dynamic skill loading
    pub dynamic_skill_dir_triggers: std::collections::HashSet<String>,
    /// Trigger paths for nested memory attachments
    pub nested_memory_attachment_triggers: std::collections::HashSet<String>,
}

/// Query tracking for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryTracking {
    pub chain_id: String,
    pub depth: u32,
}

/// Process user input context options
#[derive(Debug, Clone)]
pub struct ProcessUserInputContextOptions {
    /// Available commands
    pub commands: Vec<Value>,
    /// Debug mode
    pub debug: bool,
    /// Available tools
    pub tools: Vec<crate::types::ToolDefinition>,
    /// Verbose mode
    pub verbose: bool,
    /// Main loop model
    pub main_loop_model: Option<String>,
    /// Thinking configuration
    pub thinking_config: Option<crate::types::api_types::ThinkingConfig>,
    /// MCP clients
    pub mcp_clients: Vec<Value>,
    /// MCP resources
    pub mcp_resources: std::collections::HashMap<String, Value>,
    /// IDE installation status
    pub ide_installation_status: Option<Value>,
    /// Non-interactive session flag
    pub is_non_interactive_session: bool,
    /// Custom system prompt
    pub custom_system_prompt: Option<String>,
    /// Append system prompt
    pub append_system_prompt: Option<String>,
    /// Agent definitions
    pub agent_definitions: AgentDefinitions,
    /// Theme
    pub theme: Option<String>,
    /// Max budget in USD
    pub max_budget_usd: Option<f64>,
}

impl Default for ProcessUserInputContext {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            cwd: String::new(),
            agent_id: None,
            query_tracking: None,
            options: ProcessUserInputContextOptions::default(),
            loaded_nested_memory_paths: std::collections::HashSet::new(),
            discovered_skill_names: std::collections::HashSet::new(),
            dynamic_skill_dir_triggers: std::collections::HashSet::new(),
            nested_memory_attachment_triggers: std::collections::HashSet::new(),
        }
    }
}

impl Default for ProcessUserInputContextOptions {
    fn default() -> Self {
        Self {
            commands: vec![],
            debug: false,
            tools: vec![],
            verbose: false,
            main_loop_model: None,
            thinking_config: None,
            mcp_clients: vec![],
            mcp_resources: std::collections::HashMap::new(),
            ide_installation_status: None,
            is_non_interactive_session: false,
            custom_system_prompt: None,
            append_system_prompt: None,
            agent_definitions: AgentDefinitions::default(),
            theme: None,
            max_budget_usd: None,
        }
    }
}

/// Agent definitions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDefinitions {
    pub active_agents: Vec<Value>,
    pub all_agents: Vec<Value>,
    pub allowed_agent_types: Option<Vec<String>>,
}

/// Effort value for the model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffortValue {
    pub effort: String,
    pub reason: Option<String>,
}

/// Result of processing user input
#[derive(Debug, Clone)]
pub struct ProcessUserInputBaseResult {
    /// Messages to be sent to the model
    pub messages: Vec<Message>,
    /// Whether a query should be made
    pub should_query: bool,
    /// Allowed tools (optional)
    pub allowed_tools: Option<Vec<String>>,
    /// Model to use (optional)
    pub model: Option<String>,
    /// Effort value (optional)
    pub effort: Option<EffortValue>,
    /// Output text for non-interactive mode
    pub result_text: Option<String>,
    /// Next input to prefilling (optional)
    pub next_input: Option<String>,
    /// Whether to submit next input
    pub submit_next_input: Option<bool>,
}

impl Default for ProcessUserInputBaseResult {
    fn default() -> Self {
        Self {
            messages: vec![],
            should_query: true,
            allowed_tools: None,
            model: None,
            effort: None,
            result_text: None,
            next_input: None,
            submit_next_input: None,
        }
    }
}

/// Input for process_user_input function
pub struct ProcessUserInputOptions {
    /// Input string or content blocks
    pub input: ProcessUserInput,
    /// Input before expansion (for ultraplan keyword detection)
    pub pre_expansion_input: Option<String>,
    /// Input mode
    pub mode: PromptInputMode,
    /// Context for processing
    pub context: ProcessUserInputContext,
    /// Pasted contents from the user
    pub pasted_contents: Option<std::collections::HashMap<u32, PastedContent>>,
    /// IDE selection
    pub ide_selection: Option<IdeSelection>,
    /// Existing messages
    pub messages: Option<Vec<Message>>,
    /// Function to set user input while processing
    pub set_user_input_on_processing: Option<Box<dyn Fn(Option<String>) + Send + Sync>>,
    /// UUID for the prompt
    pub uuid: Option<String>,
    /// Whether input is already being processed
    pub is_already_processing: Option<bool>,
    /// Query source
    pub query_source: Option<QuerySource>,
    /// Function to check if tool can be used
    pub can_use_tool: Option<crate::utils::hooks::CanUseToolFnJson>,
    /// Skip slash command processing
    pub skip_slash_commands: Option<bool>,
    /// Bridge origin (for remote control)
    pub bridge_origin: Option<bool>,
    /// Whether this is a meta message (system-generated)
    pub is_meta: Option<bool>,
    /// Skip attachment processing
    pub skip_attachments: Option<bool>,
}

impl Default for ProcessUserInputOptions {
    fn default() -> Self {
        Self {
            input: ProcessUserInput::String(String::new()),
            pre_expansion_input: None,
            mode: PromptInputMode::Prompt,
            context: ProcessUserInputContext::default(),
            pasted_contents: None,
            ide_selection: None,
            messages: None,
            set_user_input_on_processing: None,
            uuid: None,
            is_already_processing: None,
            query_source: None,
            can_use_tool: None,
            skip_slash_commands: None,
            bridge_origin: None,
            is_meta: None,
            skip_attachments: None,
        }
    }
}

/// User input - either string or content blocks
#[derive(Clone)]
pub enum ProcessUserInput {
    String(String),
    ContentBlocks(Vec<ContentBlockParam>),
}

impl std::fmt::Debug for ProcessUserInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessUserInput::String(s) => f.debug_tuple("String").field(s).finish(),
            ProcessUserInput::ContentBlocks(blocks) => {
                f.debug_tuple("ContentBlocks").field(blocks).finish()
            }
        }
    }
}

/// Content block parameter (similar to Anthropic SDK's ContentBlockParam)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContentBlockParam {
    /// Text content block
    Text {
        /// Text content
        text: String,
    },
    /// Image content block
    Image {
        /// Image source
        source: ImageSource,
    },
    /// Tool use content block
    ToolUse {
        /// Tool use ID
        id: String,
        /// Tool name
        name: String,
        /// Tool input
        input: Value,
    },
    /// Tool result content block
    ToolResult {
        /// Tool use ID
        tool_use_id: String,
        /// Tool result content
        content: Value,
        /// Whether this is an error
        #[serde(default, skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

/// Image source for content blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSource {
    /// Image type (base64)
    #[serde(rename = "type")]
    pub source_type: String,
    /// Media type (e.g., "image/png")
    pub media_type: String,
    /// Base64-encoded image data
    pub data: String,
}

/// Pasted content from user
#[derive(Debug, Clone)]
pub struct PastedContent {
    /// Unique ID
    pub id: u32,
    /// Content (base64-encoded)
    pub content: String,
    /// Media type
    pub media_type: Option<String>,
    /// Source path (optional)
    pub source_path: Option<String>,
    /// Dimensions (optional)
    pub dimensions: Option<ImageDimensions>,
}

/// Image dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

/// IDE selection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdeSelection {
    /// File path
    pub file_path: String,
    /// Selected text
    pub selected_text: Option<String>,
    /// Cursor position
    pub cursor_position: Option<CursorPosition>,
}

/// Cursor position in IDE
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorPosition {
    pub line: u32,
    pub character: u32,
}

/// Query source enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuerySource {
    Prompt,
    Continue,
    SlashCommand,
    BashCommand,
    Attachments,
    AutoAttach,
    Resubmit,
}

/// Process user input - main entry point
///
/// # Arguments
/// * `options` - Options for processing user input
///
/// # Returns
/// A future that resolves to ProcessUserInputBaseResult
pub async fn process_user_input(
    options: ProcessUserInputOptions,
) -> Result<ProcessUserInputBaseResult, String> {
    let input_string = match &options.input {
        ProcessUserInput::String(s) => Some(s.clone()),
        ProcessUserInput::ContentBlocks(blocks) => blocks.iter().find_map(|b| {
            if let ContentBlockParam::Text { text } = b {
                Some(text.clone())
            } else {
                None
            }
        }),
    };

    // Set user input on processing if in prompt mode
    if options.mode == PromptInputMode::Prompt
        && input_string.is_some()
        && options.is_meta != Some(true)
    {
        if let Some(ref callback) = options.set_user_input_on_processing {
            callback(input_string.clone());
        }
    }

    // Process the input - take ownership of needed fields
    let input = options.input;
    let mode = options.mode;
    let context = options.context;
    let pasted_contents = options.pasted_contents;
    let uuid = options.uuid;
    let is_meta = options.is_meta;
    let skip_slash_commands = options.skip_slash_commands;
    let bridge_origin = options.bridge_origin;

    let result = process_user_input_base(
        input,
        mode,
        context,
        pasted_contents,
        uuid,
        is_meta,
        skip_slash_commands,
        bridge_origin,
    )
    .await?;

    // Execute user prompt submit hooks (simplified stub)
    // In the full implementation, this would execute hooks and potentially modify result

    Ok(result)
}

/// Internal function to process user input
async fn process_user_input_base(
    input: ProcessUserInput,
    mode: PromptInputMode,
    _context: ProcessUserInputContext,
    pasted_contents: Option<std::collections::HashMap<u32, PastedContent>>,
    uuid: Option<String>,
    is_meta: Option<bool>,
    skip_slash_commands: Option<bool>,
    bridge_origin: Option<bool>,
) -> Result<ProcessUserInputBaseResult, String> {
    let input_string = match &input {
        ProcessUserInput::String(s) => Some(s.clone()),
        ProcessUserInput::ContentBlocks(blocks) => blocks.iter().find_map(|b| {
            if let ContentBlockParam::Text { text } = b {
                Some(text.clone())
            } else {
                None
            }
        }),
    };

    let mut preceding_input_blocks: Vec<ContentBlockParam> = vec![];
    let mut normalized_input = input.clone();

    // Handle content blocks - extract text and preceding blocks
    if let ProcessUserInput::ContentBlocks(blocks) = &input {
        if !blocks.is_empty() {
            let last_block = blocks.last().unwrap();
            if let ContentBlockParam::Text { text } = last_block {
                let text = text.clone();
                preceding_input_blocks = blocks[..blocks.len() - 1].to_vec();
                normalized_input = ProcessUserInput::String(text);
            } else {
                preceding_input_blocks = blocks.clone();
            }
        }
    }

    // Validate mode requires string input
    if input_string.is_none() && mode != PromptInputMode::Prompt {
        return Err(format!("Mode: {:?} requires a string input.", mode));
    }

    // Process pasted images
    let image_content_blocks = process_pasted_images(pasted_contents.as_ref()).await;

    // Check for bridge-safe slash command override
    let effective_skip_slash = check_bridge_safe_slash_command(
        bridge_origin,
        input_string.as_deref(),
        skip_slash_commands,
    );

    // Handle bash commands
    if let Some(input) = input_string {
        if mode == PromptInputMode::Bash {
            // Process bash command (simplified)
            return process_bash_command(input, preceding_input_blocks, vec![]);
        }

        // Handle slash commands
        if !effective_skip_slash && input.starts_with('/') {
            return process_slash_command(
                input,
                preceding_input_blocks,
                image_content_blocks,
                vec![],
            );
        }
    }

    // Regular user prompt
    process_text_prompt(
        normalized_input,
        image_content_blocks,
        vec![],
        uuid,
        None, // permission_mode
        is_meta,
    )
}

/// Check if slash commands should be skipped for bridge origin
fn check_bridge_safe_slash_command(
    bridge_origin: Option<bool>,
    input_string: Option<&str>,
    skip_slash_commands: Option<bool>,
) -> bool {
    if bridge_origin != Some(true) {
        return skip_slash_commands.unwrap_or(false);
    }

    let input = match input_string {
        Some(s) => s,
        None => return skip_slash_commands.unwrap_or(false),
    };

    if !input.starts_with('/') {
        return skip_slash_commands.unwrap_or(false);
    }

    // For bridge origin with slash command, we don't skip
    false
}

/// Process pasted images
async fn process_pasted_images(
    pasted_contents: Option<&std::collections::HashMap<u32, PastedContent>>,
) -> Vec<ContentBlockParam> {
    if pasted_contents.is_none() {
        return vec![];
    }

    let contents = pasted_contents.unwrap();
    let mut image_blocks = vec![];

    for (_, pasted) in contents.iter() {
        let media_type = pasted.media_type.as_deref().unwrap_or("image/png");
        image_blocks.push(ContentBlockParam::Image {
            source: ImageSource {
                source_type: "base64".to_string(),
                media_type: media_type.to_string(),
                data: pasted.content.clone(),
            },
        });
    }

    image_blocks
}

/// Process text prompt
fn process_text_prompt(
    input: ProcessUserInput,
    _image_content_blocks: Vec<ContentBlockParam>,
    _attachment_messages: Vec<Message>,
    uuid: Option<String>,
    _permission_mode: Option<crate::types::api_types::PermissionMode>,
    is_meta: Option<bool>,
) -> Result<ProcessUserInputBaseResult, String> {
    let content = match input {
        ProcessUserInput::String(s) => {
            if s.trim().is_empty() {
                vec![]
            } else {
                vec![Value::String(s)]
            }
        }
        ProcessUserInput::ContentBlocks(blocks) => blocks
            .iter()
            .map(|b| serde_json::to_value(b).unwrap_or(Value::Null))
            .collect(),
    };

    let message = Message {
        role: crate::types::MessageRole::User,
        content: serde_json::json!({ "type": "text", "text": content }).to_string(),
        attachments: None,
        tool_call_id: None,
        tool_calls: None,
        is_error: None,
        is_meta: None,
    };

    Ok(ProcessUserInputBaseResult {
        messages: vec![message],
        should_query: true,
        ..Default::default()
    })
}

/// Process bash command (simplified stub)
fn process_bash_command(
    _input: String,
    _preceding_input_blocks: Vec<ContentBlockParam>,
    _attachment_messages: Vec<Message>,
) -> Result<ProcessUserInputBaseResult, String> {
    // Simplified stub - full implementation would be in processBashCommand.tsx
    Ok(ProcessUserInputBaseResult {
        messages: vec![],
        should_query: false,
        allowed_tools: None,
        model: None,
        effort: None,
        result_text: Some("Bash command processing not yet implemented".to_string()),
        next_input: None,
        submit_next_input: None,
    })
}

/// Process slash command (simplified stub)
fn process_slash_command(
    _input: String,
    _preceding_input_blocks: Vec<ContentBlockParam>,
    _image_content_blocks: Vec<ContentBlockParam>,
    _attachment_messages: Vec<Message>,
) -> Result<ProcessUserInputBaseResult, String> {
    // Simplified stub - full implementation would be in processSlashCommand.tsx
    Ok(ProcessUserInputBaseResult {
        messages: vec![],
        should_query: false,
        allowed_tools: None,
        model: None,
        effort: None,
        result_text: Some("Slash command processing not yet implemented".to_string()),
        next_input: None,
        submit_next_input: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_user_input_default() {
        let options = ProcessUserInputOptions::default();
        assert!(matches!(options.input, ProcessUserInput::String(s) if s.is_empty()));
        assert_eq!(options.mode, PromptInputMode::Prompt);
    }

    #[test]
    fn test_process_text_prompt() {
        let result = process_text_prompt(
            ProcessUserInput::String("Hello".to_string()),
            vec![],
            vec![],
            Some("test-uuid".to_string()),
            None,
            Some(true),
        )
        .unwrap();

        assert!(result.should_query);
        assert_eq!(result.messages.len(), 1);
    }
}
