use crate::tool::{Tool, ToolResultRenderOptions};
use crate::types::*;
use crate::utils::diff::{self, StructuredPatchHunk};
use std::fs;

pub const FILE_EDIT_TOOL_NAME: &str = "Edit";
pub const AI_FOLDER_PERMISSION_PATTERN: &str = "/.ai/**";
pub const GLOBAL_AI_FOLDER_PERMISSION_PATTERN: &str = "~/.ai/**";
pub const FILE_UNEXPECTEDLY_MODIFIED_ERROR: &str =
    "File has been unexpectedly modified. Read it again before attempting to write it.";

/// Result of a file edit operation, returned as JSON in ToolResult.content
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileEditResult {
    pub file_path: String,
    pub old_string: String,
    pub new_string: String,
    pub original_file: String,
    pub structured_patch: Vec<StructuredPatchHunk>,
    #[serde(default)]
    pub replace_all: bool,
    /// Count of lines added
    #[serde(default)]
    pub additions: usize,
    /// Count of lines removed
    #[serde(default)]
    pub removals: usize,
}

pub struct FileEditTool;

impl FileEditTool {
    pub fn new() -> Self {
        Self
    }

    pub fn name(&self) -> &str {
        "FileEdit"
    }

    pub fn description(&self) -> &str {
        "Edit files by performing exact string replacements"
    }

    pub fn input_schema(&self) -> ToolInputSchema {
        ToolInputSchema {
            schema_type: "object".to_string(),
            properties: serde_json::json!({
                "file_path": {
                    "type": "string",
                    "description": "The absolute path to the file to modify"
                },
                "old_string": {
                    "type": "string",
                    "description": "The exact text to find and replace"
                },
                "new_string": {
                    "type": "string",
                    "description": "The replacement text"
                },
                "replace_all": {
                    "type": "boolean",
                    "description": "Replace all occurrences (default false)"
                }
            }),
            required: Some(vec![
                "file_path".to_string(),
                "old_string".to_string(),
                "new_string".to_string(),
            ]),
        }
    }

