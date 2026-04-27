// Source: ~/claudecode/openclaudecode/src/tools/REPLTool/constants.ts
use std::collections::HashSet;
use std::sync::LazyLock;

pub const REPL_TOOL_NAME: &str = "REPL";

/// REPL mode is default-on for ants in the interactive CLI (opt out with
/// CLAUDE_CODE_REPL=0). The legacy CLAUDE_REPL_MODE=1 also forces it on.
///
/// SDK entrypoints (sdk-ts, sdk-py, sdk-cli) are NOT defaulted on — SDK
/// consumers script direct tool calls (Bash, Read, etc.) and REPL mode
/// hides those tools. USER_TYPE is a build-time --define, so the ant-native
/// binary would otherwise force REPL mode on every SDK subprocess regardless
/// of the env the caller passes.
pub fn is_repl_mode_enabled() -> bool {
    if is_env_defined_falsy("AI_CODE_REPL") {
        return false;
    }
    if is_env_truthy("AI_REPL_MODE") {
        return true;
    }
    let user_type = std::env::var("USER_TYPE").unwrap_or_default();
    let entrypoint = std::env::var("AI_CODE_ENTRYPOINT").unwrap_or_default();
    user_type == "ant" && entrypoint == "cli"
}

fn is_env_defined_falsy(key: &str) -> bool {
    std::env::var(key).is_ok()
}

fn is_env_truthy(key: &str) -> bool {
    std::env::var(key)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
        .unwrap_or(false)
}

/// Tools that are only accessible via REPL when REPL mode is enabled.
/// When REPL mode is on, these tools are hidden from Claude's direct use,
/// forcing Claude to use REPL for batch operations.
pub fn repl_only_tools() -> &'static HashSet<&'static str> {
    static REPL_ONLY_TOOLS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
        HashSet::from([
            "Read",
            "Write",
            "FileEdit",
            "Glob",
            "Grep",
            "Bash",
            "NotebookEdit",
            "Agent",
        ])
    });
    &REPL_ONLY_TOOLS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_tool_name() {
        assert_eq!(REPL_TOOL_NAME, "REPL");
    }

    #[test]
    fn test_repl_only_tools_contains_expected_tools() {
        let tools = repl_only_tools();
        assert!(tools.contains("Read"));
        assert!(tools.contains("Bash"));
        assert!(tools.contains("Grep"));
    }
}
