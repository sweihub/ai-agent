// Source: ~/claudecode/openclaudecode/src/services/compact/prompt.ts
//! Compact prompt templates for conversation summarization.
//!
//! Three prompt variants:
//! - BASE_COMPACT_PROMPT: Full conversation summary
//! - PARTIAL_COMPACT_PROMPT: Recent messages only (after retained context)
//! - PARTIAL_COMPACT_UP_TO_PROMPT: Earlier messages (before retained suffix)

/// Aggressive no-tools preamble
const NO_TOOLS_PREAMBLE: &str = r#"CRITICAL: Respond with TEXT ONLY. Do NOT call any tools.

- Do NOT use Read, Bash, Grep, Glob, Edit, Write, or ANY other tool.
- You already have all the context you need in the conversation above.
- Tool calls will be REJECTED and will waste your only turn — you will fail the task.
- Your entire response must be plain text: an <analysis> block followed by a <summary> block.

"#;

/// Analysis instruction for full compaction
const DETAILED_ANALYSIS_INSTRUCTION_BASE: &str = r#"Before providing your final summary, wrap your analysis in <analysis> tags to organize your thoughts and ensure you've covered all necessary points. In your analysis process:

1. Chronologically analyze each message and section of the conversation. For each section thoroughly identify:
   - The user's explicit requests and intents
   - Your approach to addressing the user's requests
   - Key decisions, technical concepts and code patterns
   - Specific details like:
     - file names
     - full code snippets
     - function signatures
     - file edits
   - Errors that you ran into and how you fixed them
   - Pay special attention to specific user feedback that you received, especially if the user told you to do something differently.
2. Double-check for technical accuracy and completeness, addressing each required element thoroughly."#;

/// Analysis instruction for partial compaction
const DETAILED_ANALYSIS_INSTRUCTION_PARTIAL: &str = r#"Before providing your final summary, wrap your analysis in <analysis> tags to organize your thoughts and ensure you've covered all necessary points. In your analysis process:

1. Analyze the recent messages chronologically. For each section thoroughly identify:
   - The user's explicit requests and intents
   - Your approach to addressing the user's requests
   - Key decisions, technical concepts and code patterns
   - Specific details like:
     - file names
     - full code snippets
     - function signatures
     - file edits
   - Errors that you ran into and how you fixed them
   - Pay special attention to specific user feedback that you received, especially if the user told you to do something differently.
2. Double-check for technical accuracy and completeness, addressing each required element thoroughly."#;

/// Base compact prompt - full conversation summary
const BASE_COMPACT_PROMPT: &str = r#"Your task is to create a detailed summary of the conversation so far, paying close attention to the user's explicit requests and your previous actions.
This summary should be thorough in capturing technical details, code patterns, and architectural decisions that would be essential for continuing development work without losing context.

DETAILED_ANALYSIS

Your summary should include the following sections:

1. Primary Request and Intent: Capture all of the user's explicit requests and intents in detail
2. Key Technical Concepts: List all important technical concepts, technologies, and frameworks discussed.
3. Files and Code Sections: Enumerate specific files and code sections examined, modified, or created. Pay special attention to the most recent messages and include full code snippets where applicable and include a summary of why this file read or edit is important.
4. Errors and fixes: List all errors that you ran into, and how you fixed them. Pay special attention to specific user feedback that you received, especially if the user told you to do something differently.
5. Problem Solving: Document problems solved and any ongoing troubleshooting efforts.
6. All user messages: List ALL user messages that are not tool results. These are critical for understanding the users' feedback and changing intent.
7. Pending Tasks: Outline any pending tasks that you have explicitly been asked to work on.
8. Current Work: Describe in detail precisely what was being worked on immediately before this summary request, paying special attention to the most recent messages from both user and assistant. Include file names and code snippets where applicable.
9. Optional Next Step: List the next step that you will take that is related to the most recent work you were doing. IMPORTANT: ensure that this step is DIRECTLY in line with the user's most recent explicit requests, and the task you were working on immediately before this summary request. If your last task was concluded, then only list next steps if they are explicitly in line with the users request. Do not start on tangential requests or really old requests that were already completed without confirming with the user first.
                       If there is a next step, include direct quotes from the most recent conversation showing exactly what task you were working on and where you left off. This should be verbatim to ensure there's no drift in task interpretation.

Here's an example of how your output should be structured:

<example>
<analysis>
[Your thought process, ensuring all points are covered thoroughly and accurately]
</analysis>

<summary>
1. Primary Request and Intent:
   [Detailed description]

2. Key Technical Concepts:
   - [Concept 1]
   - [Concept 2]
   - [...]