    pub async fn execute(
        &self,
        input: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolResult, crate::error::AgentError> {
        let file_path = input["file_path"]
            .as_str()
            .ok_or_else(|| crate::error::AgentError::Tool("file_path is required".to_string()))?;

        let old_string = input["old_string"]
            .as_str()
            .ok_or_else(|| crate::error::AgentError::Tool("old_string is required".to_string()))?;

        let new_string = input["new_string"]
            .as_str()
            .ok_or_else(|| crate::error::AgentError::Tool("new_string is required".to_string()))?;

        let replace_all = input["replace_all"].as_bool().unwrap_or(false);

        // Resolve relative paths using cwd from context
        let file_path = if std::path::Path::new(file_path).is_relative() {
            std::path::Path::new(&context.cwd).join(file_path)
        } else {
            std::path::PathBuf::from(file_path)
        };
        let file_path_buf = file_path.clone();

        // Read original content
        let content =
            fs::read_to_string(&file_path).map_err(|e| crate::error::AgentError::Io(e))?;

        // Handle empty old_string as a create/insert operation
        let new_content = if old_string.is_empty() {
            // Prepend new_string to existing content
            format!("{}\n{}", new_string, content)
        } else {
            if old_string == new_string {
                return Ok(ToolResult {
                    result_type: "text".to_string(),
                    tool_use_id: "".to_string(),
                    content: "Error: old_string and new_string are identical".to_string(),
                    is_error: Some(true),
                    was_persisted: None,
                });
            }

            if !content.contains(old_string) {
                return Ok(ToolResult {
                    result_type: "text".to_string(),
                    tool_use_id: "".to_string(),
                    content: format!(
                        "Error: old_string not found in {}. Make sure it matches exactly including whitespace.",
                        file_path.display()
                    ),
                    is_error: Some(true),
                    was_persisted: None,
                });
            }

            if replace_all {
                content.replace(old_string, new_string)
            } else {
                // Check uniqueness
                let count = content.matches(old_string).count();
                if count > 1 {
                    return Ok(ToolResult {
                        result_type: "text".to_string(),
                        tool_use_id: "".to_string(),
                        content: format!(
                            "Error: old_string appears {} times in the file. Provide more context to make it unique, or set replace_all: true.",
                            count
                        ),
                        is_error: Some(true),
                        was_persisted: None,
                    });
                }
                content.replacen(old_string, new_string, 1)
            }
        };

        fs::write(&file_path_buf, &new_content).map_err(|e| crate::error::AgentError::Io(e))?;

        // Generate structured patch for the result
        let patch = diff::generate_patch(&content, &new_content);
        let (additions, removals) = diff::count_lines_changed(&patch, Some(&new_content));

        let result = FileEditResult {
            file_path: file_path_buf.to_string_lossy().to_string(),
            old_string: old_string.to_string(),
            new_string: new_string.to_string(),
            original_file: content,
            structured_patch: patch,
            replace_all,
            additions,
            removals,
        };

        let content_json = serde_json::to_string(&result).map_err(|e| {
            crate::error::AgentError::Tool(format!("Failed to serialize result: {}", e))
        })?;

        Ok(ToolResult {
            result_type: "text".to_string(),
            tool_use_id: "".to_string(),
            content: content_json,
            is_error: None,
            was_persisted: None,
        })
    }

    /// Returns the user-facing name for this tool based on input.
    /// Returns "Update" for edits, "Create" for new files.
    pub fn user_facing_name(&self, input: Option<&serde_json::Value>) -> String {
        match input {
            Some(inp) => {
                let old_string = inp["old_string"].as_str().unwrap_or("");
                if old_string.is_empty() {
                    "Create".to_string()
                } else {
                    "Update".to_string()
                }
            }
            None => "Edit".to_string(),
        }
    }

    /// Returns a short summary for compact views.
    pub fn get_tool_use_summary(&self, input: Option<&serde_json::Value>) -> Option<String> {
        input.and_then(|inp| inp["file_path"].as_str().map(|s| s.to_string()))
    }

    /// Renders the tool result for display.
    pub fn render_tool_result_message(&self, content: &serde_json::Value) -> Option<String> {
        let result: FileEditResult = serde_json::from_value(content.clone()).ok()?;

        // For plan files, show a hint
        let file_path = &result.file_path;
        if file_path.contains("/.ai/plans/") || file_path.contains("/.ai/plan/") {
            return Some(format!("Updated plan: {}", file_path));
        }

        // For new files (create), count all lines as additions
        if result.old_string.is_empty() {
            return Some(format!("Added {} lines in {}", result.additions, file_path));
        }

        // For edits, show additions/removals
        if result.removals == 0 && result.additions == 0 {
            // No visible changes (maybe whitespace-only)
            return Some(format!("No visible changes to {}", file_path));
        }

        let mut msg = format!(
            "Updated {} ({} {})",
            file_path,
            result.additions,
            if result.additions == 1 {
                "line"
            } else {
                "lines"
            }
        );
        if result.removals > 0 {
            msg.push_str(&format!(
                ", {} {} removed",
                result.removals,
                if result.removals == 1 {
                    "line"
                } else {
                    "lines"
                }
            ));
        }
        Some(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_edit_tool_name() {
        let tool = FileEditTool::new();
        assert_eq!(tool.name(), "FileEdit");
    }

    #[test]
    fn test_user_facing_name_edit() {
        let tool = FileEditTool::new();
        let input = serde_json::json!({
            "file_path": "/test.txt",
            "old_string": "old",
            "new_string": "new"
        });
        assert_eq!(tool.user_facing_name(Some(&input)), "Update");
    }

    #[test]
    fn test_user_facing_name_create() {
        let tool = FileEditTool::new();
        let input = serde_json::json!({
            "file_path": "/test.txt",
            "old_string": "",
            "new_string": "new content"
        });
        assert_eq!(tool.user_facing_name(Some(&input)), "Create");
    }

    #[test]
    fn test_user_facing_name_no_input() {
        let tool = FileEditTool::new();
        assert_eq!(tool.user_facing_name(None), "Edit");
    }

    #[test]
    fn test_get_tool_use_summary() {
        let tool = FileEditTool::new();
        let input = serde_json::json!({
            "file_path": "/path/to/file.rs",
            "old_string": "test",
            "new_string": "value"
        });
        assert_eq!(
            tool.get_tool_use_summary(Some(&input)),
            Some("/path/to/file.rs".to_string())
        );
    }

    #[test]
    fn test_get_tool_use_summary_no_path() {
        let tool = FileEditTool::new();
        let input = serde_json::json!({
            "old_string": "test",
            "new_string": "value"
        });
        assert_eq!(tool.get_tool_use_summary(Some(&input)), None);
    }

    #[test]
    fn test_render_tool_result_message_edit() {
        let tool = FileEditTool::new();
        let result = FileEditResult {
            file_path: "/test.txt".to_string(),
            old_string: "old".to_string(),
            new_string: "new".to_string(),
            original_file: "old\nline2".to_string(),
            structured_patch: vec![],
            replace_all: false,
            additions: 1,
            removals: 1,
        };
        let rendered = tool.render_tool_result_message(&serde_json::json!(result));
        assert!(rendered.is_some());
        let msg = rendered.unwrap();
        assert!(msg.contains("Updated"));
        assert!(msg.contains("1 line"));
    }

    #[test]
    fn test_render_tool_result_message_create() {
        let tool = FileEditTool::new();
        let result = FileEditResult {
            file_path: "/new.txt".to_string(),
            old_string: "".to_string(),
            new_string: "new content".to_string(),
            original_file: "".to_string(),
            structured_patch: vec![],
            replace_all: false,
            additions: 3,
            removals: 0,
        };
        let rendered = tool.render_tool_result_message(&serde_json::json!(result));
        assert!(rendered.is_some());
        assert!(rendered.unwrap().contains("Added 3 lines"));
    }
}
