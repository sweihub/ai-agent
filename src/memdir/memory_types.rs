// Source: /data/home/swei/claudecode/openclaudecode/src/utils/filePersistence/types.ts
//! Memory type taxonomy and structures.
//!
//! Memories are constrained to four types capturing context NOT derivable
//! from the current project state. Code patterns, architecture, git history,
//! and file structure are derivable and should NOT be saved as memories.

use serde::{Deserialize, Serialize};

/// Memory types supported by the memory system
pub const MEMORY_TYPES: &[&str] = &["user", "feedback", "project", "reference"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryType {
    User,
    Feedback,
    Project,
    Reference,
}

impl MemoryType {
    /// Parse a string into a MemoryType
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "user" => Some(Self::User),
            "feedback" => Some(Self::Feedback),
            "project" => Some(Self::Project),
            "reference" => Some(Self::Reference),
            _ => None,
        }
    }

    /// Get the type name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Feedback => "feedback",
            Self::Project => "project",
            Self::Reference => "reference",
        }
    }
}

/// Parse a raw frontmatter value into a MemoryType.
/// Invalid or missing values return None — legacy files without a
/// `type:` field keep working, files with unknown types degrade gracefully.
pub fn parse_memory_type(raw: impl AsRef<str>) -> Option<MemoryType> {
    MemoryType::from_str(raw.as_ref())
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// `## Types of memory` section for COMBINED mode (private + team directories).
/// Includes <scope> tags and team/private qualifiers in examples.
pub const TYPES_SECTION_COMBINED: &[&str] = &[
    "## Types of memory",
    "",
    "There are several discrete types of memory that you can store in your memory system. Each type below declares a <scope> of `private`, `team`, or guidance for choosing between the two.",
    "",
    "<types>",
    "<type>",
    "    <name>user</name>",
    "    <scope>always private</scope>",
    "    <description>Contain information about the user's role, goals, responsibilities, and knowledge. Great user memories help you tailor your future behavior to the user's preferences and perspective. Your goal in reading and writing these memories is to build up an understanding of who the user is and how you can be most helpful to them specifically.</description>",
    "    <when_to_save>When you learn any details about the user's role, preferences, responsibilities, or knowledge</when_to_save>",
    "    <how_to_use>When your work should be informed by the user's profile or perspective.</how_to_use>",
    "</type>",
    "<type>",
    "    <name>feedback</name>",
    "    <scope>default to private. Save as team only when the guidance is clearly a project-wide convention that every contributor should follow (e.g., a testing policy, a build invariant), not a personal style preference.</scope>",
    "    <description>Guidance the user has given you about how to approach work — both what to avoid and what to keep doing.</description>",
    "    <when_to_save>Any time the user corrects your approach (\"no not that\", \"don't\", \"stop doing X\") OR confirms a non-obvious approach worked.</when_to_save>",
    "    <how_to_use>Let these memories guide your behavior so that the user and other users in the project do not need to offer the same guidance twice.</how_to_use>",
    "</type>",
    "<type>",
    "    <name>project</name>",
    "    <scope>private or team, but strongly bias toward team</scope>",
    "    <description>Information that you learn about ongoing work, goals, initiatives, bugs, or incidents within the project that is not otherwise derivable from the code or git history.</description>",
    "    <when_to_save>When you learn who is doing what, why, or by when.</when_to_save>",
    "    <how_to_use>Use these memories to more fully understand the details and nuance behind the user's request.</how_to_use>",
    "</type>",
    "<type>",
    "    <name>reference</name>",
    "    <scope>usually team</scope>",
    "    <description>Stores pointers to where information can be found in external systems.</description>",
    "    <when_to_save>When you learn about resources in external systems and their purpose.</when_to_save>",
    "    <how_to_use>When the user references an external system or information that may be in an external system.</how_to_use>",
    "</type>",
    "</types>",
    "",
];

/// Recall-side drift caveat. Single bullet under `## When to access memories`.
pub const MEMORY_DRIFT_CAVEAT: &str =
    "- Memory records can become stale over time. Use memory as context for what was true at a given point in time. Before answering the user or building assumptions based solely on information in memory records, verify that the memory is still correct and up-to-date by reading the current state of the files or resources. If a recalled memory conflicts with current information, trust what you observe now — and update or remove the stale memory rather than acting on it.";

/// `## What NOT to save in memory` section.
pub const WHAT_NOT_TO_SAVE_SECTION: &[&str] = &[
    "## What NOT to save in memory",
    "",
    "- Code patterns, conventions, architecture, file paths, or project structure — these can be derived by reading the current project state.",
    "- Git history, recent changes, or who-changed-what — `git log` / `git blame` are authoritative.",
    "- Debugging solutions or fix recipes — the fix is in the code; the commit message has the context.",
    "- Anything already documented in AI.md files.",
    "- Ephemeral task details: in-progress work, temporary state, current conversation context.",
    "",
    "These exclusions apply even when the user explicitly asks you to save. If they ask you to save a PR list or activity summary, ask what was *surprising* or *non-obvious* about it — that is the part worth keeping.",
];

/// `## Before recommending from memory` section.
pub const TRUSTING_RECALL_SECTION: &[&str] = &[
    "## Before recommending from memory",
    "",
    "A memory that names a specific function, file, or flag is a claim that it existed *when the memory was written*. It may have been renamed, removed, or never merged. Before recommending it:",
    "",
    "- If the memory names a file path: check the file exists.",
    "- If the memory names a function or flag: grep for it.",
    "- If the user is about to act on your recommendation (not just asking about history), verify first.",
    "",
    "\"The memory says X exists\" is not the same as \"X exists now.\"",
    "",
    "A memory that summarizes repo state (activity logs, architecture snapshots) is frozen in time. If the user asks about *recent* or *current* state, prefer `git log` or reading the code over recalling the snapshot.",
];

/// A single memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub memory_type: MemoryType,
    pub content: String,
}

