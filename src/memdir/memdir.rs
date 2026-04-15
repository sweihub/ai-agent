// Source: /data/home/swei/claudecode/openclaudecode/src/memdir/memdir.ts
//! Memory directory management - translated from memdir/memdir.ts
//!
//! Provides the memory system prompt building and management.

use crate::memdir::paths::{
    ensure_memory_dir_exists, get_auto_mem_entrypoint, get_auto_mem_path, is_auto_memory_enabled,
};
use crate::memdir::memory_types::EntrypointTruncation;

/// Entrypoint filename
pub const ENTRYPOINT_NAME: &str = "MEMORY.md";

/// Maximum lines in MEMORY.md entrypoint (~125 chars/line at 200 lines)
pub const MAX_ENTRYPOINT_LINES: usize = 200;

/// Maximum bytes in MEMORY.md entrypoint (~25KB - catches long-line indexes)
pub const MAX_ENTRYPOINT_BYTES: usize = 25_000;

/// Shared guidance text appended to each memory directory prompt line.
/// Shipped because Claude was burning turns on `ls`/`mkdir -p` before writing.
/// Harness guarantees the directory exists via ensure_memory_dir_exists().
pub const DIR_EXISTS_GUIDANCE: &str =
    "This directory already exists — write to it directly with the Write tool (do not run mkdir or check for its existence).";

/// Shared guidance for when both directories exist.
pub const DIRS_EXISTS_GUIDANCE: &str =
    "Both directories already exist — write to them directly with the Write tool (do not run mkdir or check for their existence).";

