// Source: /data/home/swei/claudecode/openclaudecode/src/constants/system.ts
use crate::constants::env::ai_code;

pub const DEFAULT_PREFIX: &str = "You are Claude Code, Anthropic's official CLI for Claude.";

pub const AGENT_SDK_CLAUDE_CODE_PRESET_PREFIX: &str =
    "You are Claude Code, Anthropic's official CLI for Claude, running within the Claude Agent SDK.";

pub const AGENT_SDK_PREFIX: &str = "You are a Claude agent, built on Anthropic's Claude Agent SDK.";

pub const CLI_SYSPROMPT_PREFIX_VALUES: &[&str] = &[
    DEFAULT_PREFIX,
    AGENT_SDK_CLAUDE_CODE_PRESET_PREFIX,
    AGENT_SDK_PREFIX,
];

pub fn get_cli_sysprompt_prefix(
    is_non_interactive: bool,
    has_append_system_prompt: bool,
) -> &'static str {
    if is_non_interactive {
        if has_append_system_prompt {
            AGENT_SDK_CLAUDE_CODE_PRESET_PREFIX
        } else {
            AGENT_SDK_PREFIX
        }
    } else {
        DEFAULT_PREFIX
    }
}

pub fn get_attribution_header(fingerprint: &str, entrypoint: &str) -> String {
    let enabled = std::env::var(ai_code::ATTRIBUTION_HEADER)
        .map(|v| v != "false" && v != "0" && v.to_lowercase() != "no")
        .unwrap_or(true);

    if !enabled {
        return String::new();
    }

    let version = format!("{}.{}", env!("CARGO_PKG_VERSION"), fingerprint);

    format!(
        "x-anthropic-billing-header: cc_version={}; cc_entrypoint={};",
        version, entrypoint
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sysprompt_prefixes() {
        assert!(CLI_SYSPROMPT_PREFIX_VALUES.contains(&DEFAULT_PREFIX));
        assert!(CLI_SYSPROMPT_PREFIX_VALUES.contains(&AGENT_SDK_CLAUDE_CODE_PRESET_PREFIX));
        assert!(CLI_SYSPROMPT_PREFIX_VALUES.contains(&AGENT_SDK_PREFIX));
    }

    #[test]
    fn test_get_cli_sysprompt_prefix() {
        assert_eq!(get_cli_sysprompt_prefix(false, false), DEFAULT_PREFIX);
        assert_eq!(get_cli_sysprompt_prefix(true, false), AGENT_SDK_PREFIX);
        assert_eq!(
            get_cli_sysprompt_prefix(true, true),
            AGENT_SDK_CLAUDE_CODE_PRESET_PREFIX
        );
    }

    #[test]
    fn test_attribution_header_disabled() {}

    #[test]
    fn test_attribution_header_enabled() {}

    #[test]
    fn test_attribution_header_format() {
        let header = get_attribution_header("fingerprint123", "cli");
        assert!(header.contains("cc_version="));
        assert!(header.contains("cc_entrypoint=cli"));
    }
}