3. Files and Code Sections:
   - [File Name 1]
      - [Summary of why this file is important]
      - [Summary of the changes made to this file, if any]
      - [Important Code Snippet]
   - [File Name 2]
      - [Important Code Snippet]
   - [...]

4. Errors and fixes:
    - [Detailed description of error 1]:
      - [How you fixed the error]
      - [User feedback on the error if any]
    - [...]

5. Problem Solving:
   [Description of solved problems and ongoing troubleshooting]

6. All user messages:
    - [Detailed non tool use user message]
    - [...]

7. Pending Tasks:
   - [Task 1]
   - [Task 2]
   - [...]

8. Current Work:
   [Precise description of current work]

9. Optional Next Step:
   [Optional Next step to take]

</summary>
</example>

Please provide your summary based on the conversation so far, following this structure and ensuring precision and thoroughness in your response.

There may be additional summarization instructions provided in the included context. If so, remember to follow those instructions when creating the above summary. Examples of instructions include:
<example>
## Compact Instructions
When summarizing the conversation focus on typescript code changes and also remember the mistakes you made and how you fixed them.
</example>

<example>
# Summary instructions
When you are using compact - please focus on test output and code changes. Include file reads verbatim.
</example>

REMINDER: Do NOT call any tools. Respond with plain text only — an <analysis> block followed by a <summary> block. Tool calls will be rejected and you will fail the task.
"#;

/// Partial compact prompt - recent messages only
const PARTIAL_COMPACT_PROMPT: &str = r#"Your task is to create a detailed summary of the RECENT portion of the conversation — the messages that follow earlier retained context. The earlier messages are being kept intact and do NOT need to be summarized. Focus your summary on what was discussed, learned, and accomplished in the recent messages only.

DETAILED_ANALYSIS

Your summary should include the following sections:

1. Primary Request and Intent: Capture the user's explicit requests and intents from the recent messages
2. Key Technical Concepts: List important technical concepts, technologies, and frameworks discussed recently.
3. Files and Code Sections: Enumerate specific files and code sections examined, modified, or created. Include full code snippets where applicable and include a summary of why this file read or edit is important.
4. Errors and fixes: List errors encountered and how they were fixed.
5. Problem Solving: Document problems solved and any ongoing troubleshooting efforts.
6. All user messages: List ALL user messages from the recent portion that are not tool results.
7. Pending Tasks: Outline any pending tasks from the recent messages.
8. Current Work: Describe precisely what was being worked on immediately before this summary request.
9. Optional Next Step: List the next step related to the most recent work. Include direct quotes from the most recent conversation.

Here's an example of how your output should be structured:

<example>
<analysis>
[Your thought process, ensuring all points are covered thoroughly and accurately]
</analysis>

<summary>
1. Primary Request and Intent:
   [Detailed description]

2. Key Technical Concepts:
   - [Concept 1]
   - [Concept 2]

3. Files and Code Sections:
   - [File Name 1]
      - [Summary of why this file is important]
      - [Important Code Snippet]

4. Errors and fixes:
    - [Error description]:
      - [How you fixed it]

5. Problem Solving:
   [Description]

6. All user messages:
    - [Detailed non tool use user message]

7. Pending Tasks:
   - [Task 1]

8. Current Work:
   [Precise description of current work]

9. Optional Next Step:
   [Optional Next step to take]

</summary>
</example>

Please provide your summary of the recent messages following this structure. There may be additional summarization instructions provided in the included context.

REMINDER: Do NOT call any tools. Respond with plain text only — an <analysis> block followed by a <summary> block."#;

/// Partial compact up-to prompt - earlier messages (before retained suffix)
const PARTIAL_COMPACT_UP_TO_PROMPT: &str = r#"Your task is to create a detailed summary of the EARLIER portion of the conversation — the messages that precede the retained later context. The later messages are being kept intact and do NOT need to be summarized. Focus your summary on what was discussed, learned, and accomplished in the earlier messages only.

DETAILED_ANALYSIS

Your summary should include the following sections:

1. Primary Request and Intent: Capture the user's explicit requests and intents from the earlier messages
2. Key Technical Concepts: List important technical concepts, technologies, and frameworks discussed.
3. Files and Code Sections: Enumerate specific files and code sections examined, modified, or created.
4. Errors and fixes: List errors encountered and how they were fixed.
5. Problem Solving: Document problems solved and ongoing troubleshooting.
6. All user messages: List ALL user messages from the earlier portion that are not tool results.
7. Pending Tasks: Outline any pending tasks.
8. Current Work: Describe what was being worked on.
9. Context for Continuing Work: Provide key context, decisions, or state needed to continue the work from the earlier portion.

