//! User-Agent string helpers (legacy re-exports for backwards compat).

pub use crate::utils::user_agent::get_user_agent;

/// Backwards-compatible alias for the unified user agent.
pub fn get_claude_code_user_agent() -> String {
    get_user_agent()
}
