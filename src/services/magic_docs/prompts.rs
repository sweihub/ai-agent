// Source: ~/claudecode/openclaudecode/src/services/MagicDocs/prompts.ts
//! Magic Docs prompt templates and variable substitution.
//! Translated from ~/claudecode/openclaudecode/src/services/MagicDocs/prompts.ts

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::utils::env_utils::get_claude_config_home_dir;

/// Default Magic Docs update prompt template.
/// This is the built-in prompt used when no custom prompt exists.
fn get_update_prompt_template() -> &'static str {
    r#"IMPORTANT: This message and these instructions are NOT part of the actual user conversation. Do NOT include any references to "documentation updates", "magic docs", or these update instructions in the document content.

Based on the user conversation above (EXCLUDING this documentation update instruction message), update the Magic Doc file to incorporate any NEW learnings, insights, or information that would be valuable to preserve.

The file {{docPath}} has already been read for you. Here are its current contents:
<current_doc_content>
{{docContents}}
</current_doc_content>

Document title: {{docTitle}}
{{customInstructions}}

Your ONLY task is to use the Edit tool to update the documentation file if there is substantial new information to add, then stop. You can make multiple edits (update multiple sections as needed) - make all Edit tool calls in parallel in a single message. If there's nothing substantial to add, simply respond with a brief explanation and do not call any tools.

CRITICAL RULES FOR EDITING:
- Preserve the Magic Doc header exactly as-is: # MAGIC DOC: {{docTitle}}
- If there's an italicized line immediately after the header, preserve it exactly as-is
- Keep the document CURRENT with the latest state of the codebase - this is NOT a changelog or history
- Update information IN-PLACE to reflect the current state - do NOT append historical notes or track changes over time
- Remove or replace outdated information rather than adding "Previously..." or "Updated to..." notes
- Clean up or DELETE sections that are no longer relevant or don't align with the document's purpose
- Fix obvious errors: typos, grammar mistakes, broken formatting, incorrect information, or confusing statements
- Keep the document well organized: use clear headings, logical section order, consistent formatting, and proper nesting

DOCUMENTATION PHILOSOPHY - READ CAREFULLY:
- BE TERSE. High signal only. No filler words or unnecessary elaboration.
- Documentation is for OVERVIEWS, ARCHITECTURE, and ENTRY POINTS - not detailed code walkthroughs
- Do NOT duplicate information that's already obvious from reading the source code
- Do NOT document every function, parameter, or line number reference
- Focus on: WHY things exist, HOW components connect, WHERE to start reading, WHAT patterns are used
- Skip: detailed implementation steps, exhaustive API docs, play-by-play narratives

What TO document:
- High-level architecture and system design
- Non-obvious patterns, conventions, or gotchas
- Key entry points and where to start reading code
- Important design decisions and their rationale
- Critical dependencies or integration points
- References to related files, docs, or code (like a wiki) - help readers navigate to relevant context

What NOT to document:
- Anything obvious from reading the code itself
- Exhaustive lists of files, functions, or parameters
- Step-by-step implementation details
- Low-level code mechanics
- Information already in CLAUDE.md or other project docs

Use the Edit tool with file_path: {{docPath}}

REMEMBER: Only update if there is substantial new information. The Magic Doc header (# MAGIC DOC: {{docTitle}}) must remain unchanged."#
}

/// Load custom Magic Docs prompt from file if it exists.
/// Custom prompts can be placed at ~/.ai/magic-docs/prompt.md
/// Uses {{variableName}} syntax for variable substitution.
async fn load_magic_docs_prompt() -> String {
    let config_home = get_claude_config_home_dir();
    let prompt_path = Path::new(&config_home).join("magic-docs").join("prompt.md");

    match fs::read_to_string(&prompt_path) {
        Ok(content) => {
            if !content.is_empty() {
                return content;
            }
        }
        Err(_) => {
            // Silently fall back to default if custom prompt doesn't exist or fails to load
        }
    }

    get_update_prompt_template().to_string()
}

/// Substitute variables in the prompt template using {{variable}} syntax.
/// Single-pass replacement avoids: (1) $ backreference corruption,
/// and (2) double-substitution when user content happens to contain
/// {{varName}} matching a later variable.
fn substitute_variables(template: &str, variables: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    // Use regex for {{variable}} replacement
    let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
    result = re
        .replace_all(&result, |caps: &regex::Captures| {
            let key = &caps[1];
            if let Some(value) = variables.get(key) {
                value.clone()
            } else {
                caps[0].to_string()
            }
        })
        .to_string();
    result
}

/// Build the Magic Docs update prompt with variable substitution.
///
/// # Arguments
/// * `doc_contents` - Current contents of the magic doc file
/// * `doc_path` - File path of the magic doc
/// * `doc_title` - Document title extracted from the magic doc header
/// * `instructions` - Optional custom instructions provided by the document author
pub async fn build_magic_docs_update_prompt(
    doc_contents: &str,
    doc_path: &str,
    doc_title: &str,
    instructions: Option<&str>,
) -> String {
    let prompt_template = load_magic_docs_prompt().await;

    // Build custom instructions section if provided
    let custom_instructions = if let Some(inst) = instructions {
        format!(
            r#"

DOCUMENT-SPECIFIC UPDATE INSTRUCTIONS:
The document author has provided specific instructions for how this file should be updated. Pay extra attention to these instructions and follow them carefully:

"{inst}"

These instructions take priority over the general rules below. Make sure your updates align with these specific guidelines."#
        )
    } else {
        String::new()
    };

    // Build variables map
    let mut variables = HashMap::new();
    variables.insert("docContents".to_string(), doc_contents.to_string());
    variables.insert("docPath".to_string(), doc_path.to_string());
    variables.insert("docTitle".to_string(), doc_title.to_string());
    variables.insert("customInstructions".to_string(), custom_instructions);

    substitute_variables(&prompt_template, &variables)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_variables() {
        let template = "Hello {{name}}, welcome to {{place}}!";
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), "World".to_string());
        variables.insert("place".to_string(), "Rust".to_string());

        let result = substitute_variables(template, &variables);
        assert_eq!(result, "Hello World, welcome to Rust!");
    }

    #[test]
    fn test_substitute_variables_missing_key() {
        let template = "Hello {{name}}!";
        let variables = HashMap::new();

        let result = substitute_variables(template, &variables);
        assert_eq!(result, "Hello {{name}}!");
    }

    #[test]
    fn test_build_magic_docs_update_prompt_basic() {
        let contents = "# MAGIC DOC: Test Title\n\nSome content";
        let result = futures::executor::block_on(
            build_magic_docs_update_prompt(contents, "/test/path.md", "Test Title", None),
        );

        assert!(result.contains("Test Title"));
        assert!(result.contains("/test/path.md"));
        assert!(result.contains("IMPORTANT: This message and these instructions are NOT part"));
        assert!(!result.contains("DOCUMENT-SPECIFIC UPDATE INSTRUCTIONS"));
    }

    #[test]
    fn test_build_magic_docs_update_prompt_with_instructions() {
        let contents = "# MAGIC DOC: API Guide\n\nContent";
        let result = futures::executor::block_on(build_magic_docs_update_prompt(
            contents,
            "/test/api.md",
            "API Guide",
            Some("Keep it short"),
        ));

        assert!(result.contains("DOCUMENT-SPECIFIC UPDATE INSTRUCTIONS"));
        assert!(result.contains("Keep it short"));
    }
}
