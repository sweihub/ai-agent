//! Lorem Ipsum skill - ported from openclaudecode/src/skills/bundled/loremIpsum.ts
//!
//! Generate placeholder text.

use crate::skills::bundled_skills::{
    register_bundled_skill, BundledSkillDefinition, ContentBlock, SkillContext,
};
use crate::AgentError;

const LOREM_IPSUM_PROMPT: &str = r#"# Lorem Ipsum Skill

Generate placeholder text for design and testing.

## Usage

Generate lorem ipsum text with specified length:
- Short: 1-2 sentences
- Medium: 1 paragraph
- Long: multiple paragraphs

## Example

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
"#;

fn get_prompt_for_command(
    _args: &str,
    _context: &SkillContext,
) -> Result<Vec<ContentBlock>, AgentError> {
    Ok(vec![ContentBlock::Text {
        text: LOREM_IPSUM_PROMPT.to_string(),
    }])
}

pub fn register_lorem_ipsum_skill() {
    let _ = register_bundled_skill(BundledSkillDefinition {
        name: "lorem-ipsum".to_string(),
        description: "Generate placeholder text".to_string(),
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
