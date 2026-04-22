// Source: /data/home/swei/claudecode/openclaudecode/src/skills/bundled/skillify.ts
//! Skillify skill - ported from openclaudecode/src/skills/bundled/skillify.ts
//!
//! Convert a pattern or workflow into a reusable skill.

use crate::AgentError;
use crate::skills::bundled_skills::{
    BundledSkillDefinition, ContentBlock, SkillContext, register_bundled_skill,
};

const SKILLIFY_PROMPT: &str = r#"# Skillify Skill

Convert a pattern or workflow into a reusable skill.

## Creating a Skill

A skill is a reusable prompt that can be invoked with `/skillname`.

### Skill Structure

```markdown
---
name: skill-name
description: "What this skill does"
---

# Skill Name

Detailed instructions for the skill...

## Parameters

- param1: description
- param2: description
```

### Where to Save

- Personal skills: `~/.ai/skills/`
- Project skills: `.ai/skills/`

## Steps

1. **Extract the pattern** - Identify the recurring workflow
2. **Create the prompt** - Write clear, reusable instructions
3. **Add metadata** - Name, description, parameters
4. **Save to skills directory**
"#;

fn get_prompt_for_command(
    args: &str,
    _context: &SkillContext,
) -> Result<Vec<ContentBlock>, AgentError> {
    let mut prompt = SKILLIFY_PROMPT.to_string();
    if !args.is_empty() {
        prompt.push_str("\n\n## Request\n\n");
        prompt.push_str(args);
    }
    Ok(vec![ContentBlock::Text { text: prompt }])
}

pub fn register_skillify_skill() {
    let _ = register_bundled_skill(BundledSkillDefinition {
        name: "skillify".to_string(),
        description: "Convert a pattern or workflow into a reusable skill".to_string(),
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
