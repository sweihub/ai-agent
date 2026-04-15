// Source: /data/home/swei/claudecode/openclaudecode/src/skills/bundled/batch.ts
//! Batch skill - ported from openclaudecode/src/skills/bundled/batch.ts
//!
//! Process multiple items in batch.

use crate::skills::bundled_skills::{
    register_bundled_skill, BundledSkillDefinition, ContentBlock, SkillContext,
};
use crate::AgentError;

const BATCH_PROMPT: &str = r#"# Batch Skill

Process multiple items in batch efficiently.

## Guidelines

1. Process items in parallel when possible
2. Handle errors gracefully per item
3. Report progress and results
4. Batch similar operations together

## Best Practices

- Group by operation type
- Use concurrent processing
- Track success/failure per item
"#;

fn get_prompt_for_command(
    args: &str,
    _context: &SkillContext,
) -> Result<Vec<ContentBlock>, AgentError> {
    let mut prompt = BATCH_PROMPT.to_string();
    if !args.is_empty() {
        prompt.push_str("\n\n## Items to Process\n\n");
        prompt.push_str(args);
    }
    Ok(vec![ContentBlock::Text { text: prompt }])
}

pub fn register_batch_skill() {
    let _ = register_bundled_skill(BundledSkillDefinition {
        name: "batch".to_string(),
        description: "Process multiple items in batch".to_string(),
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
