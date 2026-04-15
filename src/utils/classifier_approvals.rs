// Source: ~/claudecode/openclaudecode/src/utils/classifierApprovals.ts
//! Tracks which tool uses were auto-approved by classifiers.
//! Populated from use_can_use_tool and permissions, read from UserToolSuccessMessage.

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

#[derive(Debug, Clone)]
enum ClassifierKind {
    Bash { matched_rule: String },
    AutoMode { reason: String },
}

#[derive(Debug, Clone)]
struct ClassifierApproval {
    classifier: ClassifierKind,
}

static CLASSIFIER_APPROVALS: Mutex<HashMap<String, ClassifierApproval>> =
    Mutex::new(HashMap::new());
static CLASSIFIER_CHECKING: Mutex<HashSet<String>> = Mutex::new(HashSet::new());

/// Set a classifier approval for a bash classifier match.
/// Only active when BASH_CLASSIFIER feature is enabled.
pub fn set_classifier_approval(tool_use_id: &str, matched_rule: &str) {
    if !is_bash_classifier_enabled() && !is_transcript_classifier_enabled() {
        return;
    }
    let mut approvals = CLASSIFIER_APPROVALS.lock().unwrap();
    approvals.insert(
        tool_use_id.to_string(),
        ClassifierApproval {
            classifier: ClassifierKind::Bash {
                matched_rule: matched_rule.to_string(),
            },
        },
    );
}

/// Get the classifier approval rule for a tool use ID.
/// Returns None if no bash classifier approval exists.
pub fn get_classifier_approval(tool_use_id: &str) -> Option<String> {
    if !is_bash_classifier_enabled() && !is_transcript_classifier_enabled() {
        return None;
    }
    let approvals = CLASSIFIER_APPROVALS.lock().unwrap();
    let approval = approvals.get(tool_use_id)?;
    match &approval.classifier {
        ClassifierKind::Bash { matched_rule } => Some(matched_rule.clone()),
        _ => None,
    }
}

/// Set a yolo classifier approval for auto-mode classifier match.
/// Only active when TRANSCRIPT_CLASSIFIER feature is enabled.
pub fn set_yolo_classifier_approval(tool_use_id: &str, reason: &str) {
    if !is_transcript_classifier_enabled() {
        return;
    }
    let mut approvals = CLASSIFIER_APPROVALS.lock().unwrap();
    approvals.insert(
        tool_use_id.to_string(),
        ClassifierApproval {
            classifier: ClassifierKind::AutoMode {
                reason: reason.to_string(),
            },
        },
    );
}

/// Get the yolo classifier approval reason for a tool use ID.
/// Returns None if no auto-mode classifier approval exists.
pub fn get_yolo_classifier_approval(tool_use_id: &str) -> Option<String> {
    if !is_transcript_classifier_enabled() {
        return None;
    }
    let approvals = CLASSIFIER_APPROVALS.lock().unwrap();
    let approval = approvals.get(tool_use_id)?;
    match &approval.classifier {
        ClassifierKind::AutoMode { reason } => Some(reason.clone()),
        _ => None,
    }
}

/// Mark a tool use ID as currently being checked by a classifier.
pub fn set_classifier_checking(tool_use_id: &str) {
    if !is_bash_classifier_enabled() && !is_transcript_classifier_enabled() {
        return;
    }
    CLASSIFIER_CHECKING
        .lock()
        .unwrap()
        .insert(tool_use_id.to_string());
}

/// Clear the classifier checking status for a tool use ID.
pub fn clear_classifier_checking(tool_use_id: &str) {
    if !is_bash_classifier_enabled() && !is_transcript_classifier_enabled() {
        return;
    }
    CLASSIFIER_CHECKING
        .lock()
        .unwrap()
        .remove(tool_use_id);
}

/// Check if a tool use ID is currently being checked by a classifier.
pub fn is_classifier_checking(tool_use_id: &str) -> bool {
    CLASSIFIER_CHECKING
        .lock()
        .unwrap()
        .contains(tool_use_id)
}

/// Delete a classifier approval for a tool use ID.
pub fn delete_classifier_approval(tool_use_id: &str) {
    CLASSIFIER_APPROVALS
        .lock()
        .unwrap()
        .remove(tool_use_id);
}

/// Clear all classifier approvals and checking status.
pub fn clear_classifier_approvals() {
    CLASSIFIER_APPROVALS.lock().unwrap().clear();
    CLASSIFIER_CHECKING.lock().unwrap().clear();
}

/// Check if the BASH_CLASSIFIER feature is enabled.
fn is_bash_classifier_enabled() -> bool {
    std::env::var("AI_CODE_ENABLE_BASH_CLASSIFIER")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
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
    fn test_set_and_get_classifier_approval() {
        set_classifier_approval("tool-1", "safe-pattern");
        assert_eq!(
            get_classifier_approval("tool-1"),
            Some("safe-pattern".to_string())
        );
    }

    #[test]
    fn test_classifier_checking() {
        set_classifier_checking("tool-2");
        assert!(is_classifier_checking("tool-2"));
        clear_classifier_checking("tool-2");
        assert!(!is_classifier_checking("tool-2"));
    }

    #[test]
    fn test_clear_classifier_approvals() {
        set_classifier_approval("tool-3", "rule");
        clear_classifier_approvals();
        assert!(get_classifier_approval("tool-3").is_none());
    }

    #[test]
    fn test_delete_classifier_approval() {
        set_classifier_approval("tool-4", "rule");
        delete_classifier_approval("tool-4");
        assert!(get_classifier_approval("tool-4").is_none());
    }
}
