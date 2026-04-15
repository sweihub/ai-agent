//! Release notes utilities.

use serde::{Deserialize, Serialize};

/// Release note entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseNote {
    pub version: String,
    pub date: String,
    pub changes: Vec<Change>,
}

/// Type of change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Changed,
    Deprecated,
    Removed,
    Fixed,
    Security,
}

/// A single change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub change_type: ChangeType,
    pub description: String,
}

/// Parse release notes from markdown
pub fn parse_release_notes(markdown: &str) -> Vec<ReleaseNote> {
    let mut notes = Vec::new();

    // Simple parsing - look for version headers
    for line in markdown.lines() {
        if line.starts_with("## ") && line.contains("v") {
            // Extract version
            let version = line
                .trim_start_matches("## ")
                .trim_start_matches("v")
                .split_whitespace()
                .next()
                .unwrap_or("0.0.0")
                .to_string();

            notes.push(ReleaseNote {
                version,
                date: String::new(),
                changes: Vec::new(),
            });
        }
    }

    notes
}
