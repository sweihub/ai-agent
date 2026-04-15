// Source: /data/home/swei/claudecode/openclaudecode/src/utils/user.ts
//! User utilities module.
//! Provides user data and analytics functionality.

use crate::constants::env::ai;
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// GitHub Actions metadata when running in CI
#[derive(Debug, Clone, Default)]
pub struct GitHubActionsMetadata {
    pub actor: Option<String>,
    pub actor_id: Option<String>,
    pub repository: Option<String>,
    pub repository_id: Option<String>,
    pub repository_owner: Option<String>,
    pub repository_owner_id: Option<String>,
}

/// Core user data used as base for all analytics providers.
#[derive(Debug, Clone)]
pub struct CoreUserData {
    pub device_id: String,
    pub session_id: String,
    pub email: Option<String>,
    pub app_version: String,
    pub platform: String,
    pub organization_uuid: Option<String>,
    pub account_uuid: Option<String>,
    pub user_type: Option<String>,
    pub subscription_type: Option<String>,
    pub rate_limit_tier: Option<String>,
    pub first_token_time: Option<i64>,
    pub github_actions_metadata: Option<GitHubActionsMetadata>,
}

/// Platform type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOS,
    Linux,
    Windows,
    Unknown,
}

impl Platform {
    pub fn from_str(s: &str) -> Self {
        match s {
            "darwin" => Platform::MacOS,
            "linux" => Platform::Linux,
            "win32" => Platform::Windows,
            _ => Platform::Unknown,
        }
    }
}

// Cache for user data
static CORE_USER_DATA: Lazy<Mutex<Option<CoreUserData>>> = Lazy::new(|| Mutex::new(None));

/// Reset all user data caches
pub fn reset_user_cache() {
    let mut data = CORE_USER_DATA.lock().unwrap();
    *data = None;
}

/// Get core user data
pub fn get_core_user_data(_include_analytics_metadata: bool) -> CoreUserData {
    // Basic implementation - returns default values
    // Full implementation would read from config, auth, etc.
    CoreUserData {
        device_id: String::new(),
        session_id: String::new(),
        email: None,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        organization_uuid: None,
        account_uuid: None,
        user_type: std::env::var(ai::USER_TYPE).ok(),
        subscription_type: None,
        rate_limit_tier: None,
        first_token_time: None,
        github_actions_metadata: None,
    }
}

/// Get user data for analytics
pub fn get_user_for_analytics() -> CoreUserData {
    get_core_user_data(true)
}

/// Get user's git email from git config
pub fn get_git_email() -> Option<String> {
    // Execute git config --get user.email
    std::process::Command::new("git")
        .args(["config", "--get", "user.email"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                let email = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !email.is_empty() {
                    return Some(email);
                }
            }
            None
        })
}

/// Set cached email
pub fn set_cached_email(_email: Option<String>) {
    // Stub for now
}
