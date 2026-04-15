// Source: ~/claudecode/openclaudecode/src/types/logs.ts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::types::ids::AgentId;
use crate::types::message::Message;
use crate::types::message_queue_types::MessageQueueEntry;

/// A serialized message with session metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedMessage {
    #[serde(flatten)]
    pub base: Message,
    pub cwd: String,
    #[serde(rename = "userType")]
    pub user_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<String>,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub timestamp: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "gitBranch")]
    pub git_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
}

/// Log option representing a session log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogOption {
    pub date: String,
    pub messages: Vec<SerializedMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "fullPath")]
    pub full_path: Option<String>,
    pub value: i64,
    pub created: chrono::DateTime<chrono::Local>,
    pub modified: chrono::DateTime<chrono::Local>,
    #[serde(rename = "firstPrompt")]
    pub first_prompt: String,
    #[serde(rename = "messageCount")]
    pub message_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "fileSize")]
    pub file_size: Option<u64>,
    #[serde(rename = "isSidechain")]
    pub is_sidechain: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isLite")]
    pub is_lite: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "teamName")]
    pub team_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "agentName")]
    pub agent_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "agentColor")]
    pub agent_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "agentSetting")]
    pub agent_setting: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isTeammate")]
    pub is_teammate: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "leafUuid")]
    pub leaf_uuid: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "customTitle")]
    pub custom_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "fileHistorySnapshots")]
    pub file_history_snapshots: Option<Vec<FileHistorySnapshot>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "attributionSnapshots")]
    pub attribution_snapshots: Option<Vec<AttributionSnapshotMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "contextCollapseCommits")]
    pub context_collapse_commits: Option<Vec<ContextCollapseCommitEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "contextCollapseSnapshot")]
    pub context_collapse_snapshot: Option<ContextCollapseSnapshotEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "gitBranch")]
    pub git_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "projectPath")]
    pub project_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "prNumber")]
    pub pr_number: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "prUrl")]
    pub pr_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "prRepository")]
    pub pr_repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<SessionMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "worktreeSession")]
    pub worktree_session: Option<Option<PersistedWorktreeSession>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "contentReplacements")]
    pub content_replacements: Option<Vec<ContentReplacementRecord>>,
}

/// Session mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SessionMode {
    Coordinator,
    Normal,
}

/// Summary message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryMessage {
    #[serde(rename = "type")]
    pub message_type: String, // "summary"
    #[serde(rename = "leafUuid")]
    pub leaf_uuid: Uuid,
    pub summary: String,
}

/// Custom title message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTitleMessage {
    #[serde(rename = "type")]
    pub message_type: String, // "custom-title"
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    #[serde(rename = "customTitle")]
    pub custom_title: String,
}

/// AI-generated session title.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiTitleMessage {
    #[serde(rename = "type")]
    pub message_type: String, // "ai-title"
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    #[serde(rename = "aiTitle")]
    pub ai_title: String,
}

/// Last prompt message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastPromptMessage {
    #[serde(rename = "type")]
    pub message_type: String, // "last-prompt"
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    #[serde(rename = "lastPrompt")]
    pub last_prompt: String,
}

/// Periodic fork-generated summary of what the agent is currently doing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummaryMessage {
    #[serde(rename = "type")]
    pub message_type: String, // "task-summary"
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    pub summary: String,
    pub timestamp: String,
}

/// Tag message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagMessage {
    #[serde(rename = "type")]
    pub message_type: String, // "tag"
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    pub tag: String,
}

/// Agent name message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentNameMessage {
    #[serde(rename = "type")]
    pub message_type: String, // "agent-name"
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    #[serde(rename = "agentName")]
    pub agent_name: String,
}

/// Agent color message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentColorMessage {
    #[serde(rename = "type")]
    pub message_type: String, // "agent-color"
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    #[serde(rename = "agentColor")]
    pub agent_color: String,
}

