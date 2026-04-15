// Source: ~/claudecode/openclaudecode/src/tools/SnipTool/prompt.ts
pub const SNIP_TOOL_NAME: &str = "snip";

pub const DESCRIPTION: &str = "Create a snip of code or text";

pub const PROMPT: &str = r#"Use this tool to create a snip (a focused excerpt) of code or text from the current context.

## When to Use This Tool

- When you need to highlight a specific portion of a file
- To create a shareable reference to a particular section of code
- When showing the user a specific area of interest

## Output

Returns the snipped content with context about its source location.
"#;
