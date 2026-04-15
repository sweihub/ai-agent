use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AwaySummaryConfig {
    pub enabled: bool,
    pub blur_delay_ms: u64,
    pub min_turns_since_last_user: usize,
}

impl Default for AwaySummaryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            blur_delay_ms: 5 * 60 * 1000,
            min_turns_since_last_user: 1,
        }
    }
}

pub struct AwaySummaryState {
    is_enabled: bool,
    is_generating: bool,
    pending_summary: Option<String>,
    last_summary_turn: usize,
}

impl AwaySummaryState {
    pub fn new() -> Self {
        Self {
            is_enabled: true,
            is_generating: false,
            pending_summary: None,
            last_summary_turn: 0,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.is_enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn set_generating(&mut self, generating: bool) {
        self.is_generating = generating;
    }

    pub fn is_generating(&self) -> bool {
        self.is_generating
    }

    pub fn set_pending_summary(&mut self, summary: String) {
        self.pending_summary = Some(summary);
    }

    pub fn get_pending_summary(&self) -> Option<&String> {
        self.pending_summary.as_ref()
    }

    pub fn clear_pending_summary(&mut self) {
        self.pending_summary = None;
    }

    pub fn mark_summary_generated(&mut self, turn: usize) {
        self.last_summary_turn = turn;
        self.is_generating = false;
        self.pending_summary = None;
    }

    pub fn get_last_summary_turn(&self) -> usize {
        self.last_summary_turn
    }
}

impl Default for AwaySummaryState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn should_show_away_summary(
    config: &AwaySummaryConfig,
    state: &AwaySummaryState,
    is_blurred: bool,
    current_turn: usize,
) -> bool {
    if !config.enabled || !state.is_enabled() {
        return false;
    }

    if !is_blurred {
        return false;
    }

    if state.is_generating() {
        return false;
    }

    let turns_since_summary = current_turn.saturating_sub(state.get_last_summary_turn());
    if turns_since_summary < config.min_turns_since_last_user {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_away_summary_state() {
        let mut state = AwaySummaryState::new();

        assert!(state.is_enabled());

        state.set_enabled(false);
        assert!(!state.is_enabled());
    }

    #[test]
    fn test_pending_summary() {
        let mut state = AwaySummaryState::new();

        state.set_pending_summary("Test summary".to_string());
        assert!(state.get_pending_summary().is_some());

        state.clear_pending_summary();
        assert!(state.get_pending_summary().is_none());
    }

    #[test]
    fn test_should_show_away_summary() {
        let config = AwaySummaryConfig::default();
        let state = AwaySummaryState::new();

        assert!(!should_show_away_summary(&config, &state, false, 1));
    }
}
