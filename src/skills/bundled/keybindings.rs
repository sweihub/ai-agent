// Source: /data/home/swei/claudecode/openclaudecode/src/commands/keybindings/keybindings.ts
//! Keybindings skill - ported from openclaudecode/src/skills/bundled/keybindings.ts
//!
//! Show keyboard shortcuts and keybindings.

use crate::AgentError;
use crate::skills::bundled_skills::{
    BundledSkillDefinition, ContentBlock, SkillContext, register_bundled_skill,
};

const KEYBINDINGS_PROMPT: &str = r#"# Keybindings Skill

Display available keyboard shortcuts and commands.

## Common Commands

- `/` - Command prefix for slash commands
- `Ctrl+C` - Cancel current operation
- `Ctrl+L` - Clear terminal
- `Ctrl+D` - Exit session

## Slash Commands

Use `/` to access these commands:
- `/help` - Show available commands
- `/compact` - Compact conversation
- `/clear` - Clear conversation/history
- `/resume` - Resume paused session
"#;

fn get_prompt_for_command(
    _args: &str,
    _context: &SkillContext,
) -> Result<Vec<ContentBlock>, AgentError> {
    Ok(vec![ContentBlock::Text {
        text: KEYBINDINGS_PROMPT.to_string(),
    }])
}

pub fn register_keybindings_skill() {
    let _ = register_bundled_skill(BundledSkillDefinition {
        name: "keybindings".to_string(),
        description: "Show keyboard shortcuts and keybindings".to_string(),
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
