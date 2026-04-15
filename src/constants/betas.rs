// Source: /data/home/swei/claudecode/openclaudecode/src/constants/betas.ts
//! Beta feature headers for API requests.

/// Claude Code 2025-02-19 beta header
pub const CLAUDE_CODE_20250219_BETA_HEADER: &str = "claude-code-20250219";

/// Interleaved thinking beta header
pub const INTERLEAVED_THINKING_BETA_HEADER: &str = "interleaved-thinking-2025-05-14";

/// Context 1M beta header
pub const CONTEXT_1M_BETA_HEADER: &str = "context-1m-2025-08-07";

/// Context management beta header
pub const CONTEXT_MANAGEMENT_BETA_HEADER: &str = "context-management-2025-06-27";

/// Structured outputs beta header
pub const STRUCTURED_OUTPUTS_BETA_HEADER: &str = "structured-outputs-2025-12-15";

/// Web search beta header
pub const WEB_SEARCH_BETA_HEADER: &str = "web-search-2025-03-05";

/// Tool search beta header for 1st party (Claude API / Foundry)
pub const TOOL_SEARCH_BETA_HEADER_1P: &str = "advanced-tool-use-2025-11-20";

/// Tool search beta header for 3rd party (Vertex AI / Bedrock)
pub const TOOL_SEARCH_BETA_HEADER_3P: &str = "tool-search-tool-2025-10-19";

/// Effort beta header
pub const EFFORT_BETA_HEADER: &str = "effort-2025-11-24";

/// Task budgets beta header
pub const TASK_BUDGETS_BETA_HEADER: &str = "task-budgets-2026-03-13";

/// Prompt caching scope beta header
pub const PROMPT_CACHING_SCOPE_BETA_HEADER: &str = "prompt-caching-scope-2026-01-05";

/// Fast mode beta header
pub const FAST_MODE_BETA_HEADER: &str = "fast-mode-2026-02-01";

/// Redact thinking beta header
pub const REDACT_THINKING_BETA_HEADER: &str = "redact-thinking-2026-02-12";

/// Token efficient tools beta header
pub const TOKEN_EFFICIENT_TOOLS_BETA_HEADER: &str = "token-efficient-tools-2026-03-28";

/// Summarize connector text beta header (feature-gated)
pub fn get_summarize_connector_text_beta_header() -> &'static str {
    // Feature flag check would go here in actual implementation
    "summarize-connector-text-2026-03-13"
}

/// AFK mode beta header (feature-gated)
pub fn get_afk_mode_beta_header() -> &'static str {
    // Feature flag check would go here in actual implementation
    "afk-mode-2026-01-31"
}

/// CLI internal beta header (ant user only)
pub fn get_cli_internal_beta_header() -> &'static str {
    // Check USER_TYPE env var - would need platform crate for actual impl
    ""
}

/// Advisor beta header
pub const ADVISOR_BETA_HEADER: &str = "advisor-tool-2026-03-01";

use std::collections::HashSet;

/// Bedrock only supports a limited number of beta headers and only through
/// extraBodyParams. This set maintains the beta strings that should be in
/// Bedrock extraBodyParams *and not* in Bedrock headers.
pub fn get_bedrock_extra_params_headers() -> HashSet<&'static str> {
    let mut set = HashSet::new();
    set.insert(INTERLEAVED_THINKING_BETA_HEADER);
    set.insert(CONTEXT_1M_BETA_HEADER);
    set.insert(TOOL_SEARCH_BETA_HEADER_3P);
    set
}

/// Betas allowed on Vertex countTokens API.
/// Other betas will cause 400 errors.
pub fn get_vertex_count_tokens_allowed_betas() -> HashSet<&'static str> {
    let mut set = HashSet::new();
    set.insert(CLAUDE_CODE_20250219_BETA_HEADER);
    set.insert(INTERLEAVED_THINKING_BETA_HEADER);
    set.insert(CONTEXT_MANAGEMENT_BETA_HEADER);
    set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bedrock_extra_params_headers() {
        let headers = get_bedrock_extra_params_headers();
        assert!(headers.contains(INTERLEAVED_THINKING_BETA_HEADER));
        assert!(headers.contains(CONTEXT_1M_BETA_HEADER));
        assert!(headers.contains(TOOL_SEARCH_BETA_HEADER_3P));
    }

    #[test]
    fn test_vertex_count_tokens_allowed_betas() {
        let betas = get_vertex_count_tokens_allowed_betas();
        assert!(betas.contains(CLAUDE_CODE_20250219_BETA_HEADER));
        assert!(betas.contains(INTERLEAVED_THINKING_BETA_HEADER));
        assert!(betas.contains(CONTEXT_MANAGEMENT_BETA_HEADER));
    }
}
