// Source: ~/claudecode/openclaudecode/src/utils/telemetry/skillLoadedEvent.ts

/// Logs a skill_loaded event for each skill available at session startup.
/// This enables analytics on which skills are available across sessions.
pub async fn log_skills_loaded(
    cwd: &str,
    context_window_tokens: usize,
) {
    let skills = get_skill_tool_commands(cwd).await;
    let skill_budget = get_char_budget(context_window_tokens);

    for skill in skills {
        if skill.skill_type != "prompt" {
            continue;
        }

        // Log skill loaded event
        log_event("skill_loaded", &[
            ("skill_name", &skill.name),
            ("skill_source", &skill.source),
            ("skill_loaded_from", &skill.loaded_from),
            ("skill_budget", &skill_budget.to_string()),
        ]);

        if let Some(kind) = &skill.kind {
            log_event("skill_loaded", &[("skill_kind", kind)]);
        }
    }
}

fn log_event(_event_name: &str, _attributes: &[(&str, &str)]) {
    // In production, this would emit to the analytics system
    // For now, log via tracing
    tracing::debug!(event_name = _event_name, ?_attributes, "skill event");
}

/// A skill tool command.
#[derive(Debug, Clone)]
pub struct SkillToolCommand {
    pub name: String,
    pub source: String,
    pub loaded_from: String,
    pub skill_type: String,
    pub kind: Option<String>,
}

/// Get skill tool commands for a directory.
async fn get_skill_tool_commands(_cwd: &str) -> Vec<SkillToolCommand> {
    // In production, this would scan the skills directory
    Vec::new()
}

/// Get character budget from token count.
fn get_char_budget(context_window_tokens: usize) -> usize {
    // Rough estimate: 1 token ~ 4 chars
    context_window_tokens * 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_char_budget() {
        assert_eq!(get_char_budget(1000), 4000);
        assert_eq!(get_char_budget(0), 0);
    }

    #[tokio::test]
    async fn test_log_skills_loaded_empty() {
        log_skills_loaded("/tmp", 200_000).await;
        // Should not panic
    }
}
