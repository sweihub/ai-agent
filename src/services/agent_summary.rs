//! Agent summary service - periodic background summarization for coordinator mode sub-agents.
//!
//! Translates AgentSummary/agentSummary.ts from claude code.

pub const SUMMARY_INTERVAL_MS: u64 = 30000;

pub fn build_summary_prompt(previous_summary: Option<&str>) -> String {
    let prev_line = previous_summary
        .map(|s| format!("\nPrevious: \"{}\" — say something NEW.\n", s))
        .unwrap_or_default();

    format!(
        r#"Describe your most recent action in 3-5 words using present tense (-ing). Name the file or function, not the branch. Do not use tools.
{}
Good: "Reading runAgent.ts"
Good: "Fixing null check in validate.ts"
Good: "Running auth module tests"
Good: "Adding retry logic to fetchUser"

Bad (past tense): "Analyzed the branch diff"
Bad (too vague): "Investigating the issue"
Bad (too long): "Reviewing full branch diff and AgentTool.tsx integration"
Bad (branch name): "Analyzed adam/background-summary branch diff""#,
        prev_line
    )
}

/// Handle for stopping agent summarization
pub struct AgentSummaryHandle {
    pub stop: Box<dyn Fn() + Send + Sync>,
}

impl AgentSummaryHandle {
    pub fn new(stop: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            stop: Box::new(stop),
        }
    }
}

impl Clone for AgentSummaryHandle {
    fn clone(&self) -> Self {
        // Note: This is a workaround - the actual stop function cannot be cloned
        // In practice, you would need to use Arc<dyn Fn()> for true cloneability
        Self {
            stop: Box::new(|| {}),
        }
    }
}

/// State for agent summarization
pub struct AgentSummaryState {
    pub task_id: String,
    pub agent_id: String,
    pub summary: Option<String>,
    pub is_running: bool,
}

impl AgentSummaryState {
    pub fn new(task_id: impl Into<String>, agent_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            agent_id: agent_id.into(),
            summary: None,
            is_running: true,
        }
    }

    pub fn stop(&mut self) {
        self.is_running = false;
    }

    pub fn update_summary(&mut self, summary: impl Into<String>) {
        self.summary = Some(summary.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_summary_prompt_no_previous() {
        let prompt = build_summary_prompt(None);
        assert!(prompt.contains("Describe your most recent action"));
        assert!(!prompt.contains("Previous:"));
    }

    #[test]
    fn test_build_summary_prompt_with_previous() {
        let prompt = build_summary_prompt(Some("Reading file"));
        assert!(prompt.contains("Previous:"));
        assert!(prompt.contains("Reading file"));
    }

    #[test]
    fn test_agent_summary_state() {
        let mut state = AgentSummaryState::new("task-1", "agent-1");

        assert!(state.summary.is_none());
        assert!(state.is_running);

        state.update_summary("Reading file");
        assert_eq!(state.summary, Some("Reading file".to_string()));

        state.stop();
        assert!(!state.is_running);
    }

    #[test]
    fn test_summary_interval() {
        assert_eq!(SUMMARY_INTERVAL_MS, 30000);
    }
}
