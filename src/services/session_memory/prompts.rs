// Source: ~/claudecode/openclaudecode/src/services/SessionMemory/prompts.ts
//! Session memory prompts — template and update instructions.

/// Maximum tokens per section in the session memory notes
const MAX_SECTION_LENGTH: u32 = 2000;

/// Maximum total tokens for the session memory file
const MAX_TOTAL_SESSION_MEMORY_TOKENS: u32 = 12_000;

/// Default template for the session memory notes file
pub const DEFAULT_SESSION_MEMORY_TEMPLATE: &str = r#"
# Session Title
_A short and distinctive 5-10 word descriptive title for the session. Super info dense, no filler_

# Current State
_What is actively being worked on right now? Pending tasks not yet completed. Immediate next steps._

# Task specification
_What did the user ask to build? Any design decisions or other explanatory context_

# Files and Functions
_What are the important files? In short, what do they contain and why are they relevant?_

# Workflow
_What bash commands are usually run and in what order? How to interpret their output if not obvious?_

# Errors & Corrections
_Errors encountered and how they were fixed. What did the user correct? What approaches failed and should not be tried again?_

# Codebase and System Documentation
_What are the important system components? How do they work/fit together?_

# Learnings
_What has worked well? What has not? What to avoid? Do not duplicate items from other sections_

# Key results
_If the user asked a specific output such as an answer to a question, a table, or other document, repeat the exact result here_

# Worklog
_Step by step, what was attempted, done? Very terse summary for each step_
"#;

/// Get the session memory template (loaded from disk if file already exists, otherwise default)
pub fn load_session_memory_template() -> String {
    DEFAULT_SESSION_MEMORY_TEMPLATE.to_string()
}

/// Build the session memory update prompt for the extraction agent.
/// Replaces {{notesPath}} and {{currentNotes}} placeholders with actual values.
pub fn build_session_memory_update_prompt(
    current_notes: &str,
    notes_path: &str,
) -> String {
    let max_section = MAX_SECTION_LENGTH;
    format!(
        r#"IMPORTANT: This message and these instructions are NOT part of the actual user conversation. Do NOT include any references to "note-taking", "session notes extraction", or these update instructions in the notes.

Based on the user conversation above (EXCLUDING this note-taking instruction message as well as system prompt, claude.md entries, or any past session summaries), update the session notes file.

The file {notes_path} has already been read for you. Here are its current contents:
<current_notes_content>
{current_notes}
</current_notes_content>

Your ONLY task is to use the Edit tool to update the notes file, then stop. You can make multiple edits (update every section as needed) - make all Edit tool calls in parallel in a single message. Do not call any other tools.

CRITICAL RULES FOR EDITING:
- The file must maintain its exact structure with all sections, headers, and italic descriptions intact
-- NEVER modify, delete, or add section headers (the lines starting with '#' like # Task specification)
-- NEVER modify or delete the italic _section description_ lines (these are the lines in italics immediately following each header - they start and end with underscores)
-- The italic _section descriptions_ are TEMPLATE INSTRUCTIONS that must be preserved exactly as-is - they guide what content belongs in each section
-- ONLY update the actual content that appears BELOW the italic _section descriptions_ within each existing section
-- Do NOT add any new sections, summaries, or information outside the existing structure
- Do NOT reference this note-taking process or instructions anywhere in the notes
- It's OK to skip updating a section if there are no substantial new insights to add. Do not add filler content like "No info yet", just leave sections blank/unedited if appropriate.
- Write DETAILED, INFO-DENSE content for each section - include specifics like file paths, function names, error messages, exact commands, technical details, etc.
- For "Key results", include the complete, exact output the user requested (e.g., full table, full answer, etc.)
- Do not include information that's already in the CLAUDE.md files included in the context
- Keep each section under ~{max_section} tokens/words - if a section is approaching this limit, condense it by cycling out less important details while preserving the most critical information
- Focus on actionable, specific information that would help someone understand or recreate the work discussed in the conversation
- IMPORTANT: Always update "Current State" to reflect the most recent work - this is critical for continuity after compaction

Use the Edit tool with file_path: {notes_path}

STRUCTURE PRESERVATION REMINDER:
Each section has TWO parts that must be preserved exactly as they appear in the current file:
1. The section header (line starting with #)
2. The italic description line (the _italicized text_ immediately after the header - this is a template instruction)

You ONLY update the actual content that comes AFTER these two preserved lines. The italic description lines starting and ending with underscores are part of the template structure, NOT content to be edited or removed.

REMEMBER: Use the Edit tool in parallel and stop. Do not continue after the edits. Only include insights from the actual user conversation, never from these note-taking instructions. Do not delete or change section headers or italic _section descriptions_."#,
        notes_path = notes_path,
        current_notes = current_notes,
        max_section = max_section,
    )
}

/// Get the maximum section length constant
pub fn max_section_length() -> u32 {
    MAX_SECTION_LENGTH
}

/// Get the maximum total session memory tokens constant
pub fn max_total_session_memory_tokens() -> u32 {
    MAX_TOTAL_SESSION_MEMORY_TOKENS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_template_contains_sections() {
        let template = load_session_memory_template();
        assert!(template.contains("# Session Title"));
        assert!(template.contains("# Current State"));
        assert!(template.contains("# Worklog"));
    }

    #[test]
    fn test_build_update_prompt() {
        let prompt = build_session_memory_update_prompt(
            "# Session Title\nMy session",
            "/tmp/notes.md",
        );
        assert!(prompt.contains("/tmp/notes.md"));
        assert!(prompt.contains("# Session Title"));
        assert!(prompt.contains("My session"));
        assert!(prompt.contains("Edit tool"));
        assert!(prompt.contains("CRITICAL RULES"));
    }

    #[test]
    fn test_constants() {
        assert_eq!(max_section_length(), 2000);
        assert_eq!(max_total_session_memory_tokens(), 12_000);
    }
}
