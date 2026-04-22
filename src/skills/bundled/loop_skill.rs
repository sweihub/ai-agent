//! Loop skill - ported from openclaudecode/src/skills/bundled/loop.ts
//!
//! Run a prompt or slash command on a recurring interval.

use crate::AgentError;
use crate::skills::bundled_skills::{
    BundledSkillDefinition, ContentBlock, SkillContext, register_bundled_skill,
};

const LOOP_PROMPT: &str = r#"# Loop Skill

Run a prompt or slash command on a recurring interval.

## Usage

/loop <interval> <command>

## Examples

- `/loop 5m /foo` - Run /foo every 5 minutes
- `/loop 10m /check` - Run /check every 10 minutes

## Intervals

- `m` - minutes
- `h` - hours
- `d` - days
"#;

fn get_prompt_for_command(
    args: &str,
    _context: &SkillContext,
) -> Result<Vec<ContentBlock>, AgentError> {
    let prompt = if args.is_empty() {
        LOOP_PROMPT.to_string()
    } else {
        format!("{}\n\n## Request\n\n{}", LOOP_PROMPT, args)
    };
    Ok(vec![ContentBlock::Text { text: prompt }])
}

pub fn register_loop_skill() {
    let _ = register_bundled_skill(BundledSkillDefinition {
        name: "loop".to_string(),
        description: "Run a prompt or slash command on a recurring interval".to_string(),
        aliases: None,
        when_to_use: None,
        argument_hint: Some("<interval> <command>".to_string()),
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
