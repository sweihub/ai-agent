//! Query helpers utilities
//!
//! Ported from ~/claudecode/openclaudecode/src/utils/queryHelpers.ts
//! Provides utilities for ripgrep search, file state caching from message history,
//! and bash tool extraction.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// Ripgrep helpers
// ---------------------------------------------------------------------------

/// Parse ripgrep output to extract matched file paths.
///
/// Ripgrep's output with `--files-with-matches` flag returns one file path per line.
pub fn parse_rg_output(output: &str) -> Vec<String> {
    output
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.trim().to_string())
        .collect()
}

/// Search with ripgrep for a pattern in the given path.
///
/// Uses ripgrep's `--files-with-matches` to return matching file paths.
/// Returns an error message if ripgrep is not available or the search fails.
pub fn search_with_rg(pattern: &str, path: &str) -> Result<String, String> {
    let output = Command::new("rg")
        .arg("--files-with-matches")
        .arg("--no-heading")
        .arg("--line-number")
        .arg(pattern)
        .arg(path)
        .output()
        .map_err(|e| format!("Failed to execute ripgrep: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        // Exit code 1 from ripgrep means no matches found (not an error)
        if output.status.code() == Some(1) {
            return Ok(String::new());
        }
        return Err(if !stderr.is_empty() {
            stderr.trim().to_string()
        } else {
            format!(
                "ripgrep exited with code {}",
                output.status.code().unwrap_or(-1)
            )
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ---------------------------------------------------------------------------
// File state cache (from extractReadFilesFromMessages)
// ---------------------------------------------------------------------------

/// Cached state of a file at a point in time.
#[derive(Debug, Clone)]
pub struct FileStateEntry {
    /// Content of the file
    pub content: String,
    /// Timestamp when the content was captured (epoch millis)
    pub timestamp: u64,
    /// Optional offset if this is a ranged read
    pub offset: Option<u64>,
    /// Optional limit if this is a ranged read
    pub limit: Option<u64>,
}

/// Cache of file states extracted from message history.
#[derive(Debug, Clone)]
pub struct FileStateCache {
    entries: lru::LruCache<String, FileStateEntry>,
}

impl FileStateCache {
    /// Create a new file state cache with the given maximum size.
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: lru::LruCache::new(std::num::NonZero::new(max_size).unwrap()),
        }
    }

    /// Insert a file state entry into the cache.
    pub fn set(&mut self, path: impl Into<String>, entry: FileStateEntry) {
        self.entries.put(path.into(), entry);
    }

    /// Get a file state entry from the cache.
    pub fn get(&self, path: &str) -> Option<&FileStateEntry> {
        self.entries.peek(path)
    }

    /// Check if the cache contains an entry for the given path.
    pub fn contains(&self, path: &str) -> bool {
        self.entries.contains(path)
    }

    /// Get the number of entries in the cache.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// FileReadTool input schema.
#[derive(Debug, Clone)]
struct FileReadInput {
    file_path: Option<String>,
    offset: Option<u64>,
    limit: Option<u64>,
}

impl FileReadInput {
    fn from_value(v: &serde_json::Value) -> Option<Self> {
        Some(FileReadInput {
            file_path: v
                .get("file_path")
                .and_then(|v| v.as_str())
                .map(String::from),
            offset: v.get("offset").and_then(|v| v.as_u64()),
            limit: v.get("limit").and_then(|v| v.as_u64()),
        })
    }
}

/// FileWriteTool input schema.
#[derive(Debug, Clone)]
struct FileWriteInput {
    file_path: Option<String>,
    content: Option<String>,
}

impl FileWriteInput {
    fn from_value(v: &serde_json::Value) -> Option<Self> {
        Some(FileWriteInput {
            file_path: v
                .get("file_path")
                .and_then(|v| v.as_str())
                .map(String::from),
            content: v.get("content").and_then(|v| v.as_str()).map(String::from),
        })
    }
}

/// FileEditTool input schema.
#[derive(Debug, Clone)]
struct FileEditInput {
    file_path: Option<String>,
}

impl FileEditInput {
    fn from_value(v: &serde_json::Value) -> Option<Self> {
        Some(FileEditInput {
            file_path: v
                .get("file_path")
                .and_then(|v| v.as_str())
                .map(String::from),
        })
    }
}

/// Stub text for unchanged files in read tool results.
const FILE_UNCHANGED_STUB: &str = "(file unchanged)";

/// Expand a path to an absolute path, resolving `~` and relative paths.
fn expand_path(path: &str, cwd: &str) -> String {
    let p = if path.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            let rest = path.trim_start_matches("~");
            let rest = rest.trim_start_matches('/');
            home.join(rest)
        } else {
            PathBuf::from(path)
        }
    } else if Path::new(path).is_relative() {
        PathBuf::from(cwd).join(path)
    } else {
        PathBuf::from(path)
    };

    p.to_string_lossy().to_string()
}

/// Strip line number prefix from ripgrep output lines (e.g., "123:content" -> "content").
fn strip_line_number_prefix(line: &str) -> &str {
    if let Some(pos) = line.find(':') {
        if line[..pos].chars().all(|c| c.is_ascii_digit()) {
            return &line[pos + 1..];
        }
    }
    line
}

/// Extract read files from messages and build a file state cache.
///
/// First pass: find all FileReadTool/FileWriteTool/FileEditTool uses in assistant messages.
/// Second pass: find corresponding tool results and extract content.
///
/// # Arguments
/// * `messages` - Message history to extract from
/// * `cwd` - Current working directory for path resolution
/// * `max_size` - Maximum number of entries in the cache
pub fn extract_read_files_from_messages(
    messages: &[serde_json::Value],
    cwd: &str,
    max_size: usize,
) -> FileStateCache {
    let mut cache = FileStateCache::new(max_size);

    // Tool name constants matching the TS source
    const FILE_READ_TOOL_NAME: &str = "Read";
    const FILE_WRITE_TOOL_NAME: &str = "Write";
    const FILE_EDIT_TOOL_NAME: &str = "Edit";

    // First pass: find all FileReadTool/FileWriteTool/FileEditTool uses in assistant messages
    let mut file_read_tool_use_ids: HashMap<String, String> = HashMap::new(); // toolUseId -> filePath
    let mut file_write_tool_use_ids: HashMap<String, (String, String)> = HashMap::new(); // toolUseId -> (filePath, content)
    let mut file_edit_tool_use_ids: HashMap<String, String> = HashMap::new(); // toolUseId -> filePath

    for message in messages {
        if let Some(msg_type) = message.get("type").and_then(|v| v.as_str()) {
            if msg_type == "assistant" {
                if let Some(content) = message.get("message").and_then(|v| v.get("content")) {
                    if let Some(blocks) = content.as_array() {
                        for block in blocks {
                            if let Some(block_type) = block.get("type").and_then(|v| v.as_str()) {
                                if block_type == "tool_use" {
                                    let tool_name =
                                        block.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                    let tool_id =
                                        block.get("id").and_then(|v| v.as_str()).unwrap_or("");
                                    let input = block.get("input");

                                    if let Some(input) = input {
                                        match tool_name {
                                            FILE_READ_TOOL_NAME => {
                                                if let Some(read_input) =
                                                    FileReadInput::from_value(input)
                                                {
                                                    // Ranged reads are not added to the cache
                                                    if let Some(fp) = read_input.file_path {
                                                        if read_input.offset.is_none()
                                                            && read_input.limit.is_none()
                                                        {
                                                            let abs_path = expand_path(&fp, cwd);
                                                            file_read_tool_use_ids.insert(
                                                                tool_id.to_string(),
                                                                abs_path,
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                            FILE_WRITE_TOOL_NAME => {
                                                if let Some(write_input) =
                                                    FileWriteInput::from_value(input)
                                                {
                                                    if let (Some(fp), Some(content)) =
                                                        (write_input.file_path, write_input.content)
                                                    {
                                                        let abs_path = expand_path(&fp, cwd);
                                                        file_write_tool_use_ids.insert(
                                                            tool_id.to_string(),
                                                            (abs_path, content),
                                                        );
                                                    }
                                                }
                                            }
                                            FILE_EDIT_TOOL_NAME => {
                                                if let Some(edit_input) =
                                                    FileEditInput::from_value(input)
                                                {
                                                    if let Some(fp) = edit_input.file_path {
                                                        let abs_path = expand_path(&fp, cwd);
                                                        file_edit_tool_use_ids
                                                            .insert(tool_id.to_string(), abs_path);
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Second pass: find corresponding tool results and extract content
    for message in messages {
        if let Some(msg_type) = message.get("type").and_then(|v| v.as_str()) {
            if msg_type == "user" {
                if let Some(content) = message.get("message").and_then(|v| v.get("content")) {
                    if let Some(blocks) = content.as_array() {
                        for block in blocks {
                            if let Some(block_type) = block.get("type").and_then(|v| v.as_str()) {
                                if block_type == "tool_result" {
                                    let tool_use_id =
                                        block.get("tool_use_id").and_then(|v| v.as_str());

                                    if let Some(tool_use_id) = tool_use_id {
                                        // Handle Read tool results
                                        if let Some(read_file_path) =
                                            file_read_tool_use_ids.get(tool_use_id)
                                        {
                                            if let Some(result_content) =
                                                block.get("content").and_then(|v| v.as_str())
                                            {
                                                // Dedup stubs contain no file content
                                                if !result_content.starts_with(FILE_UNCHANGED_STUB)
                                                {
                                                    // Remove system-reminder blocks using regex
                                                    let re = regex::Regex::new(
                                                        r"<system-reminder>[\s\S]*?</system-reminder>",
                                                    ).ok();
                                                    let processed = if let Some(ref re) = re {
                                                        re.replace_all(result_content, "")
                                                            .to_string()
                                                    } else {
                                                        result_content.to_string()
                                                    };

                                                    // Strip line number prefixes
                                                    let file_content: String = processed
                                                        .lines()
                                                        .map(strip_line_number_prefix)
                                                        .collect::<Vec<_>>()
                                                        .join("\n")
                                                        .trim()
                                                        .to_string();

                                                    // Cache the file content
                                                    let timestamp = message
                                                        .get("timestamp")
                                                        .and_then(|v| v.as_str())
                                                        .and_then(|ts| {
                                                            chrono::DateTime::parse_from_rfc3339(ts)
                                                                .ok()
                                                                .map(|dt| {
                                                                    dt.timestamp_millis() as u64
                                                                })
                                                        })
                                                        .unwrap_or(0);

                                                    cache.set(
                                                        read_file_path.clone(),
                                                        FileStateEntry {
                                                            content: file_content,
                                                            timestamp,
                                                            offset: None,
                                                            limit: None,
                                                        },
                                                    );
                                                }
                                            }
                                        }

                                        // Handle Write tool results
                                        if let Some((file_path, content)) =
                                            file_write_tool_use_ids.get(tool_use_id)
                                        {
                                            let timestamp = message
                                                .get("timestamp")
                                                .and_then(|v| v.as_str())
                                                .and_then(|ts| {
                                                    chrono::DateTime::parse_from_rfc3339(ts)
                                                        .ok()
                                                        .map(|dt| dt.timestamp_millis() as u64)
                                                })
                                                .unwrap_or(0);

                                            cache.set(
                                                file_path.clone(),
                                                FileStateEntry {
                                                    content: content.clone(),
                                                    timestamp,
                                                    offset: None,
                                                    limit: None,
                                                },
                                            );
                                        }

                                        // Handle Edit tool results
                                        if let Some(edit_file_path) =
                                            file_edit_tool_use_ids.get(tool_use_id)
                                        {
                                            let is_error = block
                                                .get("is_error")
                                                .and_then(|v| v.as_bool())
                                                .unwrap_or(false);

                                            if !is_error {
                                                // Read current disk state for edit results
                                                if let Ok(disk_content) =
                                                    fs::read_to_string(edit_file_path)
                                                {
                                                    // Use file mtime as timestamp
                                                    let timestamp = fs::metadata(edit_file_path)
                                                        .ok()
                                                        .and_then(|m| m.modified().ok())
                                                        .and_then(|t| {
                                                            t.duration_since(std::time::UNIX_EPOCH)
                                                                .ok()
                                                                .map(|d| d.as_millis() as u64)
                                                        })
                                                        .unwrap_or(0);

                                                    cache.set(
                                                        edit_file_path.clone(),
                                                        FileStateEntry {
                                                            content: disk_content,
                                                            timestamp,
                                                            offset: None,
                                                            limit: None,
                                                        },
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    cache
}

// ---------------------------------------------------------------------------
// Bash tool extraction (from extractBashToolsFromMessages)
// ---------------------------------------------------------------------------

/// Stripped command prefixes to skip when extracting CLI names.
const STRIPPED_COMMANDS: &[&str] = &["sudo"];

/// Extract the top-level CLI tools used in BashTool calls from message history.
///
/// Returns a deduplicated set of command names (e.g. 'vercel', 'aws', 'git').
/// Skips environment variable assignments and prefixes in STRIPPED_COMMANDS.
pub fn extract_bash_tools_from_messages(messages: &[serde_json::Value]) -> HashSet<String> {
    let mut tools = HashSet::new();

    for message in messages {
        if let Some(msg_type) = message.get("type").and_then(|v| v.as_str()) {
            if msg_type == "assistant" {
                if let Some(content) = message.get("message").and_then(|v| v.get("content")) {
                    if let Some(blocks) = content.as_array() {
                        for block in blocks {
                            if let Some(block_type) = block.get("type").and_then(|v| v.as_str()) {
                                if block_type == "tool_use" {
                                    let tool_name =
                                        block.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                    if tool_name == "Bash" {
                                        if let Some(input) = block.get("input") {
                                            if let Some(command) =
                                                input.get("command").and_then(|v| v.as_str())
                                            {
                                                if let Some(cli_name) = extract_cli_name(command) {
                                                    tools.insert(cli_name);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    tools
}

/// Extract the actual CLI name from a bash command string, skipping
/// env var assignments (e.g. `FOO=bar vercel` -> `vercel`) and prefixes
/// in STRIPPED_COMMANDS.
fn extract_cli_name(command: &str) -> Option<String> {
    let tokens: Vec<&str> = command.trim().split_whitespace().collect();
    for token in tokens {
        // Skip env var assignments
        if token.contains('=')
            && token
                .chars()
                .next()
                .map(|c| c.is_ascii_alphabetic() || c == '_')
                .unwrap_or(false)
        {
            continue;
        }
        // Skip stripped commands
        if STRIPPED_COMMANDS.contains(&token) {
            continue;
        }
        return Some(token.to_string());
    }
    None
}

/// Check if a result should be considered successful based on the last message.
///
/// Returns true if:
/// - Last message is assistant with text/thinking content
/// - Last message is user with only tool_result blocks
/// - Last message is the user prompt but the API completed with end_turn
pub fn is_result_successful(
    message: Option<&serde_json::Value>,
    stop_reason: Option<&str>,
) -> bool {
    let Some(msg) = message else {
        return false;
    };

    if let Some(msg_type) = msg.get("type").and_then(|v| v.as_str()) {
        if msg_type == "assistant" {
            if let Some(content) = msg.get("message").and_then(|v| v.get("content")) {
                if let Some(blocks) = content.as_array() {
                    if let Some(last_block) = blocks.last() {
                        if let Some(block_type) = last_block.get("type").and_then(|v| v.as_str()) {
                            return matches!(block_type, "text" | "thinking" | "redacted_thinking");
                        }
                    }
                }
            }
        }

        if msg_type == "user" {
            if let Some(content) = msg.get("message").and_then(|v| v.get("content")) {
                if let Some(blocks) = content.as_array() {
                    if !blocks.is_empty() {
                        return blocks.iter().all(|block| {
                            block
                                .get("type")
                                .and_then(|v| v.as_str())
                                .map(|t| t == "tool_result")
                                .unwrap_or(false)
                        });
                    }
                }
            }
        }

        // API completed with end_turn but yielded no assistant content
        if stop_reason == Some("end_turn") {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rg_output_empty() {
        assert!(parse_rg_output("").is_empty());
        assert!(parse_rg_output("\n\n").is_empty());
    }

    #[test]
    fn test_parse_rg_output_with_paths() {
        let output = "src/file1.rs\nsrc/file2.rs\n\n";
        let result = parse_rg_output(output);
        assert_eq!(result, vec!["src/file1.rs", "src/file2.rs"]);
    }

    #[test]
    fn test_extract_cli_name_basic() {
        assert_eq!(extract_cli_name("git status"), Some("git".to_string()));
        assert_eq!(extract_cli_name("ls -la"), Some("ls".to_string()));
    }

    #[test]
    fn test_extract_cli_name_env_vars() {
        assert_eq!(
            extract_cli_name("FOO=bar vercel deploy"),
            Some("vercel".to_string())
        );
    }

    #[test]
    fn test_extract_cli_name_sudo() {
        assert_eq!(extract_cli_name("sudo rm -rf /tmp"), Some("rm".to_string()));
    }

    #[test]
    fn test_strip_line_number_prefix() {
        assert_eq!(strip_line_number_prefix("123:hello world"), "hello world");
        assert_eq!(strip_line_number_prefix("hello"), "hello");
        assert_eq!(
            strip_line_number_prefix("abc:not a number prefix"),
            "abc:not a number prefix"
        );
    }

    #[test]
    fn test_expand_path_absolute() {
        let result = expand_path("/absolute/path", "/cwd");
        assert_eq!(result, "/absolute/path");
    }

    #[test]
    fn test_expand_path_relative() {
        let result = expand_path("relative/path", "/cwd");
        assert_eq!(result, "/cwd/relative/path");
    }

    #[test]
    fn test_file_state_cache() {
        let mut cache = FileStateCache::new(5);
        assert!(cache.is_empty());

        cache.set(
            "/test/file.rs",
            FileStateEntry {
                content: "hello".to_string(),
                timestamp: 12345,
                offset: None,
                limit: None,
            },
        );

        assert_eq!(cache.len(), 1);
        assert!(cache.contains("/test/file.rs"));

        let entry = cache.get("/test/file.rs").unwrap();
        assert_eq!(entry.content, "hello");
    }

    #[test]
    fn test_is_result_successful_assistant() {
        let msg = serde_json::json!({
            "type": "assistant",
            "message": { "content": [{ "type": "text", "text": "Hello" }] }
        });
        assert!(is_result_successful(Some(&msg), None));

        let msg2 = serde_json::json!({
            "type": "assistant",
            "message": { "content": [{ "type": "thinking", "text": "..." }] }
        });
        assert!(is_result_successful(Some(&msg2), None));
    }

    #[test]
    fn test_is_result_successful_user_tool_result() {
        let msg = serde_json::json!({
            "type": "user",
            "message": { "content": [{ "type": "tool_result" }] }
        });
        assert!(is_result_successful(Some(&msg), None));
    }

    #[test]
    fn test_is_result_successful_end_turn() {
        let msg = serde_json::json!({
            "type": "user",
            "message": { "content": "prompt" }
        });
        assert!(is_result_successful(Some(&msg), Some("end_turn")));
        assert!(!is_result_successful(Some(&msg), None));
    }

    #[test]
    fn test_is_result_successful_none() {
        assert!(!is_result_successful(None, None));
    }
}
