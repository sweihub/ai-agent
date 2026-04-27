//! Run Skill Generator skill - ported from openclaudecode/src/skills/bundled/runSkillGenerator.ts
//!
//! Generate new skills (feature-gated: RUN_SKILL_GENERATOR).

use crate::AgentError;
use crate::skills::bundled_skills::{
    BundledSkillDefinition, ContentBlock, SkillContext, register_bundled_skill,
};

const RUN_SKILL_GENERATOR_PROMPT: &str = r#"# Run Skill Generator

Generate a new skill from a template.
"#;

fn get_prompt_for_command(
    args: &str,
    _context: &SkillContext,
) -> Result<Vec<ContentBlock>, AgentError> {
    let prompt = if args.is_empty() {
        RUN_SKILL_GENERATOR_PROMPT.to_string()
    } else {
        format!("{}\n\n## Request\n\n{}", RUN_SKILL_GENERATOR_PROMPT, args)
    };
    Ok(vec![ContentBlock::Text { text: prompt }])
}

pub fn register_run_skill_generator_skill() {
    let _ = register_bundled_skill(BundledSkillDefinition {
        name: "run-skill-generator".to_string(),
        description: "Generate a new skill from a template".to_string(),
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
