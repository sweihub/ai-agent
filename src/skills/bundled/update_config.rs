//! Update Config skill - ported from openclaudecode/src/skills/bundled/updateConfig.ts
//!
//! Configure Claude Code harness via settings.json.

use crate::AgentError;
use crate::skills::bundled_skills::{
    BundledSkillDefinition, ContentBlock, SkillContext, register_bundled_skill,
};

const SETTINGS_EXAMPLES_DOCS: &str = r#"## Settings File Locations

Choose the appropriate file based on scope:

| File | Scope | Git | Use For |
|------|-------|-----|---------|
| `~/.ai/settings.json` | Global | N/A | Personal preferences for all projects |
| `.ai/settings.json` | Project | Commit | Team-wide hooks, permissions, plugins |
| `.ai/settings.local.json` | Project | Gitignore | Personal overrides for this project |

Settings load in order: user → project → local (later overrides earlier).

## Settings Schema Reference

### Permissions
```json
{
  "permissions": {
    "allow": ["Bash(npm:*)", "Edit(.ai)", "Read"],
    "deny": ["Bash(rm -rf:*)"],
    "ask": ["Write(/etc/*)"],
    "defaultMode": "default" | "plan" | "acceptEdits" | "dontAsk",
    "additionalDirectories": ["/extra/dir"]
  }
}
```

### Environment Variables
```json
{
  "env": {
    "DEBUG": "true",
    "MY_API_KEY": "value"
  }
}
```

### Model & Agent
```json
{
  "model": "sonnet",
  "agent": "agent-name",
  "alwaysThinkingEnabled": true
}
```
"#;

const HOOKS_DOCS: &str = r#"## Hooks Configuration

Hooks run commands at specific points in Claude Code's lifecycle.

### Hook Structure
```json
{
  "hooks": {
    "EVENT_NAME": [
      {
        "matcher": "ToolName|OtherTool",
        "hooks": [
          {
            "type": "command",
            "command": "your-command-here",
            "timeout": 60,
            "statusMessage": "Running..."
          }
        ]
      }
    ]
  }
}
```

### Hook Events

| Event | Matcher | Purpose |
|-------|---------|---------|
| PermissionRequest | Tool name | Run before permission prompt |
| PreToolUse | Tool name | Run before tool, can block |
| PostToolUse | Tool name | Run after successful tool |
| PostToolUseFailure | Tool name | Run after tool fails |
| Notification | Notification type | Run on notifications |
| Stop | - | Run when Claude stops |
| PreCompact | "manual"/"auto" | Before compaction |
| PostCompact | "manual"/"auto" | After compaction |
| UserPromptSubmit | - | When user submits |
| SessionStart | - | When session starts |

### Hook Types

**1. Command Hook** - Runs a shell command:
```json
{ "type": "command", "command": "prettier --write $FILE", "timeout": 30 }
```

**2. Prompt Hook** - Evaluates a condition with LLM:
```json
{ "type": "prompt", "prompt": "Is this safe? $ARGUMENTS" }
```

**3. Agent Hook** - Runs an agent with tools:
```json
{ "type": "agent", "prompt": "Verify tests pass: $ARGUMENTS" }
```
"#;

const UPDATE_CONFIG_PROMPT: &str = r#"# Update Config Skill

Modify Claude Code configuration by updating settings.json files.

## When Hooks Are Required (Not Memory)

If the user wants something to happen automatically in response to an EVENT, they need a **hook** configured in settings.json. Memory/preferences cannot trigger automated actions.

**These require hooks:**
- "Before compacting, ask me what to preserve" → PreCompact hook
- "After writing files, run prettier" → PostToolUse hook with Write|Edit matcher
- "When I run bash commands, log them" → PreToolUse hook with Bash matcher
- "Always run tests after code changes" → PostToolUse hook

**Hook events:** PreToolUse, PostToolUse, PreCompact, PostCompact, Stop, Notification, SessionStart

## CRITICAL: Read Before Write

**Always read the existing settings file before making changes.** Merge new settings with existing ones - never replace the entire file.

## CRITICAL: Use AskUserQuestion for Ambiguity

When the user's request is ambiguous, use AskUserQuestion to clarify:
- Which settings file to modify (user/project/local)
- Whether to add to existing arrays or replace them
- Specific values when multiple options exist

## Workflow

1. **Clarify intent** - Ask if the request is ambiguous
2. **Read existing file** - Use Read tool on the target settings file
3. **Merge carefully** - Preserve existing settings, especially arrays
4. **Edit file** - Use Edit tool (if file doesn't exist, ask user to create it first)
5. **Confirm** - Tell user what was changed

## Merging Arrays (Important!)

When adding to permission arrays or hook arrays, **merge with existing**, don't replace.

"#;

fn get_prompt_for_command(
    args: &str,
    _context: &SkillContext,
) -> Result<Vec<ContentBlock>, AgentError> {
    let mut prompt = UPDATE_CONFIG_PROMPT.to_string();
    prompt.push_str(SETTINGS_EXAMPLES_DOCS);
    prompt.push_str("\n\n");
    prompt.push_str(HOOKS_DOCS);

    if !args.is_empty() {
        prompt.push_str("\n\n## User Request\n\n");
        prompt.push_str(args);
    }

    Ok(vec![ContentBlock::Text { text: prompt }])
}

pub fn register_update_config_skill() {
    let _ = register_bundled_skill(BundledSkillDefinition {
        name: "update-config".to_string(),
        description: "Configure Claude Code harness via settings.json. Automated behaviors require hooks - memory/preferences cannot trigger automated actions. Also use for: permissions, env vars, hook troubleshooting, or any changes to settings.json.".to_string(),
        aliases: None,
        when_to_use: None,
        argument_hint: None,
        allowed_tools: Some(vec!["Read".to_string()]),
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