/// Agent setting message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSettingMessage {
    #[serde(rename = "type")]
    pub message_type: String, // "agent-setting"
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    #[serde(rename = "agentSetting")]
    pub agent_setting: String,
}

/// PR link message stored in session transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRLinkMessage {
    #[serde(rename = "type")]
    pub message_type: String, // "pr-link"
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    #[serde(rename = "prNumber")]
    pub pr_number: i64,
    #[serde(rename = "prUrl")]
    pub pr_url: String,
    #[serde(rename = "prRepository")]
    pub pr_repository: String,
    pub timestamp: String,
}

/// Mode entry for session mode tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeEntry {
    #[serde(rename = "type")]
    pub message_type: String, // "mode"
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    pub mode: SessionMode,
}

/// Worktree session state persisted to the transcript for resume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedWorktreeSession {
    #[serde(rename = "originalCwd")]
    pub original_cwd: String,
    #[serde(rename = "worktreePath")]
    pub worktree_path: String,
    #[serde(rename = "worktreeName")]
    pub worktree_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "worktreeBranch")]
    pub worktree_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "originalBranch")]
    pub original_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "originalHeadCommit")]
    pub original_head_commit: Option<String>,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "tmuxSessionName")]
    pub tmux_session_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_based: Option<bool>,
}

/// Worktree session state entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeStateEntry {
    #[serde(rename = "type")]
    pub message_type: String, // "worktree-state"
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    #[serde(rename = "worktreeSession")]
    pub worktree_session: Option<PersistedWorktreeSession>,
}

/// Content replacement entry for resume reconstruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentReplacementEntry {
    #[serde(rename = "type")]
    pub message_type: String, // "content-replacement"
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "agentId")]
    pub agent_id: Option<AgentId>,
    pub replacements: Vec<ContentReplacementRecord>,
}

/// Content replacement record.
pub type ContentReplacementRecord = HashMap<String, serde_json::Value>;

/// File history snapshot.
pub type FileHistorySnapshot = HashMap<String, serde_json::Value>;

/// File history snapshot message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHistorySnapshotMessage {
    #[serde(rename = "type")]
    pub message_type: String, // "file-history-snapshot"
    #[serde(rename = "messageId")]
    pub message_id: Uuid,
    pub snapshot: FileHistorySnapshot,
    #[serde(rename = "isSnapshotUpdate")]
    pub is_snapshot_update: bool,
}

/// Per-file attribution state tracking Claude's character contributions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttributionState {
    #[serde(rename = "contentHash")]
    pub content_hash: String,
    #[serde(rename = "claudeContribution")]
    pub claude_contribution: i64,
    pub mtime: i64,
}

/// Attribution snapshot message stored in session transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionSnapshotMessage {
    #[serde(rename = "type")]
    pub message_type: String, // "attribution-snapshot"
    #[serde(rename = "messageId")]
    pub message_id: Uuid,
    pub surface: String,
    #[serde(rename = "fileStates")]
    pub file_states: HashMap<String, FileAttributionState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "promptCount")]
    pub prompt_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "promptCountAtLastCommit")]
    pub prompt_count_at_last_commit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "permissionPromptCount")]
    pub permission_prompt_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "permissionPromptCountAtLastCommit")]
    pub permission_prompt_count_at_last_commit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "escapeCount")]
    pub escape_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "escapeCountAtLastCommit")]
    pub escape_count_at_last_commit: Option<i64>,
}

/// Transcript message with additional fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptMessage {
    #[serde(flatten)]
    pub base: SerializedMessage,
    #[serde(rename = "parentUuid")]
    pub parent_uuid: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "logicalParentUuid")]
    pub logical_parent_uuid: Option<Option<Uuid>>,
    #[serde(rename = "isSidechain")]
    pub is_sidechain: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "gitBranch")]
    pub git_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "agentId")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "teamName")]
    pub team_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "agentName")]
    pub agent_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "agentColor")]
    pub agent_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "promptId")]
    pub prompt_id: Option<String>,
}

