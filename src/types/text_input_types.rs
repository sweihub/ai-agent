// Source: ~/claudecode/openclaudecode/src/types/textInputTypes.ts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::types::ids::AgentId;
use crate::types::message::{AssistantMessage, MessageOrigin};

/// Inline ghost text for mid-input command autocomplete.
#[derive(Debug, Clone)]
pub struct InlineGhostText {
    /// The ghost text to display (e.g., "mit" for /commit)
    pub text: String,
    /// The full command name (e.g., "commit")
    pub full_command: String,
    /// Position in the input where the ghost text should appear
    pub insert_position: usize,
}

/// Key type for input handling.
pub type Key = serde_json::Value;

/// Base props for text input components.
pub struct BaseTextInputProps {
    /// Optional callback for handling history navigation on up arrow at start of input
    pub on_history_up: Option<Box<dyn Fn() + Send + Sync>>,
    /// Optional callback for handling history navigation on down arrow at end of input
    pub on_history_down: Option<Box<dyn Fn() + Send + Sync>>,
    /// Text to display when `value` is empty.
    pub placeholder: Option<String>,
    /// Allow multi-line input via line ending with backslash (default: true)
    pub multiline: Option<bool>,
    /// Listen to user's input. Useful when there are multiple input components.
    pub focus: Option<bool>,
    /// Replace all chars and mask the value. Useful for password inputs.
    pub mask: Option<String>,
    /// Whether to show cursor and allow navigation inside text input with arrow keys.
    pub show_cursor: Option<bool>,
    /// Highlight pasted text
    pub highlight_pasted_text: Option<bool>,
    /// Value to display in a text input.
    pub value: String,
    /// Function to call when value updates.
    pub on_change: Box<dyn Fn(String) + Send + Sync>,
    /// Function to call when `Enter` is pressed.
    pub on_submit: Option<Box<dyn Fn(String) + Send + Sync>>,
    /// Function to call when Ctrl+C is pressed to exit.
    pub on_exit: Option<Box<dyn Fn() + Send + Sync>>,
    /// Optional callback to show exit message
    pub on_exit_message: Option<Box<dyn Fn(bool, Option<String>) + Send + Sync>>,
    /// Optional callback to reset history position
    pub on_history_reset: Option<Box<dyn Fn() + Send + Sync>>,
    /// Optional callback when input is cleared (e.g., double-escape)
    pub on_clear_input: Option<Box<dyn Fn() + Send + Sync>>,
    /// Number of columns to wrap text at
    pub columns: usize,
    /// Maximum visible lines for the input viewport.
    pub max_visible_lines: Option<usize>,
    /// Optional callback when an image is pasted
    pub on_image_paste: Option<Box<dyn Fn(String, Option<String>, Option<String>, Option<ImageDimensions>, Option<String>) + Send + Sync>>,
    /// Optional callback when a large text (over 800 chars) is pasted
    pub on_paste: Option<Box<dyn Fn(String) + Send + Sync>>,
    /// Callback when the pasting state changes
    pub on_is_pasting_change: Option<Box<dyn Fn(bool) + Send + Sync>>,
    /// Whether to disable cursor movement for up/down arrow keys
    pub disable_cursor_movement_for_up_down_keys: Option<bool>,
    /// Skip the text-level double-press escape handler.
    pub disable_escape_double_press: Option<bool>,
    /// The offset of the cursor within the text
    pub cursor_offset: usize,
    /// Callback to set the offset of the cursor
    pub on_change_cursor_offset: Box<dyn Fn(usize) + Send + Sync>,
    /// Optional hint text to display after command input
    pub argument_hint: Option<String>,
    /// Optional callback for undo functionality
    pub on_undo: Option<Box<dyn Fn() + Send + Sync>>,
    /// Whether to render the text with dim color
    pub dim_color: Option<bool>,
    /// Optional text highlights for search results or other highlighting
    pub highlights: Option<Vec<TextHighlight>>,
    /// Optional custom element to render as placeholder.
    pub placeholder_element: Option<String>, // Simplified - would be React.ReactNode in TS
    /// Optional inline ghost text for mid-input command autocomplete
    pub inline_ghost_text: Option<InlineGhostText>,
    /// Optional filter applied to raw input before key routing.
    pub input_filter: Option<Box<dyn Fn(String, Key) -> String + Send + Sync>>,
}

/// Image dimensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

/// Text highlight region.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextHighlight {
    pub start: usize,
    pub end: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Extended props for VimTextInput.
