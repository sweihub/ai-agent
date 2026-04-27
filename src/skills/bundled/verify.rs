// Source: /data/home/swei/claudecode/openclaudecode/src/skills/bundled/verify.ts
//! Verify skill - ported from openclaudecode/src/skills/bundled/verify.ts
//!
//! Verify a code change does what it should by running the app.

use crate::AgentError;
use crate::skills::bundled_skills::{
    BundledSkillDefinition, ContentBlock, SkillContext, register_bundled_skill,
};

const VERIFY_PROMPT: &str = r#"# Verify

Verify a code change does what it should by running the app.

## Instructions

1. Understand what the user is trying to verify
2. Run the appropriate tests or commands
3. Report the results clearly
4. If something fails, help diagnose and fix
"#;

fn get_prompt_for_command(
    args: &str,
    _context: &SkillContext,
) -> Result<Vec<ContentBlock>, AgentError> {
    let mut prompt = VERIFY_PROMPT.to_string();
    if !args.is_empty() {
        prompt.push_str("\n\n## User Request\n\n");
        prompt.push_str(args);
    }
    Ok(vec![ContentBlock::Text { text: prompt }])
}

pub fn register_verify_skill() {
    let _ = register_bundled_skill(BundledSkillDefinition {
        name: "verify".to_string(),
        description: "Verify a code change does what it should by running the app.".to_string(),
        aliases: None,
        when_to_use: None,
        argument_hint: None,
        allowed_tools: None,
        model: None,
        disable_model_invocation: None,
        user_invocable: Some(true),
        is_enabled: None,
        hooks: None,
        context: None,
        agent: None,
        files: None,
        get_prompt_for_command: std::sync::Arc::new(get_prompt_for_command),
    });
}
