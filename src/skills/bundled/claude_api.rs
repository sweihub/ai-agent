//! Claude API skill - ported from openclaudecode/src/skills/bundled/claudeApi.ts
//!
//! Build apps with the Claude API or Anthropic SDK.

use crate::AgentError;
use crate::skills::bundled_skills::{
    BundledSkillDefinition, ContentBlock, SkillContext, register_bundled_skill,
};

const CLAUDE_API_PROMPT: &str = r#"# Claude API Skill

Build apps with the Claude API or Anthropic SDK.

## Overview

The Claude API allows you to integrate Claude's capabilities into your applications.

## SDK Support

Available SDKs:
- Python
- TypeScript/JavaScript
- Go
- Ruby
- Java
- PHP
- C#

## Getting Started

1. Get an API key from Anthropic console
2. Install the SDK for your language
3. Make API calls with your key

## Documentation

See the skill files for detailed guides on:
- Tool use
- Streaming
- Batches
- Files API
"#;

fn get_prompt_for_command(
    args: &str,
    _context: &SkillContext,
) -> Result<Vec<ContentBlock>, AgentError> {
    let mut prompt = CLAUDE_API_PROMPT.to_string();
    if !args.is_empty() {
        prompt.push_str("\n\n## Request\n\n");
        prompt.push_str(args);
    }
    Ok(vec![ContentBlock::Text { text: prompt }])
}

pub fn register_claude_api_skill() {
    let _ = register_bundled_skill(BundledSkillDefinition {
        name: "claude-api".to_string(),
        description: "Build apps with the Claude API or Anthropic SDK".to_string(),
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