pub struct VimTextInputProps {
    pub base: BaseTextInputProps,
    /// Initial vim mode to use
    pub initial_mode: Option<VimMode>,
    /// Optional callback for mode changes
    pub on_mode_change: Option<Box<dyn Fn(VimMode) + Send + Sync>>,
}

/// Vim editor modes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VimMode {
    Insert,
    Normal,
}

/// Common properties for input hook results.
pub struct BaseInputState {
    pub on_input: Box<dyn Fn(String, Key) + Send + Sync>,
    pub rendered_value: String,
    pub offset: usize,
    pub set_offset: Box<dyn Fn(usize) + Send + Sync>,
    /// Cursor line (0-indexed) within the rendered text, accounting for wrapping.
    pub cursor_line: usize,
    /// Cursor column (display-width) within the current line.
    pub cursor_column: usize,
    /// Character offset in the full text where the viewport starts.
    pub viewport_char_offset: usize,
    /// Character offset in the full text where the viewport ends.
    pub viewport_char_end: usize,
    /// For paste handling
    pub is_pasting: Option<bool>,
    pub paste_state: Option<PasteState>,
}

/// Paste state for tracking chunked paste operations.
pub struct PasteState {
    pub chunks: Vec<String>,
    pub timeout_id: Option<String>, // Represents a timer handle
}

/// State for text input.
pub type TextInputState = BaseInputState;

/// State for vim input with mode.
pub struct VimInputState {
    pub base: BaseInputState,
    pub mode: VimMode,
    pub set_mode: Box<dyn Fn(VimMode) + Send + Sync>,
}

/// Input modes for the prompt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PromptInputMode {
    Bash,
    Prompt,
    #[serde(rename = "orphaned-permission")]
    OrphanedPermission,
    #[serde(rename = "task-notification")]
    TaskNotification,
}

/// Queue priority levels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum QueuePriority {
    /// Interrupt and send immediately. Aborts any in-flight tool call.
    Now,
    /// Mid-turn drain. Let the current tool call finish, then send.
    Next,
    /// End-of-turn drain. Wait for the current turn to finish.
    Later,
}

/// Queued command type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedCommand {
    pub value: CommandValue,
    pub mode: PromptInputMode,
    /// Defaults to the priority implied by `mode` when enqueued.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<QueuePriority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "orphanedPermission")]
    pub orphaned_permission: Option<OrphanedPermission>,
    /// Raw pasted contents including images.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "pastedContents")]
    pub pasted_contents: Option<HashMap<usize, PastedContent>>,
    /// The input string before [Pasted text #N] placeholders were expanded.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "preExpansionValue")]
    pub pre_expansion_value: Option<String>,
    /// When true, the input is treated as plain text even if it starts with `/`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "skipSlashCommands")]
    pub skip_slash_commands: Option<bool>,
    /// When true, slash commands are dispatched but filtered through isBridgeSafeCommand().
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bridgeOrigin")]
    pub bridge_origin: Option<bool>,
    /// When true, the resulting UserMessage gets `isMeta: true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isMeta")]
    pub is_meta: Option<bool>,
    /// Provenance of this command. undefined = human (keyboard).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<MessageOrigin>,
    /// Workload tag for billing-header attribution block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workload: Option<String>,
    /// Agent that should receive this notification. Undefined = main thread.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "agentId")]
    pub agent_id: Option<AgentId>,
}

/// Command value can be a string or array of content blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CommandValue {
    Text(String),
    Blocks(Vec<serde_json::Value>),
}

/// Pasted content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastedContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub content: String,
    pub id: usize,
}

/// Type guard for image PastedContent with non-empty data.
pub fn is_valid_image_paste(c: &PastedContent) -> bool {
    c.content_type == "image" && !c.content.is_empty()
}

/// Extract image paste IDs from a QueuedCommand's pastedContents.
pub fn get_image_paste_ids(pasted_contents: Option<&HashMap<usize, PastedContent>>) -> Option<Vec<usize>> {
    let map = pasted_contents?;
    let ids: Vec<usize> = map.values()
        .filter(|c| is_valid_image_paste(c))
        .map(|c| c.id)
        .collect();
    if ids.is_empty() {
        None
    } else {
        Some(ids)
    }
}

/// Orphaned permission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanedPermission {
    #[serde(rename = "permissionResult")]
    pub permission_result: serde_json::Value,
    #[serde(rename = "assistantMessage")]
    pub assistant_message: AssistantMessage,
}
