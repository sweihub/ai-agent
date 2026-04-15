// Source: /data/home/swei/claudecode/openclaudecode/src/skills/bundled/dream.ts
//! Dream skill - ported from openclaudecode/src/skills/bundled/dream.ts
//!
//! Dream/creative mode skill (feature-gated: KAIROS or KAIROS_DREAM).

use crate::skills::bundled_skills::{
    register_bundled_skill, BundledSkillDefinition, ContentBlock, SkillContext,
};
use crate::AgentError;

const DREAM_PROMPT: &str = r#"# Dream Skill

Creative exploration mode.
"#;

fn get_prompt_for_command(
    args: &str,
    _context: &SkillContext,
) -> Result<Vec<ContentBlock>, AgentError> {
    let prompt = if args.is_empty() {
        DREAM_PROMPT.to_string()
    } else {
        format!("{}\n\n## Request\n\n{}", DREAM_PROMPT, args)
    };
    Ok(vec![ContentBlock::Text { text: prompt }])
}

pub fn register_dream_skill() {
    let _ = register_bundled_skill(BundledSkillDefinition {
        name: "dream".to_string(),
        description: "Creative exploration mode".to_string(),
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
