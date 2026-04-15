// Source: ~/claudecode/openclaudecode/src/utils/collapseReadSearch.ts
//! Collapse consecutive Read/Search operations into summary groups.
//!
//! Rules:
//! - Groups consecutive search/read tool uses (Grep, Glob, Read, and Bash search/read commands)
//! - Includes their corresponding tool results in the group
//! - Breaks groups when assistant text appears

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Tool name constants
const BASH_TOOL_NAME: &str = "Bash";
const READ_TOOL_NAME: &str = "Read";
const GREP_TOOL_NAME: &str = "Grep";
const GLOB_TOOL_NAME: &str = "Glob";
const REPL_TOOL_NAME: &str = "REPL";
const FILE_EDIT_TOOL_NAME: &str = "Edit";
const FILE_WRITE_TOOL_NAME: &str = "Write";
const TOOL_SEARCH_TOOL_NAME: &str = "ToolSearch";
const SNIP_TOOL_NAME: &str = "Snip";

/// Result of checking if a tool use is a search or read operation.
#[derive(Debug, Clone)]
pub struct SearchOrReadResult {
    pub is_collapsible: bool,
    pub is_search: bool,
    pub is_read: bool,
    pub is_list: bool,
    pub is_repl: bool,
    /// True if this is a Write/Edit targeting a memory file
    pub is_memory_write: bool,
    /// True for meta-operations that should be absorbed into a collapse group
    /// without incrementing any count (Snip, ToolSearch).
    pub is_absorbed_silently: bool,
    /// MCP server name when this is an MCP tool
    pub mcp_server_name: Option<String>,
    /// Bash command that is NOT a search/read (under fullscreen mode)
    pub is_bash: Option<bool>,
}

/// Information about a collapsible tool use.
#[derive(Debug, Clone)]
pub struct CollapsibleToolInfo {
    pub name: String,
    pub input: serde_json::Value,
    pub is_search: bool,
    pub is_read: bool,
    pub is_list: bool,
    pub is_repl: bool,
    pub is_memory_write: bool,
    pub is_absorbed_silently: bool,
    pub mcp_server_name: Option<String>,
    pub is_bash: Option<bool>,
}

