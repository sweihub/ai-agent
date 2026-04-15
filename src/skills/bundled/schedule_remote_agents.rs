//! Schedule Remote Agents skill - ported from openclaudecode/src/skills/bundled/scheduleRemoteAgents.ts
//!
//! Schedule remote agents (feature-gated: AGENT_TRIGGERS_REMOTE).

use crate::skills::bundled_skills::{
    register_bundled_skill, BundledSkillDefinition, ContentBlock, SkillContext,
};
use crate::AgentError;

const SCHEDULE_PROMPT: &str = r#"# Schedule Remote Agents Skill

Schedule agents to run remotely on a cron schedule.
"#;

fn get_prompt_for_command(
    args: &str,
    _context: &SkillContext,
) -> Result<Vec<ContentBlock>, AgentError> {
    let prompt = if args.is_empty() {
        SCHEDULE_PROMPT.to_string()
    } else {
        format!("{}\n\n## Request\n\n{}", SCHEDULE_PROMPT, args)
    };
    Ok(vec![ContentBlock::Text { text: prompt }])
}

pub fn register_schedule_remote_agents_skill() {
    let _ = register_bundled_skill(BundledSkillDefinition {
        name: "schedule-remote-agents".to_string(),
        description: "Schedule agents to run remotely".to_string(),
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
