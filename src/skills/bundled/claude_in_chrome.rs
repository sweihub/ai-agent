//! Claude in Chrome skill - ported from openclaudecode/src/skills/bundled/claudeInChrome.ts
//!
//! Claude in Chrome extension integration (feature-gated: auto-enabled).

use crate::AgentError;
use crate::skills::bundled_skills::{
    BundledSkillDefinition, ContentBlock, SkillContext, register_bundled_skill,
};

const CLAUDE_IN_CHROME_PROMPT: &str = r#"# Claude in Chrome

Use Claude Code within Chrome browser.

## Features

- Claude assistance in Chrome
- Web page analysis
- Browser automation

## Usage

Access Claude in Chrome through the extension.
"#;

fn get_prompt_for_command(
    args: &str,
    _context: &SkillContext,
) -> Result<Vec<ContentBlock>, AgentError> {
    let prompt = if args.is_empty() {
        CLAUDE_IN_CHROME_PROMPT.to_string()
    } else {
        format!("{}\n\n## Request\n\n{}", CLAUDE_IN_CHROME_PROMPT, args)
    };
    Ok(vec![ContentBlock::Text { text: prompt }])
}

pub fn register_claude_in_chrome_skill() {
    let _ = register_bundled_skill(BundledSkillDefinition {
        name: "claude-in-chrome".to_string(),
        description: "Use Claude Code within Chrome browser".to_string(),
        aliases: None,
        when_to_use: None,
        argument_hint: None,
        allowed_tools: None,
        model: None,
        disable_model_invocation: None,
        user_invocable: Some(true),
        is_enabled: None,
        context: None,
        agent: None,
        files: None,
        get_prompt_for_command,
    });
}