/// Extract the primary file/directory path from a tool_use input.
/// Handles both `file_path` (Read/Write/Edit) and `path` (Grep/Glob).
fn get_file_path_from_tool_input(input: &serde_json::Value) -> Option<String> {
    input
        .get("file_path")
        .or_else(|| input.get("path"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Check if a search tool use targets memory files by examining its path, pattern, and glob.
fn is_memory_search(tool_input: &serde_json::Value) -> bool {
    if let Some(path) = tool_input.get("path").and_then(|v| v.as_str()) {
        if is_auto_managed_memory_file(path) || is_memory_directory(path) {
            return true;
        }
    }
    if let Some(glob) = tool_input.get("glob").and_then(|v| v.as_str()) {
        if is_auto_managed_memory_pattern(glob) {
            return true;
        }
    }
    if let Some(command) = tool_input.get("command").and_then(|v| v.as_str()) {
        if is_shell_command_targeting_memory(command) {
            return true;
        }
    }
    false
}

/// Check if a Write or Edit tool use targets a memory file and should be collapsed.
fn is_memory_write_or_edit(tool_name: &str, tool_input: &serde_json::Value) -> bool {
    if tool_name != FILE_WRITE_TOOL_NAME && tool_name != FILE_EDIT_TOOL_NAME {
        return false;
    }
    get_file_path_from_tool_input(tool_input).is_some()
        && get_file_path_from_tool_input(tool_input)
            .map(|p| is_auto_managed_memory_file(&p))
            .unwrap_or(false)
}

/// ~5 lines x ~60 cols. Generous static cap -- the renderer lets Ink wrap.
const MAX_HINT_CHARS: usize = 300;

/// Format a bash command for the hint. Drops blank lines, collapses runs of
/// inline whitespace, then caps total length.
fn command_as_hint(command: &str) -> String {
    let cleaned: String = "$ "
        + &command
            .lines()
            .map(|l| {
                let trimmed = l.split_whitespace().collect::<Vec<_>>().join(" ");
                trimmed
            })
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

    if cleaned.len() > MAX_HINT_CHARS {
        format!("{}...", &cleaned[..MAX_HINT_CHARS - 1])
    } else {
        cleaned
    }
}

/// Check if an environment variable is truthy.
fn is_env_truthy(key: &str) -> bool {
    std::env::var(key)
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Check if fullscreen mode is enabled.
fn is_fullscreen_env_enabled() -> bool {
    std::env::var("AI_FULLSCREEN")
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// Check if a file path is an auto-managed memory file.
fn is_auto_managed_memory_file(path: &str) -> bool {
    path.contains(".ai/memory") || path.contains(".ai/AI.md")
}

/// Check if a path is a memory directory.
fn is_memory_directory(path: &str) -> bool {
    path.contains(".ai/memory") || path.ends_with(".ai/memory")
}

/// Check if a shell command targets memory paths.
fn is_shell_command_targeting_memory(command: &str) -> bool {
    command.contains(".ai/memory") || command.contains("AI.md")
}

/// Check if a pattern is an auto-managed memory pattern.
fn is_auto_managed_memory_pattern(pattern: &str) -> bool {
    pattern.contains(".ai/memory") || pattern.contains("AI.md")
}

/// Check if feature flag is enabled.
fn is_feature_enabled(feature: &str) -> bool {
    match feature {
        "TEAMMEM" => is_env_truthy("AI_CODE_ENABLE_TEAM_MEMORY"),
        "HISTORY_SNIP" => is_env_truthy("AI_CODE_ENABLE_HISTORY_SNIP"),
        "BASH_CLASSIFIER" => is_env_truthy("AI_CODE_ENABLE_BASH_CLASSIFIER"),
        "TRANSCRIPT_CLASSIFIER" => is_env_truthy("AI_CODE_ENABLE_TRANSCRIPT_CLASSIFIER"),
        _ => false,
    }
}

/// Extract bash comment label from a command.
fn extract_bash_comment_label(command: &str) -> Option<String> {
    command
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("# "))
        .map(String::from)
}

/// Get the display path (simplified version).
fn get_display_path(path: &str) -> String {
    // In a full implementation, this would shorten the path relative to cwd.
    path.to_string()
}

/// Checks if a tool is a search/read operation using the tool's isSearchOrReadCommand method.
/// Also treats Write/Edit of memory files as collapsible.
pub fn get_tool_search_or_read_info(
    tool_name: &str,
    tool_input: &serde_json::Value,
) -> SearchOrReadResult {
    // REPL is absorbed silently
    if tool_name == REPL_TOOL_NAME {
        return SearchOrReadResult {
            is_collapsible: true,
            is_search: false,
            is_read: false,
            is_list: false,
            is_repl: true,
            is_memory_write: false,
            is_absorbed_silently: true,
            mcp_server_name: None,
            is_bash: None,
        };
    }

    // Memory file writes/edits are collapsible
    if is_memory_write_or_edit(tool_name, tool_input) {
        return SearchOrReadResult {
            is_collapsible: true,
            is_search: false,
            is_read: false,
            is_list: false,
            is_repl: false,
            is_memory_write: true,
            is_absorbed_silently: false,
            mcp_server_name: None,
            is_bash: None,
        };
    }

    // Meta-operations absorbed silently: Snip and ToolSearch
    if (is_feature_enabled("HISTORY_SNIP") && tool_name == SNIP_TOOL_NAME)
        || (is_fullscreen_env_enabled() && tool_name == TOOL_SEARCH_TOOL_NAME)
    {
        return SearchOrReadResult {
            is_collapsible: true,
            is_search: false,
            is_read: false,
            is_list: false,
            is_repl: false,
            is_memory_write: false,
            is_absorbed_silently: true,
            mcp_server_name: None,
            is_bash: None,
        };
    }

    // For the tool's isSearchOrReadCommand check, we use a simplified approach.
    // In the TS version, this calls tool.isSearchOrReadCommand() on the actual tool.
    // Here we use pattern matching on known tool names.
    let (is_search, is_read, is_list) = match tool_name {
        GREP_TOOL_NAME | "grep" => (true, false, false),
        GLOB_TOOL_NAME | "glob" => (true, false, false),
        READ_TOOL_NAME | "Read" => (false, true, false),
        BASH_TOOL_NAME => {
            // Determine if bash command is a search, read, or list operation
            let command = tool_input
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if is_bash_search_command(command) {
                (true, false, false)
            } else if is_bash_read_command(command) {
                (false, true, false)
            } else if is_bash_list_command(command) {
                (false, false, true)
            } else {
                (false, false, false)
            }
        }
        _ => (false, false, false),
    };

    let is_collapsible = is_search || is_read || is_list;

    // Under fullscreen mode, non-search/read Bash commands are also collapsible
    SearchOrReadResult {
        is_collapsible: is_collapsible
            || (is_fullscreen_env_enabled() && tool_name == BASH_TOOL_NAME),
        is_search,
        is_read,
        is_list,
        is_repl: false,
        is_memory_write: false,
        is_absorbed_silently: false,
        mcp_server_name: None,
        is_bash: if is_fullscreen_env_enabled() {
            Some(!is_collapsible && tool_name == BASH_TOOL_NAME)
        } else {
            None
        },
    }
}

/// Check if a bash command is a search command.
fn is_bash_search_command(command: &str) -> bool {
    let cmd = command.trim_start();
    cmd.starts_with("grep ")
        || cmd.starts_with("rg ")
        || cmd.starts_with("ag ")
        || cmd.starts_with("ack ")
        || cmd.starts_with("find ")
        || cmd.starts_with("ugrep ")
}

/// Check if a bash command is a read command.
fn is_bash_read_command(command: &str) -> bool {
    let cmd = command.trim_start();
    cmd.starts_with("cat ")
        || cmd.starts_with("head ")
        || cmd.starts_with("tail ")
        || cmd.starts_with("less ")
        || cmd.starts_with("more ")
        || cmd.starts_with("wc ")
}

/// Check if a bash command is a list/directory command.
fn is_bash_list_command(command: &str) -> bool {
    let cmd = command.trim_start();
    cmd.starts_with("ls ") || cmd.starts_with("tree ") || cmd.starts_with("du ")
}

/// Check if a tool_use content block is a search/read operation.
pub fn get_search_or_read_from_content(
    content: Option<&serde_json::Value>,
) -> Option<CollapsibleToolInfo> {
    let content = content?;
    if content.get("type").and_then(|v| v.as_str()) != Some("tool_use") {
        return None;
    }
    let name = content.get("name").and_then(|v| v.as_str())?;
    let input = content.get("input").cloned().unwrap_or_default();
    let info = get_tool_search_or_read_info(name, &input);
    if info.is_collapsible || info.is_repl {
        Some(CollapsibleToolInfo {
            name: name.to_string(),
            input,
            is_search: info.is_search,
            is_read: info.is_read,
            is_list: info.is_list,
            is_repl: info.is_repl,
            is_memory_write: info.is_memory_write,
            is_absorbed_silently: info.is_absorbed_silently,
            mcp_server_name: info.mcp_server_name,
            is_bash: info.is_bash,
        })
    } else {
        None
    }
}

/// Checks if a tool is a search/read operation (for backwards compatibility).
fn is_tool_search_or_read(tool_name: &str, tool_input: &serde_json::Value) -> bool {
    get_tool_search_or_read_info(tool_name, tool_input).is_collapsible
}

/// Get all tool use IDs from a message.
fn get_tool_use_ids_from_message(msg: &serde_json::Value) -> Vec<String> {
    // In a full implementation, this would extract IDs from the message structure.
    // For now, return a placeholder.
    msg.get("tool_use_id")
        .and_then(|v| v.as_str())
        .map(|id| vec![id.to_string()])
        .unwrap_or_default()
}

/// Get file paths from a read message.
fn get_file_paths_from_read_message(msg: &serde_json::Value) -> Vec<String> {
    let mut paths = Vec::new();

    if let Some(input) = msg.get("input") {
        if let Some(file_path) = input.get("file_path").and_then(|v| v.as_str()) {
            paths.push(file_path.to_string());
        }
    }

    paths
}

/// Accumulator for building a collapsed group.
struct GroupAccumulator {
    messages: Vec<serde_json::Value>,
    search_count: usize,
    read_file_paths: HashSet<String>,
    read_operation_count: usize,
    list_count: usize,
    tool_use_ids: HashSet<String>,
    memory_search_count: usize,
    memory_read_file_paths: HashSet<String>,
    memory_write_count: usize,
    non_mem_search_args: Vec<String>,
    latest_display_hint: Option<String>,
    hook_total_ms: u64,
    hook_count: usize,
    hook_infos: Vec<serde_json::Value>,
    // Fullscreen-specific fields
    bash_count: usize,
    bash_commands: HashMap<String, String>,
    commits: Vec<serde_json::Value>,
    pushes: Vec<serde_json::Value>,
    branches: Vec<serde_json::Value>,
    prs: Vec<serde_json::Value>,
    git_op_bash_count: usize,
    // MCP-specific fields
    mcp_call_count: usize,
    mcp_server_names: HashSet<String>,
    // Memory-specific fields
    team_memory_search_count: usize,
    team_memory_read_file_paths: HashSet<String>,
    team_memory_write_count: usize,
}

fn create_empty_group() -> GroupAccumulator {
    GroupAccumulator {
        messages: Vec::new(),
        search_count: 0,
        read_file_paths: HashSet::new(),
        read_operation_count: 0,
        list_count: 0,
        tool_use_ids: HashSet::new(),
        memory_search_count: 0,
        memory_read_file_paths: HashSet::new(),
        memory_write_count: 0,
        non_mem_search_args: Vec::new(),
        latest_display_hint: None,
        hook_total_ms: 0,
        hook_count: 0,
        hook_infos: Vec::new(),
        bash_count: 0,
        bash_commands: HashMap::new(),
        commits: Vec::new(),
        pushes: Vec::new(),
        branches: Vec::new(),
        prs: Vec::new(),
        git_op_bash_count: 0,
        mcp_call_count: 0,
        mcp_server_names: HashSet::new(),
        team_memory_search_count: 0,
        team_memory_read_file_paths: HashSet::new(),
        team_memory_write_count: 0,
    }
}

/// Collapse consecutive Read/Search operations into summary groups.
pub fn collapse_read_search_groups(messages: &[serde_json::Value]) -> Vec<serde_json::Value> {
    let mut result = Vec::new();
    let mut current_group = create_empty_group();
    let mut deferred_skippable: Vec<serde_json::Value> = Vec::new();

    fn flush_group(
        result: &mut Vec<serde_json::Value>,
        current_group: &mut GroupAccumulator,
        deferred_skippable: &mut Vec<serde_json::Value>,
    ) {
        if current_group.messages.is_empty() {
            return;
        }
        result.push(create_collapsed_group(current_group));
        for deferred in deferred_skippable.drain(..) {
            result.push(deferred);
        }
        *current_group = create_empty_group();
    }

    for msg in messages {
        let msg_type = msg.get("type").and_then(|v| v.as_str()).unwrap_or("");

        if msg_type == "assistant" {
            // Check if this is a collapsible tool use
            let content = msg.get("message").and_then(|m| m.get("content"));
            if let Some(content_arr) = content.and_then(|c| c.as_array()) {
                if let Some(first_content) = content_arr.first() {
                    if first_content.get("type").and_then(|v| v.as_str()) == Some("tool_use") {
                        if let Some(tool_name) = first_content.get("name").and_then(|v| v.as_str())
                        {
                            let input =
                                first_content.get("input").cloned().unwrap_or_default();
                            let info = get_tool_search_or_read_info(tool_name, &input);

                            if info.is_collapsible {
                                process_collapsible_tool_use(
                                    &info,
                                    tool_name,
                                    &input,
                                    msg,
                                    &mut current_group,
                                    &mut deferred_skippable,
                                    &mut result,
                                    &mut flush_group,
                                );
                                continue;
                            }
                        }
                    }
                }
            }
        }

        // If we get here, this message breaks the group
        flush_group(
            &mut result,
            &mut current_group,
            &mut deferred_skippable,
        );
        result.push(msg.clone());
    }

    flush_group(
        &mut result,
        &mut current_group,
        &mut deferred_skippable,
    );
    result
}

/// Process a collapsible tool use message.
fn process_collapsible_tool_use(
    info: &SearchOrReadResult,
    tool_name: &str,
    tool_input: &serde_json::Value,
    msg: &serde_json::Value,
    current_group: &mut GroupAccumulator,
    deferred_skippable: &mut Vec<serde_json::Value>,
    result: &mut Vec<serde_json::Value>,
    flush_fn: &mut impl FnMut(
        &mut Vec<serde_json::Value>,
        &mut GroupAccumulator,
        &mut Vec<serde_json::Value>,
    ),
) {
    if info.is_memory_write {
        // Memory file write/edit
        if is_feature_enabled("TEAMMEM") && is_team_memory_write_or_edit(tool_name, tool_input) {
            current_group.team_memory_write_count += 1;
        } else {
            current_group.memory_write_count += 1;
        }
    } else if info.is_absorbed_silently {
        // Snip/ToolSearch absorbed silently
    } else if let Some(ref mcp_server) = info.mcp_server_name {
        // MCP search/read
        current_group.mcp_call_count += 1;
        current_group.mcp_server_names.insert(mcp_server.clone());
        if let Some(query) = tool_input.get("query").and_then(|v| v.as_str()) {
            current_group.latest_display_hint = Some(format!("\"{query}\""));
        }
    } else if is_fullscreen_env_enabled() && info.is_bash == Some(true) {
        // Non-search/read Bash command
        current_group.bash_count += 1;
        if let Some(command) = tool_input.get("command").and_then(|v| v.as_str()) {
            current_group.latest_display_hint =
                Some(extract_bash_comment_label(command).unwrap_or_else(|| command_as_hint(command)));
            for id in get_tool_use_ids_from_message(msg) {
                current_group
                    .bash_commands
                    .insert(id, command.to_string());
            }
        }
    } else if info.is_list {
        current_group.list_count += 1;
        if let Some(command) = tool_input.get("command").and_then(|v| v.as_str()) {
            current_group.latest_display_hint = Some(command_as_hint(command));
        }
    } else if info.is_search {
        current_group.search_count += 1;
        if is_feature_enabled("TEAMMEM") && is_team_memory_search(tool_input) {
            current_group.team_memory_search_count += 1;
        } else if is_memory_search(tool_input) {
            current_group.memory_search_count += 1;
        } else {
            if let Some(pattern) = tool_input.get("pattern").and_then(|v| v.as_str()) {
                current_group.non_mem_search_args.push(pattern.to_string());
                current_group.latest_display_hint = Some(format!("\"{pattern}\""));
            }
        }
    } else {
        // For reads, track unique file paths
        let file_paths = get_file_paths_from_read_message(msg);
        for file_path in &file_paths {
            current_group.read_file_paths.insert(file_path.clone());
            if is_feature_enabled("TEAMMEM") && is_team_mem_file(file_path) {
                current_group
                    .team_memory_read_file_paths
                    .insert(file_path.clone());
            } else if is_auto_managed_memory_file(file_path) {
                current_group
                    .memory_read_file_paths
                    .insert(file_path.clone());
            } else {
                current_group.latest_display_hint = Some(get_display_path(file_path));
            }
        }
        if file_paths.is_empty() {
            current_group.read_operation_count += 1;
            if let Some(command) = tool_input.get("command").and_then(|v| v.as_str()) {
                current_group.latest_display_hint = Some(command_as_hint(command));
            }
        }
    }

    // Track tool use IDs
    for id in get_tool_use_ids_from_message(msg) {
        current_group.tool_use_ids.insert(id);
    }

    current_group.messages.push(msg.clone());
}

/// Team memory stubs
fn is_team_memory_write_or_edit(_tool_name: &str, _tool_input: &serde_json::Value) -> bool {
    false
}

fn is_team_memory_search(_tool_input: &serde_json::Value) -> bool {
    false
}

fn is_team_mem_file(_path: &str) -> bool {
    false
}

/// Create a collapsed group JSON value.
fn create_collapsed_group(group: &GroupAccumulator) -> serde_json::Value {
    let total_read_count = if !group.read_file_paths.is_empty() {
        group.read_file_paths.len()
    } else {
        group.read_operation_count
    };

    serde_json::json!({
        "type": "collapsed_read_search",
        "searchCount": group.search_count.saturating_sub(group.memory_search_count).saturating_sub(group.team_memory_search_count),
        "readCount": total_read_count.saturating_sub(group.memory_read_file_paths.len()),
        "listCount": group.list_count,
        "replCount": 0,
        "memorySearchCount": group.memory_search_count,
        "memoryReadCount": group.memory_read_file_paths.len(),
        "memoryWriteCount": group.memory_write_count,
        "readFilePaths": group.read_file_paths.iter().cloned().collect::<Vec<_>>(),
        "searchArgs": group.non_mem_search_args,
        "latestDisplayHint": group.latest_display_hint,
        "messages": group.messages,
    })
}

/// Generate a summary text for search/read counts.
pub fn get_search_read_summary_text(
    search_count: usize,
    read_count: usize,
    is_active: bool,
    repl_count: usize,
    memory_counts: Option<MemoryCounts>,
    list_count: usize,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    // Memory operations first
    if let Some(mc) = &memory_counts {
        if mc.memory_read_count > 0 {
            let verb = if is_active {
                if parts.is_empty() {
                    "Recalling"
                } else {
                    "recalling"
                }
            } else if parts.is_empty() {
                "Recalled"
            } else {
                "recalled"
            };
            let noun = if mc.memory_read_count == 1 {
                "memory"
            } else {
                "memories"
            };
            parts.push(format!("{verb} {} {noun}", mc.memory_read_count));
        }
        if mc.memory_search_count > 0 {
            let verb = if is_active {
                if parts.is_empty() { "Searching" } else { "searching" }
            } else if parts.is_empty() {
                "Searched"
            } else {
                "searched"
            };
            parts.push(format!("{verb} memories"));
        }
        if mc.memory_write_count > 0 {
            let verb = if is_active {
                if parts.is_empty() { "Writing" } else { "writing" }
            } else if parts.is_empty() {
                "Wrote"
            } else {
                "wrote"
            };
            let noun = if mc.memory_write_count == 1 {
                "memory"
            } else {
                "memories"
            };
            parts.push(format!("{verb} {} {noun}", mc.memory_write_count));
        }
    }

    if search_count > 0 {
        let search_verb = if is_active {
            if parts.is_empty() {
                "Searching for"
            } else {
                "searching for"
            }
        } else if parts.is_empty() {
            "Searched for"
        } else {
            "searched for"
        };
        let pattern = if search_count == 1 {
            "pattern"
        } else {
            "patterns"
        };
        parts.push(format!("{search_verb} {search_count} {pattern}"));
    }

    if read_count > 0 {
        let read_verb = if is_active {
            if parts.is_empty() { "Reading" } else { "reading" }
        } else if parts.is_empty() {
            "Read"
        } else {
            "read"
        };
        let file = if read_count == 1 { "file" } else { "files" };
        parts.push(format!("{read_verb} {read_count} {file}"));
    }

    if list_count > 0 {
        let list_verb = if is_active {
            if parts.is_empty() { "Listing" } else { "listing" }
        } else if parts.is_empty() {
            "Listed"
        } else {
            "listed"
        };
        let dir = if list_count == 1 {
            "directory"
        } else {
            "directories"
        };
        parts.push(format!("{list_verb} {list_count} {dir}"));
    }

    if repl_count > 0 {
        let repl_verb = if is_active { "REPL'ing" } else { "REPL'd" };
        let time = if repl_count == 1 { "time" } else { "times" };
        parts.push(format!("{repl_verb} {repl_count} {time}"));
    }

    let text = parts.join(", ");
    if is_active {
        format!("{text}...")
    } else {
        text
    }
}

/// Memory counts for summary.
#[derive(Debug, Clone)]
pub struct MemoryCounts {
    pub memory_search_count: usize,
    pub memory_read_count: usize,
    pub memory_write_count: usize,
    pub team_memory_search_count: usize,
    pub team_memory_read_count: usize,
    pub team_memory_write_count: usize,
}

/// Summarize recent activities into a compact description.
pub fn summarize_recent_activities(
    activities: &[ActivityDescription],
) -> Option<String> {
    if activities.is_empty() {
        return None;
    }

    // Count trailing search/read activities from the end
    let mut search_count = 0;
    let mut read_count = 0;
    for activity in activities.iter().rev() {
        if activity.is_search {
            search_count += 1;
        } else if activity.is_read {
            read_count += 1;
        } else {
            break;
        }
    }

    let collapsible_count = search_count + read_count;
    if collapsible_count >= 2 {
        return Some(get_search_read_summary_text(
            search_count,
            read_count,
            true,
            0,
            None,
            0,
        ));
    }

    // Fall back to most recent activity with a description
    for activity in activities.iter().rev() {
        if let Some(ref desc) = activity.activity_description {
            return Some(desc.clone());
        }
    }
    None
}

/// Description of an activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDescription {
    #[serde(rename = "activityDescription", skip_serializing_if = "Option::is_none")]
    pub activity_description: Option<String>,
    #[serde(rename = "isSearch", skip_serializing_if = "Option::is_none")]
    pub is_search: Option<bool>,
    #[serde(rename = "isRead", skip_serializing_if = "Option::is_none")]
    pub is_read: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tool_search_or_read_info_repl() {
        let info = get_tool_search_or_read_info(REPL_TOOL_NAME, &serde_json::json!({}));
        assert!(info.is_collapsible);
        assert!(info.is_repl);
        assert!(info.is_absorbed_silently);
    }

    #[test]
    fn test_get_tool_search_or_read_info_grep() {
        let info = get_tool_search_or_read_info(
            GREP_TOOL_NAME,
            &serde_json::json!({"pattern": "foo", "path": "."}),
        );
        assert!(info.is_collapsible);
        assert!(info.is_search);
    }

    #[test]
    fn test_get_tool_search_or_read_info_read() {
        let info = get_tool_search_or_read_info(
            READ_TOOL_NAME,
            &serde_json::json!({"file_path": "test.txt"}),
        );
        assert!(info.is_collapsible);
        assert!(info.is_read);
    }

    #[test]
    fn test_command_as_hint() {
        let hint = command_as_hint("ls -la /some/path");
        assert!(hint.starts_with("$ "));
    }

    #[test]
    fn test_command_as_hint_truncation() {
        let long_cmd = "x".repeat(400);
        let hint = command_as_hint(&long_cmd);
        assert!(hint.len() <= MAX_HINT_CHARS);
        assert!(hint.ends_with("..."));
    }

    #[test]
    fn test_get_search_read_summary_text() {
        let summary = get_search_read_summary_text(3, 2, false, 0, None, 0);
        assert!(summary.contains("Searched for 3 patterns"));
        assert!(summary.contains("Read 2 files"));
    }

    #[test]
    fn test_get_search_read_summary_text_active() {
        let summary = get_search_read_summary_text(1, 1, true, 0, None, 0);
        assert!(summary.ends_with("..."));
    }

    #[test]
    fn test_summarize_recent_activities_multiple_searches() {
        let activities = vec![
            ActivityDescription {
                activity_description: Some("Doing something".to_string()),
                is_search: Some(false),
                is_read: Some(false),
            },
            ActivityDescription {
                activity_description: None,
                is_search: Some(true),
                is_read: Some(false),
            },
            ActivityDescription {
                activity_description: None,
                is_search: Some(true),
                is_read: Some(false),
            },
        ];
        let summary = summarize_recent_activities(&activities);
        assert!(summary.is_some());
        let s = summary.unwrap();
        assert!(s.contains("Searching for 2 patterns"));
    }

    #[test]
    fn test_summarize_recent_activities_empty() {
        assert!(summarize_recent_activities(&[]).is_none());
    }
}
