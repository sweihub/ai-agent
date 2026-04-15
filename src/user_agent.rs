//! User-Agent string helpers.
//!
//! Kept dependency-free so SDK-bundled code (bridge, cli/transports) can
//! import without pulling in auth.ts and its transitive dependency tree.

use crate::MACRO_VERSION;

/// Get the Claude Code User-Agent string.
pub fn get_claude_code_user_agent() -> String {
    format!("claude-code/{}", MACRO_VERSION)
}
