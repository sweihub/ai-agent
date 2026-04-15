// Source: ~/claudecode/openclaudecode/src/utils/autoModeDenials.ts
//! Tracks commands recently denied by the auto mode classifier.
//! Populated from use_can_use_tool, read from RecentDenialsTab in /permissions.

#![allow(dead_code)]

use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct AutoModeDenial {
    pub tool_name: String,
    /// Human-readable description of the denied command (e.g. bash command string)
    pub display: String,
    pub reason: String,
    pub timestamp: u64,
}

thread_local! {
    static DENIALS: RefCell<Vec<AutoModeDenial>> = const { RefCell::new(Vec::new()) };
}

const MAX_DENIALS: usize = 20;

/// Record an auto mode denial. Only active when TRANSCRIPT_CLASSIFIER feature is enabled.
pub fn record_auto_mode_denial(denial: AutoModeDenial) {
    // Feature gate: TRANSCRIPT_CLASSIFIER
    if !is_transcript_classifier_enabled() {
        return;
    }
    DENIALS.with(|denials| {
        let mut d = denials.borrow_mut();
        d.insert(0, denial);
        if d.len() > MAX_DENIALS {
            d.truncate(MAX_DENIALS);
        }
    });
}

/// Get the current list of auto mode denials.
pub fn get_auto_mode_denials() -> Vec<AutoModeDenial> {
    DENIALS.with(|denials| denials.borrow().clone())
}

/// Check if the TRANSCRIPT_CLASSIFIER feature is enabled.
fn is_transcript_classifier_enabled() -> bool {
    std::env::var("AI_CODE_ENABLE_TRANSCRIPT_CLASSIFIER")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_auto_mode_denial() {
        let denial = AutoModeDenial {
            tool_name: "Bash".to_string(),
            display: "ls -la".to_string(),
            reason: "Command denied".to_string(),
            timestamp: 1234567890,
        };
        // May be a no-op if feature flag is not set
        record_auto_mode_denial(denial);
        // Just verify no panic
        let _denials = get_auto_mode_denials();
    }

    #[test]
    fn test_max_denials() {
        for i in 0..25 {
            let denial = AutoModeDenial {
                tool_name: "Bash".to_string(),
                display: format!("cmd-{i}"),
                reason: "denied".to_string(),
                timestamp: i,
            };
            record_auto_mode_denial(denial);
        }
        let denials = get_auto_mode_denials();
        assert!(denials.len() <= MAX_DENIALS);
    }
}
