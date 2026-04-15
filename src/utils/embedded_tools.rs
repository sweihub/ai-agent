// Source: ~/claudecode/openclaudecode/src/utils/embeddedTools.ts
//! Whether this build has bfs/ugrep embedded in the bun binary (ant-native only).
//!
//! When true:
//! - `find` and `grep` in Claude's Bash shell are shadowed by shell functions
//!   that invoke the bun binary with argv0='bfs' / argv0='ugrep' (same trick
//!   as embedded ripgrep)
//! - The dedicated Glob/Grep tools are removed from the tool registry
//! - Prompt guidance steering Claude away from find/grep is omitted
//!
//! Set as a build-time define in scripts/build-with-plugins for ant-native builds.

#![allow(dead_code)]

/// Whether this build has bfs/ugrep embedded in the bun binary.
pub fn has_embedded_search_tools() -> bool {
    if !is_env_truthy("EMBEDDED_SEARCH_TOOLS") {
        return false;
    }
    let entrypoint = std::env::var("AI_CODE_ENTRYPOINT").unwrap_or_default();
    !matches!(
        entrypoint.as_str(),
        "sdk-ts" | "sdk-py" | "sdk-cli" | "local-agent"
    )
}

/// Path to the bun binary that contains the embedded search tools.
/// Only meaningful when has_embedded_search_tools() is true.
pub fn embedded_search_tools_binary_path() -> String {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Check if an environment variable is truthy.
fn is_env_truthy(key: &str) -> bool {
    std::env::var(key)
        .map(|v| v == "1" || v.to_lowercase() == "true" || v.to_lowercase() == "yes")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_embedded_search_tools_default() {
        // Without EMBEDDED_SEARCH_TOOLS set, should be false
        assert!(!has_embedded_search_tools());
    }

    #[test]
    fn test_is_env_truthy() {
        // These test the helper function logic
        assert!(is_env_truthy_helper(Some("1")));
        assert!(is_env_truthy_helper(Some("true")));
        assert!(is_env_truthy_helper(Some("yes")));
        assert!(!is_env_truthy_helper(Some("0")));
        assert!(!is_env_truthy_helper(Some("false")));
        assert!(!is_env_truthy_helper(None));
    }

    fn is_env_truthy_helper(val: Option<&str>) -> bool {
        match val {
            Some(v) => matches!(v, "1" | "true" | "yes")
                || v.to_lowercase() == "true"
                || v.to_lowercase() == "yes",
            None => false,
        }
    }
}
