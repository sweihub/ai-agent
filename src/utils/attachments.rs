// Source: /data/home/swei/claudecode/openclaudecode/src/tools/BriefTool/attachments.ts
#![allow(dead_code)]

use crate::constants::env::ai_code;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

use serde::{Deserialize, Serialize};

// ============ Configuration Constants ============

pub const TODO_REMINDER_CONFIG: TodoReminderConfig = TodoReminderConfig {
    turns_since_write: 10,
    turns_between_reminders: 10,
};

#[derive(Debug, Clone, Copy)]
pub struct TodoReminderConfig {
    pub turns_since_write: u32,
    pub turns_between_reminders: u32,
}

pub const PLAN_MODE_ATTACHMENT_CONFIG: PlanModeAttachmentConfig = PlanModeAttachmentConfig {
    turns_between_attachments: 5,
    full_reminder_every_n_attachments: 5,
};

#[derive(Debug, Clone, Copy)]
pub struct PlanModeAttachmentConfig {
    pub turns_between_attachments: u32,
    pub full_reminder_every_n_attachments: u32,
}

pub const AUTO_MODE_ATTACHMENT_CONFIG: AutoModeAttachmentConfig = AutoModeAttachmentConfig {
    turns_between_attachments: 5,
    full_reminder_every_n_attachments: 5,
};

#[derive(Debug, Clone, Copy)]
pub struct AutoModeAttachmentConfig {
    pub turns_between_attachments: u32,
    pub full_reminder_every_n_attachments: u32,
}

const MAX_MEMORY_LINES: usize = 200;
const MAX_MEMORY_BYTES: usize = 4096;

pub const RELEVANT_MEMORIES_CONFIG: RelevantMemoriesConfig = RelevantMemoriesConfig {
    max_session_bytes: 60 * 1024,
};

#[derive(Debug, Clone, Copy)]
pub struct RelevantMemoriesConfig {
    pub max_session_bytes: usize,
}

pub const VERIFY_PLAN_REMINDER_CONFIG: VerifyPlanReminderConfig = VerifyPlanReminderConfig {
    turns_between_reminders: 10,
};

#[derive(Debug, Clone, Copy)]
pub struct VerifyPlanReminderConfig {
    pub turns_between_reminders: u32,
}

