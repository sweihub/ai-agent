// Source: /data/home/swei/claudecode/openclaudecode/src/memdir/teamMemPrompts.ts
//! Team memory prompts
//!
//! Provides the combined memory prompt when both auto memory and team memory are enabled.

use crate::memdir::memdir::{
    DIR_EXISTS_GUIDANCE, ENTRYPOINT_NAME, MAX_ENTRYPOINT_LINES, MEMORY_FRONTMATTER_EXAMPLE,
    build_searching_past_context_section,
};
use crate::memdir::memory_types::{
    MEMORY_DRIFT_CAVEAT, TRUSTING_RECALL_SECTION, TYPES_SECTION_COMBINED, WHAT_NOT_TO_SAVE_SECTION,
};
use crate::memdir::paths::get_auto_mem_path;
use crate::memdir::team_mem_paths::get_team_mem_path;

/// Build the combined prompt when both auto memory and team memory are enabled.
/// Closed four-type taxonomy (user / feedback / project / reference) with
/// per-type <scope> guidance embedded in XML-style <type> blocks.
pub fn build_combined_memory_prompt(
    extra_guidelines: Option<Vec<&str>>,
    skip_index: bool,
) -> String {
    let auto_dir = get_auto_mem_path();
    let team_dir = get_team_mem_path();

    let how_to_save: Vec<String> = if skip_index {
        let mut lines = vec![
            "## How to save memories".to_string(),
            String::new(),
            "Write each memory to its own file in the chosen directory (private or team, per the type's scope guidance) using this frontmatter format:".to_string(),
            String::new(),
        ];
        for s in MEMORY_FRONTMATTER_EXAMPLE {
            lines.push(s.to_string());
        }
        lines.push(String::new());
        lines.extend(vec![
            "- Keep the name, description, and type fields in memory files up-to-date with the content".to_string(),
            "- Organize memory semantically by topic, not chronologically".to_string(),
            "- Update or remove memories that turn out to be wrong or outdated".to_string(),
            "- Do not write duplicate memories. First check if there is an existing memory you can update before writing a new one.".to_string(),
        ]);
        lines
    } else {
        let mut lines = vec![
            "## How to save memories".to_string(),
            String::new(),
            "Saving a memory is a two-step process:".to_string(),
            String::new(),
            "**Step 1** — write the memory to its own file in the chosen directory (private or team, per the type's scope guidance) using this frontmatter format:".to_string(),
            String::new(),
        ];
        for s in MEMORY_FRONTMATTER_EXAMPLE {
            lines.push(s.to_string());
        }
        lines.push(String::new());
        lines.push(format!(
            "**Step 2** — add a pointer to that file in the same directory's `{}`. Each directory (private and team) has its own `{}` index — each entry should be one line, under ~150 characters: `- [Title](file.md) — one-line hook`. They have no frontmatter. Never write memory content directly into a `{}`.",
            ENTRYPOINT_NAME, ENTRYPOINT_NAME, ENTRYPOINT_NAME
        ));
        lines.push(String::new());
        lines.push(format!(
            "- Both `{}` indexes are loaded into your conversation context — lines after {} will be truncated, so keep them concise",
            ENTRYPOINT_NAME, MAX_ENTRYPOINT_LINES
        ));
        lines.extend(vec![
            "- Keep the name, description, and type fields in memory files up-to-date with the content".to_string(),
            "- Organize memory semantically by topic, not chronologically".to_string(),
            "- Update or remove memories that turn out to be wrong or outdated".to_string(),
            "- Do not write duplicate memories. First check if there is an existing memory you can update before writing a new one.".to_string(),
        ]);
        lines
    };

    let dir_exists_guidance = "Both directories already exist — write to them directly with the Write tool (do not run mkdir or check for their existence).";

    let mut lines: Vec<String> = vec![
        "# Memory".to_string(),
        String::new(),
        format!(
            "You have a persistent, file-based memory system with two directories: a private directory at `{}` and a shared team directory at `{}`. {}",
            auto_dir.to_string_lossy(),
            team_dir.to_string_lossy(),
            dir_exists_guidance
        ),
        String::new(),
        "You should build up this memory system over time so that future conversations can have a complete picture of who the user is, how they'd like to collaborate with you, what behaviors to avoid or repeat, and the context behind the work the user gives you.".to_string(),
        String::new(),
        "If the user explicitly asks you to remember something, save it immediately as whichever type fits best. If they ask you to forget something, find and remove the relevant entry.".to_string(),
        String::new(),
        "## Memory scope".to_string(),
        String::new(),
        "There are two scope levels:".to_string(),
        String::new(),
        format!("- private: memories that are private between you and the current user. They persist across conversations with only this specific user and are stored at the root `{}`.", auto_dir.to_string_lossy()),
        format!("- team: memories that are shared with and contributed by all of the users who work within this project directory. Team memories are synced at the beginning of every session and they are stored at `{}`.", team_dir.to_string_lossy()),
        String::new(),
    ];

    // Types section
    for s in TYPES_SECTION_COMBINED {
        lines.push(s.to_string());
    }
    for s in WHAT_NOT_TO_SAVE_SECTION {
        lines.push(s.to_string());
    }
    lines.push("- You MUST avoid saving sensitive data within shared team memories. For example, never save API keys or user credentials.".to_string());
    lines.push(String::new());

    // How to save
    lines.extend(how_to_save);
    lines.push(String::new());

    // When to access
    lines.push("## When to access memories".to_string());
    lines.push("- When memories (personal or team) seem relevant, or the user references prior work with them or others in their organization.".to_string());
    lines.push(
        "- You MUST access memory when the user explicitly asks you to check, recall, or remember."
            .to_string(),
    );
    lines.push("- If the user says to *ignore* or *not use* memory: proceed as if MEMORY.md were empty. Do not apply remembered facts, cite, compare against, or mention memory content.".to_string());
    lines.push(MEMORY_DRIFT_CAVEAT.to_string());
    lines.push(String::new());

    // Trusting recall
    for s in TRUSTING_RECALL_SECTION {
        lines.push(s.to_string());
    }
    lines.push(String::new());

    // Memory and other forms of persistence
    lines.push("## Memory and other forms of persistence".to_string());
    lines.push("Memory is one of several persistence mechanisms available to you as you assist the user in a given conversation. The distinction is often that memory can be recalled in future conversations and should not be used for persisting information that is only useful within the scope of the current conversation.".to_string());
    lines.push("- When to use or update a plan instead of memory: If you are about to start a non-trivial implementation task and you would like to reach alignment with the user on your approach you should use a Plan rather than saving this information to memory. Similarly, if you already have a plan within the conversation and you have changed your approach persist that change by updating the plan rather than saving a memory.".to_string());
    lines.push("- When to use or update tasks instead of memory: When you need to break your work in current conversation into discrete steps or keep track of your progress use tasks instead of saving to memory. Tasks are great for persisting information about the work that needs to be done in the current conversation, but memory should be reserved for information that will be useful in future conversations.".to_string());

    // Extra guidelines
    if let Some(guidelines) = extra_guidelines {
        for g in guidelines {
            lines.push(g.to_string());
        }
        lines.push(String::new());
    }

    // Searching past context section
    for s in build_searching_past_context_section(&auto_dir.to_string_lossy()) {
        lines.push(s);
    }

    lines.join("\n")
}
