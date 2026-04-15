// Source: /data/home/swei/claudecode/openclaudecode/src/entrypoints/init.ts
use super::Command;

const OLD_INIT_PROMPT: &str = r#"Please analyze this codebase and create a AI.md file, which will be given to future instances of AI Agent SDK to operate in this repository.

What to add:
1. Commands that will be commonly used, such as how to build, lint, and run tests. Include the necessary commands to develop in this codebase, such as how to run a single test.
2. High-level code architecture and structure so that future instances can be productive more quickly. Focus on the "big picture" architecture that requires reading multiple files to understand.

Usage notes:
- If there's already a AI.md, suggest improvements to it.
- When you make the initial AI.md, do not repeat yourself and do not include obvious instructions like "Provide helpful error messages to users", "Write unit tests for all new utilities", "Never include sensitive information (API keys, tokens) in code or commits".
- Avoid listing every component or file structure that can be easily discovered.
- Don't include generic development practices.
- If there are Cursor rules (in .cursor/rules/ or .cursorrules) or Copilot rules (in .github/copilot-instructions.md), make sure to include the important parts.
- If there is a README.md, make sure to include the important parts.
- Do not make up information such as "Common Development Tasks", "Tips for Development", "Support and Documentation" unless this is expressly included in other files that you read.
- Be sure to prefix the file with the following text:

```
# AI.md

This file provides guidance to AI Agent SDK (claude.ai/code) when working with code in this repository.
```"#;

pub fn create_init_command() -> Command {
    Command::prompt(
        "init",
        "Initialize a new AI.md file with codebase documentation",
    )
    .argument_hint("[--new]")
}

pub fn get_init_prompt(is_new: bool) -> String {
    if is_new {
        "Use the new init flow".to_string()
    } else {
        OLD_INIT_PROMPT.to_string()
    }
}