// ============ Attachment Types ============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Attachment {
    // File attachments
    File(FileAttachment),
    CompactFileReference(CompactFileReferenceAttachment),
    PdfReference(PdfReferenceAttachment),
    AlreadyReadFile(AlreadyReadFileAttachment),
    
    // Edited files
    EditedTextFile(EditedTextFileAttachment),
    EditedImageFile(EditedImageFileAttachment),
    
    // Directory
    Directory(DirectoryAttachment),
    
    // IDE selections
    SelectedLinesInIde(SelectedLinesInIdeAttachment),
    OpenedFileInIde(OpenedFileInIdeAttachment),
    
    // Reminders
    TodoReminder(TodoReminderAttachment),
    TaskReminder(TaskReminderAttachment),
    
    // Memory
    NestedMemory(NestedMemoryAttachment),
    RelevantMemories(RelevantMemoriesAttachment),
    CurrentSessionMemory(CurrentSessionMemoryAttachment),
    
    // Skills
    DynamicSkill(DynamicSkillAttachment),
    SkillListing(SkillListingAttachment),
    SkillDiscovery(SkillDiscoveryAttachment),
    InvokedSkills(InvokedSkillsAttachment),
    
    // Commands
    QueuedCommand(QueuedCommandAttachment),
    CommandPermissions(CommandPermissionsAttachment),
    
    // Output
    OutputStyle(OutputStyleAttachment),
    Diagnostics(DiagnosticsAttachment),
    
    // Mode
    PlanMode(PlanModeAttachment),
    PlanModeReentry(PlanModeReentryAttachment),
    PlanModeExit(PlanModeExitAttachment),
    AutoMode(AutoModeAttachment),
    AutoModeExit(AutoModeExitAttachment),
    
    // System
    CriticalSystemReminder(CriticalSystemReminderAttachment),
    PlanFileReference(PlanFileReferenceAttachment),
    McpResource(McpResourceAttachment),
    
    // Token/Budget
    TokenUsage(TokenUsageAttachment),
    BudgetUsd(BudgetUsdAttachment),
    OutputTokenUsage(OutputTokenUsageAttachment),
    
    // Agent
    AgentMention(AgentMentionAttachment),
    TaskStatus(TaskStatusAttachment),
    
    // Hooks
    AsyncHookResponse(AsyncHookResponseAttachment),
    
    // Misc
    StructuredOutput(StructuredOutputAttachment),
    VerifyPlanReminder(VerifyPlanReminderAttachment),
    MaxTurnsReached(MaxTurnsReachedAttachment),
    TeammateShutdownBatch(TeammateShutdownBatchAttachment),
    CompactionReminder(CompactionReminderAttachment),
    ContextEfficiency(ContextEfficiencyAttachment),
    DateChange(DateChangeAttachment),
    UltrathinkEffort(UltrathinkEffortAttachment),
    DeferredToolsDelta(DeferredToolsDeltaAttachment),
    AgentListingDelta(AgentListingDeltaAttachment),
    McpInstructionsDelta(McpInstructionsDeltaAttachment),
    CompanionIntro(CompanionIntroAttachment),
    BagelConsole(BagelConsoleAttachment),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttachment {
    pub filename: String,
    pub content: FileReadToolOutput,
    pub truncated: Option<bool>,
    pub display_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactFileReferenceAttachment {
    pub filename: String,
    pub display_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfReferenceAttachment {
    pub filename: String,
    pub page_count: u32,
    pub file_size: u64,
    pub display_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlreadyReadFileAttachment {
    pub filename: String,
    pub content: FileReadToolOutput,
    pub truncated: Option<bool>,
    pub display_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditedTextFileAttachment {
    pub filename: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditedImageFileAttachment {
    pub filename: String,
    pub content: FileReadToolOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryAttachment {
    pub path: String,
    pub content: String,
    pub display_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedLinesInIdeAttachment {
    pub ide_name: String,
    pub line_start: u32,
    pub line_end: u32,
    pub filename: String,
    pub content: String,
    pub display_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenedFileInIdeAttachment {
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoReminderAttachment {
    pub content: TodoList,
    pub item_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskReminderAttachment {
    pub content: Vec<Task>,
    pub item_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedMemoryAttachment {
    pub path: String,
    pub content: MemoryFileInfo,
    pub display_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevantMemoriesAttachment {
    pub memories: Vec<RelevantMemory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevantMemory {
    pub path: String,
    pub content: String,
    pub mtime_ms: u64,
    pub header: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentSessionMemoryAttachment {
    pub content: String,
    pub path: String,
    pub token_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicSkillAttachment {
    pub skill_dir: String,
    pub skill_names: Vec<String>,
    pub display_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillListingAttachment {
    pub content: String,
    pub skill_count: u32,
    pub is_initial: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDiscoveryAttachment {
    pub skills: Vec<SkillInfo>,
    pub signal: DiscoverySignal,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub short_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverySignal {
    // Placeholder for discovery signal type
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvokedSkillsAttachment {
    pub skills: Vec<InvokedSkill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvokedSkill {
    pub name: String,
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedCommandAttachment {
    pub prompt: QueuedCommandPrompt,
    pub source_uuid: Option<String>,
    pub image_paste_ids: Option<Vec<u32>>,
    pub command_mode: Option<String>,
    pub origin: Option<MessageOrigin>,
    pub is_meta: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QueuedCommandPrompt {
    String(String),
    ContentBlocks(Vec<ContentBlock>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: Option<String>,
    pub source: Option<ImageSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageOrigin {
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPermissionsAttachment {
    pub allowed_tools: Vec<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputStyleAttachment {
    pub style: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsAttachment {
    pub files: Vec<DiagnosticFile>,
    pub is_new: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticFile {
    pub path: String,
    // Additional diagnostic fields
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanModeAttachment {
    pub reminder_type: String,
    pub is_sub_agent: Option<bool>,
    pub plan_file_path: String,
    pub plan_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanModeReentryAttachment {
    pub plan_file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanModeExitAttachment {
    pub plan_file_path: String,
    pub plan_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoModeAttachment {
    pub reminder_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoModeExitAttachment {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalSystemReminderAttachment {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanFileReferenceAttachment {
    pub plan_file_path: String,
    pub plan_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceAttachment {
    pub server: String,
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub content: ReadResourceResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResult {
    // Resource result contents
    #[serde(default)]
    pub contents: Vec<ResourceContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: Option<String>,
    pub mime_type: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageAttachment {
    pub used: u32,
    pub total: u32,
    pub remaining: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetUsdAttachment {
    pub used: f64,
    pub total: f64,
    pub remaining: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputTokenUsageAttachment {
    pub turn: u32,
    pub session: u32,
    pub budget: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMentionAttachment {
    pub agent_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusAttachment {
    pub task_id: String,
    pub task_type: String,
    pub status: String,
    pub description: String,
    pub delta_summary: Option<String>,
    pub output_file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncHookResponseAttachment {
    pub process_id: String,
    pub hook_name: String,
    pub hook_event: String,
    pub tool_name: Option<String>,
    pub response: SyncHookJsonOutput,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHookJsonOutput {
    // Hook output fields
    #[serde(default)]
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredOutputAttachment {
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyPlanReminderAttachment {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaxTurnsReachedAttachment {
    pub max_turns: u32,
    pub turn_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeammateShutdownBatchAttachment {
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionReminderAttachment {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEfficiencyAttachment {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateChangeAttachment {
    pub new_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UltrathinkEffortAttachment {
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferredToolsDeltaAttachment {
    pub added_names: Vec<String>,
    pub added_lines: Vec<String>,
    pub removed_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentListingDeltaAttachment {
    pub added_types: Vec<String>,
    pub added_lines: Vec<String>,
    pub removed_types: Vec<String>,
    pub is_initial: bool,
    pub show_concurrency_note: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpInstructionsDeltaAttachment {
    pub added_names: Vec<String>,
    pub added_blocks: Vec<String>,
    pub removed_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionIntroAttachment {
    pub name: String,
    pub species: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BagelConsoleAttachment {
    pub error_count: u32,
    pub warning_count: u32,
    pub sample: String,
}

// ============ Supporting Types ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadToolOutput {
    pub content: String,
    #[serde(default)]
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoList {
    // Todo list fields
    #[serde(default)]
    pub items: Vec<TodoItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub content: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFileInfo {
    pub path: String,
    pub frontmatter: Option<String>,
    pub content: String,
}

// ============ Main Functions ============

/// Get attachments for the current context
pub async fn get_attachments(
    input: Option<String>,
    tool_use_context: ToolUseContext,
    ide_selection: Option<IdeSelection>,
    queued_commands: Vec<QueuedCommand>,
    messages: Option<Vec<Message>>,
    query_source: Option<QuerySource>,
    options: Option<GetAttachmentsOptions>,
) -> Vec<Attachment> {
    // Check if attachments are disabled
    if is_env_truthy("AI_CODE_DISABLE_ATTACHMENTS") || is_env_truthy("AI_CODE_SIMPLE") {
        return get_queued_command_attachments(queued_commands);
    }

    // Process user input attachments first
    let mut user_attachments = vec![];
    if let Some(input_str) = input {
        // Process at-mentioned files
        if let Ok(files) = process_at_mentioned_files(&input_str, &tool_use_context) {
            user_attachments.extend(files);
        }

        // Process MCP resources
        if let Ok(resources) = process_mcp_resource_attachments(&input_str, &tool_use_context) {
            user_attachments.extend(resources);
        }

        // Process agent mentions
        if let Ok(mentions) = process_agent_mentions(&input_str, &tool_use_context) {
            user_attachments.extend(mentions);
        }
    }

    // Process thread-safe attachments
    let mut thread_attachments = vec![];

    // Queued commands
    thread_attachments.extend(get_queued_command_attachments(queued_commands));

    // Date change
    if let Some(msgs) = messages {
        thread_attachments.extend(get_date_change_attachments(&msgs));
    }

    // Changed files
    if let Ok(files) = get_changed_files(&tool_use_context) {
        thread_attachments.extend(files);
    }

    // Nested memory
    if let Ok(memory) = get_nested_memory_attachments(&tool_use_context) {
        thread_attachments.extend(memory);
    }

    // Plan mode
    if let Some(msgs) = messages {
        thread_attachments.extend(get_plan_mode_attachments(&msgs, &tool_use_context).await);
    }

    // Todo reminders
    thread_attachments.extend(get_todo_reminder_attachments(&messages, &tool_use_context).await);

    // Main thread attachments (only for main thread)
    let mut main_attachments = vec![];
    if !tool_use_context.is_sub_agent {
        if let Some(selection) = ide_selection {
            if let Ok(lines) = get_selected_lines_from_ide(&selection, &tool_use_context) {
                main_attachments.extend(lines);
            }
        }

        // Token usage
        if let Some(msgs) = messages {
            main_attachments.push(get_token_usage_attachment(&msgs, &tool_use_context));
        }
    }

    // Combine all attachments
    let mut all_attachments = user_attachments;
    all_attachments.extend(thread_attachments);
    all_attachments.extend(main_attachments);

    // Filter out None values
    all_attachments.into_iter().filter(|a| a.is_some()).map(|a| a.unwrap()).collect()
}

// Helper to get attachment as Option
fn get_attachment_option(attachment: Attachment) -> Option<Attachment> {
    Some(attachment)
}

/// Get queued command attachments
pub fn get_queued_command_attachments(queued_commands: Vec<QueuedCommand>) -> Vec<Attachment> {
    queued_commands
        .into_iter()
        .filter(|cmd| cmd.mode == "prompt" || cmd.mode == "task-notification")
        .map(|cmd| {
            Attachment::QueuedCommand(QueuedCommandAttachment {
                prompt: QueuedCommandPrompt::String(cmd.value),
                source_uuid: Some(cmd.uuid),
                image_paste_ids: Some(cmd.image_paste_ids),
                command_mode: Some(cmd.mode),
                origin: Some(MessageOrigin { kind: "user".to_string() }),
                is_meta: Some(cmd.is_meta),
            })
        })
        .collect()
}

fn get_date_change_attachments(messages: &[Message]) -> Vec<Attachment> {
    let current_date = get_local_iso_date();
    let last_date = get_last_emitted_date();

    if last_date.is_none() {
        // First turn — just record, no attachment needed
        set_last_emitted_date(&current_date);
        return vec![];
    }

    if current_date == *last_date.as_ref().unwrap() {
        return vec![];
    }

    set_last_emitted_date(&current_date);
    vec![Attachment::DateChange(DateChangeAttachment { new_date: current_date })]
}

fn get_plan_mode_attachments(
    messages: &[Message],
    tool_use_context: &ToolUseContext,
) -> Vec<Attachment> {
    let app_state = tool_use_context.get_app_state();
    let permission_context = app_state.tool_permission_context();
    
    if permission_context.mode != "plan" {
        return vec![];
    }

    // Check turn count
    let (turn_count, found_attachment) = get_plan_mode_attachment_turn_count(messages);
    if found_attachment && turn_count < PLAN_MODE_ATTACHMENT_CONFIG.turns_between_attachments {
        return vec![];
    }

    let plan_file_path = get_plan_file_path(tool_use_context.agent_id.as_ref());
    let existing_plan = get_plan(tool_use_context.agent_id.as_ref());

    let mut attachments = vec![];

    // Check for re-entry
    if has_exited_plan_mode_in_session() && existing_plan.is_some() {
        attachments.push(Attachment::PlanModeReentry(PlanModeReentryAttachment {
            plan_file_path: plan_file_path.clone(),
        }));
        set_has_exited_plan_mode(false);
    }

    // Determine reminder type
    let attachment_count = count_plan_mode_attachments_since_last_exit(messages) + 1;
    let reminder_type = if attachment_count % PLAN_MODE_ATTACHMENT_CONFIG.full_reminder_every_n_attachments == 1 {
        "full"
    } else {
        "sparse"
    };

    attachments.push(Attachment::PlanMode(PlanModeAttachment {
        reminder_type: reminder_type.to_string(),
        is_sub_agent: tool_use_context.agent_id.is_some(),
        plan_file_path,
        plan_exists: existing_plan.is_some(),
    }));

    attachments
}

fn get_plan_mode_attachment_turn_count(messages: &[Message]) -> (u32, bool) {
    let mut turns_since_last_attachment = 0u32;
    let mut found_plan_mode_attachment = false;

    for i in (0..messages.len()).rev() {
        let message = &messages[i];

        if let Message::User(msg) = message {
            if !msg.is_meta && !has_tool_result_content(&msg.content) {
                turns_since_last_attachment += 1;
            }
        } else if let Message::Attachment(attachment) = message {
            if matches!(attachment, Attachment::PlanMode(_) | Attachment::PlanModeReentry(_)) {
                found_plan_mode_attachment = true;
                break;
            }
        }
    }

    (turns_since_last_attachment, found_plan_mode_attachment)
}

fn count_plan_mode_attachments_since_last_exit(messages: &[Message]) -> u32 {
    let mut count = 0u32;
    for i in (0..messages.len()).rev() {
        if let Message::Attachment(attachment) = &messages[i] {
            match attachment {
                Attachment::PlanModeExit(_) => break,
                Attachment::PlanMode(_) => count += 1,
                _ => {}
            }
        }
    }
    count
}

fn get_todo_reminder_attachments(
    messages: &Option<Vec<Message>>,
    tool_use_context: &ToolUseContext,
) -> Vec<Attachment> {
    // Simplified - would check todo list and return reminder if needed
    vec![]
}

fn get_changed_files(_context: &ToolUseContext) -> Result<Vec<Attachment>, String> {
    // Would implement file change detection
    Ok(vec![])
}

fn get_nested_memory_attachments(_context: &ToolUseContext) -> Result<Vec<Attachment>, String> {
    // Would implement nested memory processing
    Ok(vec![])
}

fn get_token_usage_attachment(messages: &[Message], tool_use_context: &ToolUseContext) -> Attachment {
    // Simplified token usage calculation
    Attachment::TokenUsage(TokenUsageAttachment {
        used: 0,
        total: 200000,
        remaining: 200000,
    })
}

fn get_selected_lines_from_ide(
    _selection: &IdeSelection,
    _context: &ToolUseContext,
) -> Result<Vec<Attachment>, String> {
    // Would get IDE selection
    Ok(vec![])
}

// ============ Helper Functions ============

fn process_at_mentioned_files(input: &str, _context: &ToolUseContext) -> Result<Vec<Attachment>, String> {
    // Extract @-mentioned files from input
    let mut attachments = vec![];
    
    // Simple regex for @filename
    let re = regex::Regex::new(r"@([^\s]+)").unwrap();
    for cap in re.captures_iter(input) {
        if let Some(filename) = cap.get(1) {
            let filename = filename.as_str();
            // Would read the file and create attachment
            attachments.push(Attachment::File(FileAttachment {
                filename: filename.to_string(),
                content: FileReadToolOutput {
                    content: format!("Content of {}", filename),
                    truncated: false,
                },
                truncated: Some(false),
                display_path: filename.to_string(),
            }));
        }
    }
    
    Ok(attachments)
}

fn process_mcp_resource_attachments(_input: &str, _context: &ToolUseContext) -> Result<Vec<Attachment>, String> {
    // Would process MCP resource mentions
    Ok(vec![])
}

fn process_agent_mentions(input: &str, context: &ToolUseContext) -> Result<Vec<Attachment>, String> {
    let mut attachments = vec![];
    
    // Look for @agent mentions
    let re = regex::Regex::new(r"@(\w+)").unwrap();
    for cap in re.captures_iter(input) {
        if let Some(agent_type) = cap.get(1) {
            let agent_type = agent_type.as_str();
            // Check if it's a valid agent
            let active_agents = context.get_active_agents();
            if active_agents.iter().any(|a| a.agent_type == agent_type) {
                attachments.push(Attachment::AgentMention(AgentMentionAttachment {
                    agent_type: agent_type.to_string(),
                }));
            }
        }
    }
    
    Ok(attachments)
}

fn has_tool_result_content(_content: &str) -> bool {
    // Would check if content is tool result
    false
}

// ============ State Management ============

static LAST_EMITTED_DATE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

fn get_local_iso_date() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now();
    let duration = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let secs = duration.as_secs();
    // Simplified - would use chrono or similar
    format!("{}", secs / 86400)
}

fn get_last_emitted_date() -> Option<String> {
    LAST_EMITTED_DATE.lock().ok().and_then(|g| g.clone())
}

fn set_last_emitted_date(date: &str) {
    if let Ok(mut guard) = LAST_EMITTED_DATE.lock() {
        *guard = Some(date.to_string());
    }
}

fn has_exited_plan_mode_in_session() -> bool {
    std::env::var(ai_code::PLAN_MODE_EXITED).is_ok()
}

fn set_has_exited_plan_mode(exited: bool) {
    if exited {
        std::env::set_var(ai_code::PLAN_MODE_EXITED, "1");
    } else {
        std::env::remove_var(ai_code::PLAN_MODE_EXITED);
    }
}

fn get_plan_file_path(_agent_id: Option<&str>) -> String {
    // Would get plan file path
    ".plan.md".to_string()
}

fn get_plan(_agent_id: Option<&str>) -> Option<String> {
    // Would read plan content
    None
}

// ============ Context Types ============

#[derive(Clone)]
pub struct ToolUseContext {
    pub agent_id: Option<String>,
    pub get_app_state: Arc<dyn Fn() -> AppState + Send + Sync>,
    pub options: ToolUseContextOptions,
}

impl ToolUseContext {
    pub fn get_active_agents(&self) -> Vec<AgentDefinition> {
        vec![]
    }
}

pub struct ToolUseContextOptions {
    pub tools: Vec<ToolInfo>,
    pub agent_definitions: AgentDefinitions,
    pub mcp_clients: Vec<McpClient>,
    pub main_loop_model: String,
    pub max_budget_usd: Option<f64>,
}

pub struct ToolInfo {
    pub name: String,
    pub is_mcp: bool,
}

pub struct AgentDefinitions {
    pub active_agents: Vec<AgentDefinition>,
    pub allowed_agent_types: Option<Vec<String>>,
}

pub struct AgentDefinition {
    pub agent_type: String,
    pub name: Option<String>,
    pub team_name: Option<String>,
}

pub struct McpClient {
    pub name: String,
    // Additional fields
}

#[derive(Clone)]
pub struct AppState {
    tool_permission_context: ToolPermissionContext,
}

impl AppState {
    pub fn tool_permission_context(&self) -> &ToolPermissionContext {
        &self.tool_permission_context
    }
}

pub struct ToolPermissionContext {
    pub mode: String,
}

pub struct IdeSelection {
    pub filename: String,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub content: Option<String>,
    pub ide_name: Option<String>,
}

pub struct QueuedCommand {
    pub uuid: String,
    pub value: String,
    pub mode: String,
    pub is_meta: bool,
    pub image_paste_ids: Vec<u32>,
    pub pasted_contents: Option<HashMap<u32, PastedContent>>,
}

#[derive(Debug, Clone)]
pub struct PastedContent {
    pub content: String,
    pub media_type: Option<String>,
}

pub struct Message {
    // Message variants would be enum
}

impl Message {
    // Placeholder - actual implementation would have variants
}

pub enum QuerySource {
    Main,
    SessionMemory,
    // etc
}

pub struct GetAttachmentsOptions {
    pub skip_skill_discovery: Option<bool>,
}

fn is_env_truthy(var: &str) -> bool {
    std::env::var(var).map(|v| v == "1" || v == "true").unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_queued_command_attachments() {
        let commands = vec![
            QueuedCommand {
                uuid: "1".to_string(),
                value: "test command".to_string(),
                mode: "prompt".to_string(),
                is_meta: false,
                image_paste_ids: vec![],
                pasted_contents: None,
            },
        ];

        let attachments = get_queued_command_attachments(commands);
        assert_eq!(attachments.len(), 1);
    }

    #[test]
    fn test_process_at_mentioned_files() {
        let context = ToolUseContext {
            agent_id: None,
            get_app_state: Arc::new(|| AppState {
                tool_permission_context: ToolPermissionContext { mode: "default".to_string() },
            }),
            options: ToolUseContextOptions {
                tools: vec![],
                agent_definitions: AgentDefinitions {
                    active_agents: vec![],
                    allowed_agent_types: None,
                },
                mcp_clients: vec![],
                main_loop_model: "claude-sonnet-4-20250514".to_string(),
                max_budget_usd: None,
            },
        };

        let result = process_at_mentioned_files("@file.txt and @other.txt", &context);
        assert!(result.is_ok());
    }
}
