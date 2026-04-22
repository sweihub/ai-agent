// Source: /data/home/swei/claudecode/openclaudecode/src/skills/bundled/remember.ts
//! Remember skill - ported from openclaudecode/src/skills/bundled/remember.ts
//!
//! Remember information across sessions.

use crate::AgentError;
use crate::skills::bundled_skills::{
    BundledSkillDefinition, ContentBlock, SkillContext, register_bundled_skill,
};

const REMEMBER_PROMPT: &str = r#"# Remember Skill

Remember information across sessions by saving to memory.

## Usage

The memory system stores:
- **user** - User profile, preferences, expertise level
- **feedback** - Guidance on how to approach work, what to avoid
- **project** - Project-specific context, goals, deadlines
- **reference** - Pointers to external systems and resources

## How to Save

1. Write memory to file in `memory/` directory
2. Add pointer to `MEMORY.md` index

## Memory Structure

```markdown
---
name: memory-name
description: one-line description
type: user | feedback | project | reference
---

Memory content with **Why:** and **How to apply:** lines.
```
"#;

fn get_prompt_for_command(
    args: &str,
    _context: &SkillContext,
) -> Result<Vec<ContentBlock>, AgentError> {
    let mut prompt = REMEMBER_PROMPT.to_string();
    if !args.is_empty() {
        prompt.push_str("\n\n## User Request\n\n");
        prompt.push_str(args);
    }
    Ok(vec![ContentBlock::Text { text: prompt }])
}

pub fn register_remember_skill() {
    let _ = register_bundled_skill(BundledSkillDefinition {
        name: "remember".to_string(),
        description: "Remember information across sessions using the memory system".to_string(),
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
