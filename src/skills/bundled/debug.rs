// Source: /data/home/swei/claudecode/openclaudecode/src/skills/bundled/debug.ts
//! Debug skill - ported from openclaudecode/src/skills/bundled/debug.ts
//!
//! Enable debug logging for this session and help diagnose issues.

use crate::AgentError;
use crate::skills::bundled_skills::{
    BundledSkillDefinition, ContentBlock, SkillContext, register_bundled_skill,
};

#[allow(dead_code)]
const DEFAULT_DEBUG_LINES_READ: usize = 20;
#[allow(dead_code)]
const TAIL_READ_BYTES: usize = 64 * 1024;

fn get_prompt_for_command(
    args: &str,
    _context: &SkillContext,
) -> Result<Vec<ContentBlock>, AgentError> {
    // TODO: Implement debug log reading when logging is integrated
    // For now, just provide the prompt template

    let prompt = format!(
        r#"# Debug Skill

Help the user debug an issue they're encountering in this current session.

## Debug Logging

Debug logging can be enabled to capture session activity.

## Issue Description

{}

## Instructions

1. Review the user's issue description
2. Look for errors, warnings, and failure patterns
3. Explain what you found in plain language
4. Suggest concrete fixes or next steps
"#,
        if args.is_empty() {
            "The user did not describe a specific issue.".to_string()
        } else {
            args.to_string()
        }
    );

    Ok(vec![ContentBlock::Text { text: prompt }])
}

pub fn register_debug_skill() {
    let _ = register_bundled_skill(BundledSkillDefinition {
        name: "debug".to_string(),
        description: "Enable debug logging for this session and help diagnose issues".to_string(),
        aliases: None,
        when_to_use: None,
        argument_hint: Some("[issue description]".to_string()),
        allowed_tools: Some(vec![
            "Read".to_string(),
            "Grep".to_string(),
            "Glob".to_string(),
        ]),
        model: None,
        disable_model_invocation: Some(true),
        user_invocable: Some(true),
        is_enabled: None,
        context: None,
        agent: None,
        files: None,
        get_prompt_for_command,
    });
}