/// Truncate MEMORY.md content to the line AND byte caps, appending a warning
/// that names which cap fired. Line-truncates first (natural boundary), then
/// byte-truncates at the last newline before the cap so we don't cut mid-line.
pub fn truncate_entrypoint_content(raw: &str) -> EntrypointTruncation {
    let trimmed = raw.trim();
    let content_lines: Vec<&str> = trimmed.lines().collect();
    let line_count = content_lines.len();
    let byte_count = trimmed.len();

    let was_line_truncated = line_count > MAX_ENTRYPOINT_LINES;
    // Check original byte count — long lines are the failure mode the byte cap
    // targets, so post-line-truncation size would understate the warning.
    let was_byte_truncated = byte_count > MAX_ENTRYPOINT_BYTES;

    if !was_line_truncated && !was_byte_truncated {
        return EntrypointTruncation {
            content: trimmed.to_string(),
            line_count,
            byte_count,
            was_line_truncated,
            was_byte_truncated,
        };
    }

    let truncated = if was_line_truncated {
        content_lines[..MAX_ENTRYPOINT_LINES].join("\n")
    } else {
        trimmed.to_string()
    };

    let truncated = if truncated.len() > MAX_ENTRYPOINT_BYTES {
        if let Some(cut_at) = truncated.rfind('\n') {
            if cut_at > 0 {
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
        format!(
            "{} (limit: {} bytes) — index entries are too long",
            format_file_size(byte_count),
            format_file_size(MAX_ENTRYPOINT_BYTES)
        )
    } else if was_line_truncated && !was_byte_truncated {
        format!("{} lines (limit: {})", line_count, MAX_ENTRYPOINT_LINES)
    } else {
        format!("{} lines and {}", line_count, format_file_size(byte_count))
    };

    let content = format!(
        "{}\n\n> WARNING: {} is {}. Only part of it was loaded. Keep index entries to one line under ~200 chars; move detail into topic files.",
        truncated, ENTRYPOINT_NAME, reason
    );

    EntrypointTruncation {
        content,
        line_count,
        byte_count,
        was_line_truncated,
        was_byte_truncated,
    }
}

/// Simple file size formatter (equivalent to TypeScript formatFileSize)
fn format_file_size(bytes: usize) -> String {
    if bytes >= 1_000_000 {
        format!("{:.1}M", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1}K", bytes as f64 / 1_000.0)
    } else {
        format!("{}B", bytes)
    }
}

/// Build the typed-memory behavioral instructions (without MEMORY.md content).
/// Constrains memories to a closed four-type taxonomy (user / feedback / project /
/// reference) — content that is derivable from the current project state (code
/// patterns, architecture, git history) is explicitly excluded.
pub fn build_memory_lines(
    display_name: &str,
    memory_dir: &str,
    extra_guidelines: Option<&[&str]>,
    skip_index: bool,
) -> Vec<String> {
    let how_to_save = if skip_index {
        vec![
            "## How to save memories".to_string(),
            String::new(),
            "Write each memory to its own file (e.g., `user_role.md`, `feedback_testing.md`) using this frontmatter format:".to_string(),
            String::new(),
        ]
        .into_iter()
        .chain(MEMORY_FRONTMATTER_EXAMPLE.iter().map(|s| s.to_string()))
        .chain(vec![
            String::new(),
            "- Keep the name, description, and type fields in memory files up-to-date with the content".to_string(),
            "- Organize memory semantically by topic, not chronologically".to_string(),
            "- Update or remove memories that turn out to be wrong or outdated".to_string(),
            "- Do not write duplicate memories. First check if there is an existing memory you can update before writing a new one.".to_string(),
        ])
        .collect::<Vec<_>>()
    } else {
        vec![
            "## How to save memories".to_string(),
            String::new(),
            "Saving a memory is a two-step process:".to_string(),
            String::new(),
            "**Step 1** — write the memory to its own file (e.g., `user_role.md`, `feedback_testing.md`) using this frontmatter format:".to_string(),
            String::new(),
        ]
        .into_iter()
        .chain(MEMORY_FRONTMATTER_EXAMPLE.iter().map(|s| s.to_string()))
        .chain(vec![
            String::new(),
            format!("**Step 2** — add a pointer to that file in `{}`. `{}` is an index, not a memory — each entry should be one line, under ~150 characters: `- [Title](file.md) — one-line hook`. It has no frontmatter. Never write memory content directly into `{}`.", ENTRYPOINT_NAME, ENTRYPOINT_NAME, ENTRYPOINT_NAME),
            String::new(),
            format!("- `{}` is always loaded into your conversation context — lines after {} will be truncated, so keep the index concise", ENTRYPOINT_NAME, MAX_ENTRYPOINT_LINES),
            "- Keep the name, description, and type fields in memory files up-to-date with the content".to_string(),
            "- Organize memory semantically by topic, not chronologically".to_string(),
            "- Update or remove memories that turn out to be wrong or outdated".to_string(),
            "- Do not write duplicate memories. First check if there is an existing memory you can update before writing a new one.".to_string(),
        ])
        .collect::<Vec<_>>()
    };

    let mut lines = vec![
        format!("# {}", display_name),
        String::new(),
        format!(
            "You have a persistent, file-based memory system at `{}`. {}",
            memory_dir, DIR_EXISTS_GUIDANCE
        ),
        String::new(),
        "You should build up this memory system over time so that future conversations can have a complete picture of who the user is, how they'd like to collaborate with you, what behaviors to avoid or repeat, and the context behind the work the user gives you.".to_string(),
        String::new(),
        "If the user explicitly asks you to remember something, save it immediately as whichever type fits best. If they ask you to forget something, find and remove the relevant entry.".to_string(),
        String::new(),
    ];

    // Types section
    lines.extend(TYPES_SECTION_INDIVIDUAL.iter().map(|s| s.to_string()));
    lines.push(String::new());

    // What NOT to save
    lines.extend(WHAT_NOT_TO_SAVE_SECTION.iter().map(|s| s.to_string()));
    lines.push(String::new());

    // How to save
    lines.extend(how_to_save);
    lines.push(String::new());

    // When to access
    lines.extend(WHEN_TO_ACCESS_SECTION.iter().map(|s| s.to_string()));
    lines.push(String::new());

    // Trusting recall
    lines.extend(TRUSTING_RECALL_SECTION.iter().map(|s| s.to_string()));
    lines.push(String::new());

    // Memory and other forms of persistence
    lines.push("## Memory and other forms of persistence".to_string());
    lines.push("Memory is one of several persistence mechanisms available to you as you assist the user in a given conversation. The distinction is often that memory can be recalled in future conversations and should not be used for persisting information that is only useful within the scope of the current conversation.".to_string());
    lines.push("- When to use or update a plan instead of memory: If you are about to start a non-trivial implementation task and would like to reach alignment with the user on your approach you should use a Plan rather than saving this information to memory. Similarly, if you already have a plan within the conversation and you have changed your approach persist that change by updating the plan rather than saving a memory.".to_string());
    lines.push("- When to use or update tasks instead of memory: When you need to break your work in current conversation into discrete steps or keep track of your progress use tasks instead of saving to memory. Tasks are great for persisting information about the work that needs to be done in the current conversation, but memory should be reserved for information that will be useful in future conversations.".to_string());
    lines.push(String::new());

    // Extra guidelines
    if let Some(guidelines) = extra_guidelines {
        lines.extend(guidelines.iter().map(|s| s.to_string()));
        lines.push(String::new());
    }

    // Searching past context section (simplified - would integrate with growthbook feature flags)
    lines.extend(build_searching_past_context_section(memory_dir));

    lines
}

/// Frontmatter example for memory files
pub const MEMORY_FRONTMATTER_EXAMPLE: &[&str] = &[
    "```markdown",
    "---",
    "name: {{memory name}}",
    "description: {{one-line description — used to decide relevance in future conversations, so be specific}}",
    "type: {{user, feedback, project, reference}}",
    "---",
    "",
    "{{memory content — for feedback/project types, structure as: rule/fact, then **Why:** and **How to apply:** lines}}",
    "```",
];

/// Types section content
const TYPES_SECTION_INDIVIDUAL: &[&str] = &[
    "## Types of memory",
    "",
    "There are several discrete types of memory that you can store in your memory system:",
    "",
    "<types>",
    "<type>",
    "    <name>user</name>",
    "    <description>Contain information about the user's role, goals, responsibilities, and knowledge. Great user memories help you tailor your future behavior to the user's preferences and perspective. Your goal in reading and writing these memories is to build up an understanding of who the user is and how you can be most helpful to them specifically.</description>",
    "    <when_to_save>When you learn any details about the user's role, preferences, responsibilities, or knowledge</when_to_save>",
    "    <how_to_use>When your work should be informed by the user's profile or perspective.</how_to_use>",
    "</type>",
    "<type>",
    "    <name>feedback</name>",
    "    <description>Guidance the user has given you about how to approach work — both what to avoid and what to keep doing.</description>",
    "    <when_to_save>Any time the user corrects your approach (\"no not that\", \"don't\", \"stop doing X\") OR confirms a non-obvious approach worked.</when_to_save>",
    "    <how_to_use>Let these memories guide your behavior so that the user does not need to offer the same guidance twice.</how_to_use>",
    "</type>",
    "<type>",
    "    <name>project</name>",
    "    <description>Information that you learn about ongoing work, goals, initiatives, bugs, or incidents within the project that is not otherwise derivable from the code or git history.</description>",
    "    <when_to_save>When you learn who is doing what, why, or by when.</when_to_save>",
    "    <how_to_use>Use these memories to more fully understand the details and nuance behind the user's request.</how_to_use>",
    "</type>",
    "<type>",
    "    <name>reference</name>",
    "    <description>Stores pointers to where information can be found in external systems.</description>",
    "    <when_to_save>When you learn about resources in external systems and their purpose.</when_to_save>",
    "    <how_to_use>When the user references an external system or information that may be in an external system.</how_to_use>",
    "</type>",
    "</types>",
    "",
];

/// What NOT to save section content
const WHAT_NOT_TO_SAVE_SECTION: &[&str] = &[
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

/// When to access section content
const WHEN_TO_ACCESS_SECTION: &[&str] = &[
    "## When to access memories",
    "- When memories seem relevant, or the user references prior-conversation work.",
    "- You MUST access memory when the user explicitly asks you to check, recall, or remember.",
    "- If the user says to *ignore* or *not use* memory: proceed as if MEMORY.md were empty.",
    "- Memory records can become stale over time. Verify that the memory is still correct and up-to-date.",
];

/// Trusting recall section content
const TRUSTING_RECALL_SECTION: &[&str] = &[
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

/// Build the "Searching past context" section
pub fn build_searching_past_context_section(auto_mem_dir: &str) -> Vec<String> {
    // Simplified version - would integrate with growthbook feature flags
    // In full implementation, this would check getFeatureValue_CACHED_MAY_BE_STALE('tengu_coral_fern', false)
    vec![]
}

/// Build the memory prompt with MEMORY.md content included.
/// Used by agent memory.
pub fn build_memory_prompt(params: BuildMemoryPromptParams) -> String {
    let BuildMemoryPromptParams {
        display_name,
        extra_guidelines,
    } = params;

    let memory_dir = get_auto_mem_path();
    let memory_dir_str = memory_dir.to_string_lossy();

    // Read existing memory entrypoint
    let entrypoint_path = get_auto_mem_entrypoint();
    let entrypoint_content = if entrypoint_path.exists() {
        std::fs::read_to_string(&entrypoint_path).unwrap_or_default()
    } else {
        String::new()
    };

    let mut lines = build_memory_lines(
        &display_name,
        &memory_dir_str,
        extra_guidelines.as_deref(),
        false,
    );

    if !entrypoint_content.trim().is_empty() {
        let t = truncate_entrypoint_content(&entrypoint_content);
        lines.push(format!("## {}", ENTRYPOINT_NAME));
        lines.push(String::new());
        lines.push(t.content);
    } else {
        lines.push(format!("## {}", ENTRYPOINT_NAME));
        lines.push(String::new());
        lines.push(format!(
            "Your {} is currently empty. When you save new memories, they will appear here.",
            ENTRYPOINT_NAME
        ));
    }

    lines.join("\n")
}

/// Parameters for build_memory_prompt
pub struct BuildMemoryPromptParams<'a> {
    pub display_name: &'a str,
    pub extra_guidelines: Option<Vec<&'a str>>,
}

/// Load the unified memory prompt for inclusion in the system prompt.
/// Returns None when auto memory is disabled.
pub async fn load_memory_prompt() -> Option<String> {
    if !is_auto_memory_enabled() {
        return None;
    }

    let auto_dir = get_auto_mem_path();
    // Ensure the directory exists
    ensure_memory_dir_exists(&auto_dir).ok()?;

    Some(build_memory_prompt(BuildMemoryPromptParams {
        display_name: "auto memory",
        extra_guidelines: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_entrypoint_content_no_truncation() {
        let content = "line 1\nline 2\nline 3";
        let result = truncate_entrypoint_content(content);

        assert_eq!(result.content, content);
        assert!(!result.was_line_truncated);
        assert!(!result.was_byte_truncated);
    }

    #[test]
    fn test_truncate_entrypoint_content_line_truncation() {
        let content: String = (0..=MAX_ENTRYPOINT_LINES)
            .map(|i| format!("line {}\n", i))
            .collect();
        let result = truncate_entrypoint_content(&content);

        assert!(result.was_line_truncated);
        assert!(result.content.contains("WARNING: MEMORY.md is"));
    }

    #[test]
    fn test_build_memory_lines() {
        let lines = build_memory_lines("auto memory", "/tmp/memory", None, false);
        assert!(!lines.is_empty());
        assert!(lines.iter().any(|l| l.contains("Types of memory")));
        assert!(lines.iter().any(|l| l.contains("How to save memories")));
    }

    #[test]
    fn test_build_memory_lines_skip_index() {
        let lines = build_memory_lines("auto memory", "/tmp/memory", None, true);
        assert!(!lines.iter().any(|l| l.contains("Step 1")));
        assert!(!lines.iter().any(|l| l.contains("Step 2")));
    }

    #[test]
    fn test_build_memory_prompt() {
        let prompt = build_memory_prompt(BuildMemoryPromptParams {
            display_name: "auto memory",
            extra_guidelines: None,
        });
        assert!(prompt.contains("auto memory"));
        assert!(prompt.contains("MEMORY.md"));
    }
}