/// Speculation accept message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeculationAcceptMessage {
    #[serde(rename = "type")]
    pub message_type: String, // "speculation-accept"
    pub timestamp: String,
    #[serde(rename = "timeSavedMs")]
    pub time_saved_ms: i64,
}

/// Persisted context-collapse commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCollapseCommitEntry {
    #[serde(rename = "type")]
    pub message_type: String, // "marble-origami-commit"
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    #[serde(rename = "collapseId")]
    pub collapse_id: String,
    #[serde(rename = "summaryUuid")]
    pub summary_uuid: String,
    #[serde(rename = "summaryContent")]
    pub summary_content: String,
    pub summary: String,
    #[serde(rename = "firstArchivedUuid")]
    pub first_archived_uuid: String,
    #[serde(rename = "lastArchivedUuid")]
    pub last_archived_uuid: String,
}

/// Snapshot of the staged queue and spawn trigger state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCollapseSnapshotEntry {
    #[serde(rename = "type")]
    pub message_type: String, // "marble-origami-snapshot"
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    pub staged: Vec<StagedSpan>,
    pub armed: bool,
    #[serde(rename = "lastSpawnTokens")]
    pub last_spawn_tokens: i64,
}

/// A staged span in the context collapse snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedSpan {
    #[serde(rename = "startUuid")]
    pub start_uuid: String,
    #[serde(rename = "endUuid")]
    pub end_uuid: String,
    pub summary: String,
    pub risk: f64,
    #[serde(rename = "stagedAt")]
    pub staged_at: i64,
}

/// Entry union of all transcript entry types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Entry {
    #[serde(rename = "user")]
    Transcript(TranscriptMessage),
    #[serde(rename = "summary")]
    Summary(SummaryMessage),
    #[serde(rename = "custom-title")]
    CustomTitle(CustomTitleMessage),
    #[serde(rename = "ai-title")]
    AiTitle(AiTitleMessage),
    #[serde(rename = "last-prompt")]
    LastPrompt(LastPromptMessage),
    #[serde(rename = "task-summary")]
    TaskSummary(TaskSummaryMessage),
    #[serde(rename = "tag")]
    Tag(TagMessage),
    #[serde(rename = "agent-name")]
    AgentName(AgentNameMessage),
    #[serde(rename = "agent-color")]
    AgentColor(AgentColorMessage),
    #[serde(rename = "agent-setting")]
    AgentSetting(AgentSettingMessage),
    #[serde(rename = "pr-link")]
    PRLink(PRLinkMessage),
    #[serde(rename = "file-history-snapshot")]
    FileHistorySnapshot(FileHistorySnapshotMessage),
    #[serde(rename = "attribution-snapshot")]
    AttributionSnapshot(AttributionSnapshotMessage),
    #[serde(rename = "queue_operation")]
    QueueOperation(MessageQueueEntry),
    #[serde(rename = "speculation-accept")]
    SpeculationAccept(SpeculationAcceptMessage),
    #[serde(rename = "mode")]
    Mode(ModeEntry),
    #[serde(rename = "worktree-state")]
    WorktreeState(WorktreeStateEntry),
    #[serde(rename = "content-replacement")]
    ContentReplacement(ContentReplacementEntry),
    #[serde(rename = "marble-origami-commit")]
    ContextCollapseCommit(ContextCollapseCommitEntry),
    #[serde(rename = "marble-origami-snapshot")]
    ContextCollapseSnapshot(ContextCollapseSnapshotEntry),
}

/// Sort logs by modified date (newest first), then by created date.
pub fn sort_logs(logs: &mut Vec<LogOption>) {
    logs.sort_by(|a, b| {
        let modified_diff = b.modified.cmp(&a.modified);
        if modified_diff != std::cmp::Ordering::Equal {
            return modified_diff;
        }
        b.created.cmp(&a.created)
    });
}
