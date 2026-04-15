//! Away summary service - generates a short session recap for the "while you were away" card.
//!
//! Translates awaySummary.ts from claude code.

/// Recap only needs recent context — truncate to avoid "prompt too long" on
/// large sessions. 30 messages ≈ ~15 exchanges, plenty for "where we left off."
pub const RECENT_MESSAGE_WINDOW: usize = 30;

/// Build the prompt for generating away summary
pub fn build_away_summary_prompt(memory: Option<&str>) -> String {
    let memory_block = memory
        .map(|m| format!("Session memory (broker context):\n{}\n\n", m))
        .unwrap_or_default();

    format!(
        "{}The user stepped away and is coming back. Write exactly 1-3 short sentences. Start by stating the high-level task — what they are building or debugging, not implementation details. Next: the concrete next step. Skip status reports and commit recaps.",
        memory_block
    )
}

/// Generate away summary result
#[derive(Debug, Clone)]
pub struct AwaySummaryResult {
    pub summary: Option<String>,
    pub was_aborted: bool,
}

impl AwaySummaryResult {
    pub fn aborted() -> Self {
        Self {
            summary: None,
            was_aborted: true,
        }
    }

    pub fn success(summary: String) -> Self {
        Self {
            summary: Some(summary),
            was_aborted: false,
        }
    }

    pub fn empty() -> Self {
        Self {
            summary: None,
            was_aborted: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_away_summary_prompt_with_memory() {
        let memory = "Working on the AI agent SDK";
        let prompt = build_away_summary_prompt(Some(memory));
        assert!(prompt.contains("Session memory"));
        assert!(prompt.contains(memory));
    }

    #[test]
    fn test_build_away_summary_prompt_without_memory() {
        let prompt = build_away_summary_prompt(None);
        assert!(!prompt.contains("Session memory"));
    }

    #[test]
    fn test_away_summary_result() {
        let result = AwaySummaryResult::success("Test summary".to_string());
        assert!(result.summary.is_some());
        assert!(!result.was_aborted);

        let result = AwaySummaryResult::aborted();
        assert!(result.summary.is_none());
        assert!(result.was_aborted);

        let result = AwaySummaryResult::empty();
        assert!(result.summary.is_none());
        assert!(!result.was_aborted);
    }
}
