// Source: /data/home/swei/claudecode/openclaudecode/src/skills/bundled/hunter.ts
//! Hunter skill - ported from openclaudecode/src/skills/bundled/hunter.ts
//!
//! Review artifacts (feature-gated: REVIEW_ARTIFACT).

use crate::skills::bundled_skills::{
    register_bundled_skill, BundledSkillDefinition, ContentBlock, SkillContext,
};
use crate::AgentError;

const HUNTER_PROMPT: &str = r#"# Hunter Skill

Review and analyze artifacts.
"#;

fn get_prompt_for_command(
    args: &str,
    _context: &SkillContext,
) -> Result<Vec<ContentBlock>, AgentError> {
    let prompt = if args.is_empty() {
        HUNTER_PROMPT.to_string()
    } else {
        format!("{}\n\n## Request\n\n{}", HUNTER_PROMPT, args)
    };
    Ok(vec![ContentBlock::Text { text: prompt }])
}

pub fn register_hunter_skill() {
    let _ = register_bundled_skill(BundledSkillDefinition {
        name: "hunter".to_string(),
        description: "Review and analyze artifacts".to_string(),
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