/// Frontmatter for memory files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub memory_type: MemoryType,
}

/// Parse frontmatter from markdown content
pub fn parse_frontmatter(content: &str) -> Option<MemoryFrontmatter> {
    let trimmed = content.trim();

    // Check for frontmatter delimiters
    if !trimmed.starts_with("---") {
        return None;
    }

    // Find the closing delimiter
    let end_idx = trimmed[3..].find("---")? + 3;
    let frontmatter = &trimmed[3..end_idx];

    let mut name = String::new();
    let mut description = String::new();
    let mut memory_type = MemoryType::User; // Default

    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "name" => name = value.to_string(),
                "description" => description = value.to_string(),
                "type" => {
                    if let Some(t) = MemoryType::from_str(value) {
                        memory_type = t;
                    }
                }
                _ => {}
            }
        }
    }

    if name.is_empty() {
        return None;
    }

    Some(MemoryFrontmatter {
        name,
        description,
        memory_type,
    })
}

/// Extract content after frontmatter
pub fn extract_content(content: &str) -> String {
    let trimmed = content.trim();

    if !trimmed.starts_with("---") {
        return trimmed.to_string();
    }

    if let Some(end_idx) = trimmed[3..].find("---") {
        let after_frontmatter = &trimmed[3 + end_idx + 3..];
        after_frontmatter.trim().to_string()
    } else {
        trimmed.to_string()
    }
}

/// Entrypoint truncation result
#[derive(Debug, Clone)]
pub struct EntrypointTruncation {
    pub content: String,
    pub line_count: usize,
    pub byte_count: usize,
    pub was_line_truncated: bool,
    pub was_byte_truncated: bool,
}

/// Maximum lines in MEMORY.md entrypoint
pub const MAX_ENTRYPOINT_LINES: usize = 200;
/// Maximum bytes in MEMORY.md entrypoint (~125 chars/line * 200 lines)
pub const MAX_ENTRYPOINT_BYTES: usize = 25_000;

/// Truncate MEMORY.md content to line and byte caps
pub fn truncate_entrypoint(raw: &str) -> EntrypointTruncation {
    let trimmed = raw.trim();
    let content_lines: Vec<&str> = trimmed.lines().collect();
    let line_count = content_lines.len();
    let byte_count = trimmed.len();

    let was_line_truncated = line_count > MAX_ENTRYPOINT_LINES;
    let was_byte_truncated = byte_count > MAX_ENTRYPOINT_BYTES;

    if !was_line_truncated && !byte_count <= MAX_ENTRYPOINT_BYTES {
        return EntrypointTruncation {
            content: trimmed.to_string(),
            line_count,
            byte_count,
            was_line_truncated: false,
            was_byte_truncated: false,
        };
    }

    let truncated = if was_line_truncated {
        content_lines[..MAX_ENTRYPOINT_LINES].join("\n")
    } else {
        trimmed.to_string()
    };

    let truncated = if truncated.len() > MAX_ENTRYPOINT_BYTES {
        if let Some(cut_at) = truncated.rfind('\n') {
            if cut_at > MAX_ENTRYPOINT_BYTES {
                truncated[..cut_at].to_string()
            } else {
                truncated[..MAX_ENTRYPOINT_BYTES].to_string()
            }
        } else {
            truncated[..MAX_ENTRYPOINT_BYTES].to_string()
        }
    } else {
        truncated
    };

    let reason = if was_byte_truncated && !was_line_truncated {
        format!("{} (limit: {} bytes)", byte_count, MAX_ENTRYPOINT_BYTES)
    } else if was_line_truncated && !was_byte_truncated {
        format!("{} lines (limit: {})", line_count, MAX_ENTRYPOINT_LINES)
    } else {
        format!("{} lines and {} bytes", line_count, byte_count)
    };

    let content = format!(
        "{}\n\n> WARNING: MEMORY.md is {}. Only part of it was loaded. Keep index entries to one line under ~200 chars; move detail into topic files.",
        truncated, reason
    );

    EntrypointTruncation {
        content,
        line_count,
        byte_count,
        was_line_truncated,
        was_byte_truncated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_type_from_str() {
        assert_eq!(MemoryType::from_str("user"), Some(MemoryType::User));
        assert_eq!(MemoryType::from_str("feedback"), Some(MemoryType::Feedback));
        assert_eq!(MemoryType::from_str("project"), Some(MemoryType::Project));
        assert_eq!(
            MemoryType::from_str("reference"),
            Some(MemoryType::Reference)
        );
        assert_eq!(MemoryType::from_str("unknown"), None);
    }

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
name: test_memory
description: A test memory
type: user
---

This is the content."#;

        let fm = parse_frontmatter(content).unwrap();
        assert_eq!(fm.name, "test_memory");
        assert_eq!(fm.description, "A test memory");
        assert_eq!(fm.memory_type, MemoryType::User);
    }

    #[test]
    fn test_extract_content() {
        let content = r#"---
name: test
description: test
type: user
---

This is the actual content."#;

        let extracted = extract_content(content);
        assert_eq!(extracted, "This is the actual content.");
    }
}
