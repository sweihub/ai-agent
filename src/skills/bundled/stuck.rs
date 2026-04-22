// Source: /data/home/swei/claudecode/openclaudecode/src/skills/bundled/stuck.ts
//! Stuck skill - ported from openclaudecode/src/skills/bundled/stuck.ts
//!
//! Help when stuck on a problem.

use crate::AgentError;
use crate::skills::bundled_skills::{
    BundledSkillDefinition, ContentBlock, SkillContext, register_bundled_skill,
};

const STUCK_PROMPT: &str = r#"# Stuck Skill

Help when you're stuck on a problem.

## Strategies

1. **Take a break** - Step away and come back with fresh eyes
2. **Explain the problem** - Articulate what's blocking you
3. **Simplify** - Break the problem into smaller pieces
4. **Search** - Look for similar problems/solutions
5. **Ask for hints** - Request guidance without full solution

## What to Share

- What you've tried
- What you expected to happen
- What actually happened
- Any error messages

## Don't

- Don't ask for the full solution immediately
- Don't keep spinning on the same approach
"#;

fn get_prompt_for_command(
    args: &str,
    _context: &SkillContext,
) -> Result<Vec<ContentBlock>, AgentError> {
    let prompt = if args.is_empty() {
        STUCK_PROMPT.to_string()
    } else {
        format!("{}\n\n## Your Situation\n\n{}", STUCK_PROMPT, args)
    };
    Ok(vec![ContentBlock::Text { text: prompt }])
}

pub fn register_stuck_skill() {
    let _ = register_bundled_skill(BundledSkillDefinition {
        name: "stuck".to_string(),
        description: "Help when you're stuck on a problem".to_string(),
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
