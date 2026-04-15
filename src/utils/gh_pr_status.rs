//! GitHub PR status utilities
//!
//! Fetch PR status for the current branch using `gh pr view`.

use serde::{Deserialize, Serialize};

/// PR review state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PrReviewState {
    Approved,
    Pending,
    ChangesRequested,
    Draft,
    Merged,
    Closed,
}

/// PR status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrStatus {
    /// PR number
    pub number: u32,
    /// PR URL
    pub url: String,
    /// Review state
    pub review_state: PrReviewState,
}

/// GH timeout in milliseconds
const GH_TIMEOUT_MS: u64 = 5000;

/// Derive review state from GitHub API values.
///
/// Draft PRs always show as 'draft' regardless of reviewDecision.
/// reviewDecision can be: APPROVED, CHANGES_REQUESTED, REVIEW_REQUIRED, or empty string.
///
/// # Arguments
/// * `is_draft` - Whether the PR is a draft
/// * `review_decision` - The review decision string from GitHub API
///
/// # Returns
/// The derived PR review state
pub fn derive_review_state(is_draft: bool, review_decision: &str) -> PrReviewState {
    if is_draft {
        return PrReviewState::Draft;
    }

    match review_decision {
        "APPROVED" => PrReviewState::Approved,
        "CHANGES_REQUESTED" => PrReviewState::ChangesRequested,
        _ => PrReviewState::Pending,
    }
}

/// Fetch PR status for the current branch using `gh pr view`.
///
/// Returns None on any failure (gh not installed, no PR, not in git repo, etc).
/// Also returns None if the PR's head branch is the default branch (e.g., main/master).
///
/// # Returns
/// Some(PrStatus) if PR found, None otherwise
pub async fn fetch_pr_status() -> Option<PrStatus> {
    use std::process::Command;

    // Check if in a git repository
    let is_git = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !is_git {
        return None;
    }

    // Get current branch and default branch in parallel
    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;
    let branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();

    let default_branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "origin/HEAD"])
        .output()
        .ok();
    let default_branch = default_branch_output
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .trim_start_matches("origin/")
                .to_string()
        })
        .unwrap_or_else(|| "main".to_string());

    // Skip on the default branch
    if branch == default_branch || branch == "main" || branch == "master" {
        return None;
    }

    // Run gh pr view
    let output = Command::new("gh")
        .args([
            "pr",
            "view",
            "--json",
            "number,url,reviewDecision,isDraft,headRefName,state",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        return None;
    }

    // Parse JSON response
    let data: serde_json::Value = serde_json::from_str(&stdout).ok()?;

    let number = data["number"].as_u64()? as u32;
    let url = data["url"].as_str()?.to_string();
    let is_draft = data["isDraft"].as_bool().unwrap_or(false);
    let review_decision = data["reviewDecision"].as_str().unwrap_or("");
    let head_ref = data["headRefName"].as_str().unwrap_or("");
    let state = data["state"].as_str().unwrap_or("");

    // Don't show PR status for PRs from the default branch
    if head_ref == default_branch || head_ref == "main" || head_ref == "master" {
        return None;
    }

    // Don't show PR status for merged or closed PRs
    if state == "MERGED" || state == "CLOSED" {
        return None;
    }

    Some(PrStatus {
        number,
        url,
        review_state: derive_review_state(is_draft, review_decision),
    })
}