Here's an example of how your output should be structured:

<example>
<analysis>
[Your thought process, ensuring all points are covered thoroughly and accurately]
</analysis>

<summary>
1. Primary Request and Intent:
   [Detailed description]

2. Key Technical Concepts:
   - [Concept 1]
   - [Concept 2]

3. Files and Code Sections:
   - [File Name 1]
      - [Summary of why this file is important]
      - [Important Code Snippet]

4. Errors and fixes:
    - [Error description]:
      - [How you fixed it]

5. Problem Solving:
   [Description]

6. All user messages:
    - [Detailed non tool use user message]

7. Pending Tasks:
   - [Task 1]

8. Current Work:
   [Precise description of current work]

9. Context for Continuing Work:
   [Key context needed to continue]

</summary>
</example>

Please provide your summary of the earlier messages following this structure.

REMINDER: Do NOT call any tools. Respond with plain text only — an <analysis> block followed by a <summary> block."#;

/// Get the full compact prompt with optional custom instructions
pub fn get_compact_prompt(custom_instructions: Option<&str>) -> String {
    let prompt = format!(
        "{}{}{}",
        NO_TOOLS_PREAMBLE, DETAILED_ANALYSIS_INSTRUCTION_BASE, BASE_COMPACT_PROMPT
    );

    if let Some(instructions) = custom_instructions {
        if !instructions.trim().is_empty() {
            format!("{}\n\n## Custom Instructions\n{}\n", prompt, instructions)
        } else {
            prompt
        }
    } else {
        prompt
    }
}

/// Get the partial compact prompt
pub fn get_partial_compact_prompt(
    direction: PartialCompactDirection,
    custom_instructions: Option<&str>,
) -> String {
    let analysis_instruction = DETAILED_ANALYSIS_INSTRUCTION_PARTIAL;
    let base_prompt = match direction {
        PartialCompactDirection::From => PARTIAL_COMPACT_PROMPT,
        PartialCompactDirection::UpTo => PARTIAL_COMPACT_UP_TO_PROMPT,
    };

    let prompt = format!(
        "{}{}{}",
        NO_TOOLS_PREAMBLE, analysis_instruction, base_prompt
    );

    if let Some(instructions) = custom_instructions {
        if !instructions.trim().is_empty() {
            format!("{}\n\n## Custom Instructions\n{}\n", prompt, instructions)
        } else {
            prompt
        }
    } else {
        prompt
    }
}

/// Direction for partial compaction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PartialCompactDirection {
    /// Summarize messages AFTER pivot index (keeps earlier context)
    From,
    /// Summarize messages BEFORE pivot index (keeps later context)
    UpTo,
}

impl Default for PartialCompactDirection {
    fn default() -> Self {
        PartialCompactDirection::From
    }
}

/// Format the compact summary by stripping analysis tags and cleaning whitespace.
/// Matches TypeScript's formatCompactSummary()
pub fn format_compact_summary(summary: &str) -> String {
    // Strip <analysis>...</analysis> section (drafting scratchpad)
    let without_analysis = strip_analysis_tags(summary);

    // Replace <summary>...</summary> with "Summary:" header
    let with_header = replace_summary_tags(&without_analysis);

    // Clean extra whitespace
    clean_whitespace(&with_header)
}

/// Strip <analysis> tags and their content from the summary
fn strip_analysis_tags(text: &str) -> String {
    // Remove everything between <analysis> and </analysis> (inclusive)
    let mut result = text.to_string();
    while let Some(start) = result.find("<analysis>") {
        if let Some(end) = result[start..].find("</analysis>") {
            let end_pos = start + end + "</analysis>".len();
            result.replace_range(start..end_pos, "");
        } else {
            break;
        }
    }
    result
}

/// Replace <summary> tags with "Summary:" header
fn replace_summary_tags(text: &str) -> String {
    let mut result = text
        .replace("<summary>", "Summary:\n")
        .replace("</summary>", "");
    // Clean up the Summary: header formatting
    result = result.replace("Summary:\n\n", "Summary:\n");
    result
}

/// Clean extra whitespace: trim, collapse multiple blank lines to one
fn clean_whitespace(text: &str) -> String {
    let trimmed = text.trim();
    // Collapse 3+ consecutive newlines to 2 (one blank line)
    let mut result = String::with_capacity(trimmed.len());
    let mut consecutive_newlines = 0;
    for ch in trimmed.chars() {
        if ch == '\n' {
            consecutive_newlines += 1;
            if consecutive_newlines <= 2 {
                result.push(ch);
            }
        } else {
            consecutive_newlines = 0;
            result.push(ch);
        }
    }
    result
}

/// Get the user-facing compact summary message.
/// Matches TypeScript's getCompactUserSummaryMessage()
pub fn get_compact_user_summary_message(
    summary: &str,
    suppress_follow_up_questions: Option<bool>,
    transcript_path: Option<&str>,
    recent_preserved: Option<bool>,
) -> String {
    let mut message = String::new();

    message.push_str("## Session Continued from Previous Conversation\n\n");
    message.push_str("The conversation below is a continuation of a previous session. ");
    message.push_str("Here's a summary of what was discussed before:\n\n");
    message.push_str(summary);

    if recent_preserved == Some(true) {
        message.push_str(
            "\n\nRecent messages are preserved verbatim — they do not need to be re-summarized.",
        );
    }

    if let Some(path) = transcript_path {
        message.push_str(&format!(
            "\n\nFor the complete conversation transcript, see: {}",
            path
        ));
    }

    if suppress_follow_up_questions == Some(true) {
        message.push_str(
            "\n\nPlease continue working on the task without asking follow-up questions. \
             If you need to take action, do so directly.",
        );
    }

    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_compact_summary_strips_analysis() {
        let input = r#"<analysis>
Let me think through this...
</analysis>

<summary>
1. Primary Request: Build a tool
</summary>"#;
        let result = format_compact_summary(input);
        assert!(!result.contains("<analysis>"));
        assert!(!result.contains("</analysis>"));
        assert!(!result.contains("<summary>"));
        assert!(!result.contains("</summary>"));
        assert!(result.contains("Summary:"));
        assert!(result.contains("1. Primary Request: Build a tool"));
    }

    #[test]
    fn test_format_compact_summary_no_analysis() {
        let input = "<summary>\n1. Request: Test\n</summary>";
        let result = format_compact_summary(input);
        assert!(result.contains("Summary:"));
        assert!(result.contains("1. Request: Test"));
    }

    #[test]
    fn test_format_compact_summary_cleans_whitespace() {
        let input = "<summary>\n\nTest\n\n\n\nMore\n\n\n</summary>";
        let result = format_compact_summary(input);
        // Should not have 3+ consecutive newlines
        assert!(!result.contains("\n\n\n"));
    }

    #[test]
    fn test_get_compact_prompt_contains_no_tools() {
        let prompt = get_compact_prompt(None);
        assert!(prompt.contains("TEXT ONLY"));
        assert!(prompt.contains("Do NOT call any tools"));
    }

    #[test]
    fn test_get_compact_prompt_custom_instructions() {
        let prompt = get_compact_prompt(Some("Focus on code changes"));
        assert!(prompt.contains("Focus on code changes"));
    }

    #[test]
    fn test_get_compact_prompt_empty_custom_instructions() {
        let prompt = get_compact_prompt(Some(""));
        assert!(!prompt.contains("Custom Instructions"));
    }

    #[test]
    fn test_get_partial_compact_prompt_from() {
        let prompt = get_partial_compact_prompt(PartialCompactDirection::From, None);
        assert!(prompt.contains("RECENT portion"));
        assert!(prompt.contains("Do NOT call any tools"));
    }

    #[test]
    fn test_get_partial_compact_prompt_up_to() {
        let prompt = get_partial_compact_prompt(PartialCompactDirection::UpTo, None);
        assert!(prompt.contains("EARLIER portion"));
        assert!(prompt.contains("Context for Continuing Work"));
    }

    #[test]
    fn test_get_compact_user_summary_message_basic() {
        let msg = get_compact_user_summary_message("Test summary", None, None, None);
        assert!(msg.contains("Session Continued from Previous Conversation"));
        assert!(msg.contains("Test summary"));
    }

    #[test]
    fn test_get_compact_user_summary_message_suppress() {
        let msg = get_compact_user_summary_message("Test summary", Some(true), None, None);
        assert!(msg.contains("without asking follow-up questions"));
    }

    #[test]
    fn test_get_compact_user_summary_message_transcript() {
        let msg = get_compact_user_summary_message(
            "Test summary",
            None,
            Some("/path/to/transcript"),
            None,
        );
        assert!(msg.contains("/path/to/transcript"));
    }

    #[test]
    fn test_get_compact_user_summary_message_recent_preserved() {
        let msg = get_compact_user_summary_message("Test summary", None, None, Some(true));
        assert!(msg.contains("Recent messages are preserved verbatim"));
    }
}
