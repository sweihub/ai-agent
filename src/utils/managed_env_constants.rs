//! Environment variable constants for managed inference routing.
//!
//! These control which provider to use, which endpoint to hit, and which model IDs to send.
//! When AI_CODE_PROVIDER_MANAGED_BY_HOST is truthy in the spawn env, these
//! are stripped from settings-sourced env so the host's routing config isn't
//! overridden by a user's ~/.ai/settings.json.

use crate::constants::env::{ai_code, ai};
use std::collections::HashSet;

/// Provider-managed environment variables that control inference routing
pub const PROVIDER_MANAGED_ENV_VARS: &[&str] = &[
    // The flag itself — settings can't unset it once the host set it
    ai_code::PROVIDER_MANAGED_BY_HOST,
    // Provider selection
    ai_code::USE_BEDROCK,
    ai_code::USE_VERTEX,
    ai_code::USE_FOUNDRY,
    // Endpoint config (base URLs, project/resource identifiers)
    ai::BASE_URL,
    ai::BEDROCK_BASE_URL,
    ai::VERTEX_BASE_URL,
    ai::FOUNDRY_BASE_URL,
    ai::FOUNDRY_RESOURCE,
    ai::VERTEX_PROJECT_ID,
    // Region routing (per-model VERTEX_REGION_CLAUDE_* handled by prefix below)
    "CLOUD_ML_REGION",
    // Auth
    ai::API_KEY,
    ai::AUTH_TOKEN,
    "AI_CODE_OAUTH_TOKEN",
    "AWS_BEARER_TOKEN_BEDROCK",
    ai::FOUNDRY_API_KEY,
    ai_code::SKIP_BEDROCK_AUTH,
    ai_code::SKIP_VERTEX_AUTH,
    ai_code::SKIP_FOUNDRY_AUTH,
    // Model defaults — often set to provider-specific ID formats
    ai::MODEL,
    ai::DEFAULT_HAIKU_MODEL,
    "ANTHROPIC_DEFAULT_HAIKU_MODEL_DESCRIPTION",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL_NAME",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL_SUPPORTED_CAPABILITIES",
    ai::DEFAULT_OPUS_MODEL,
    "ANTHROPIC_DEFAULT_OPUS_MODEL_DESCRIPTION",
    "ANTHROPIC_DEFAULT_OPUS_MODEL_NAME",
    "ANTHROPIC_DEFAULT_OPUS_MODEL_SUPPORTED_CAPABILITIES",
    ai::DEFAULT_SONNET_MODEL,
    "ANTHROPIC_DEFAULT_SONNET_MODEL_DESCRIPTION",
    "ANTHROPIC_DEFAULT_SONNET_MODEL_NAME",
    "ANTHROPIC_DEFAULT_SONNET_MODEL_SUPPORTED_CAPABILITIES",
    "AI_SMALL_FAST_MODEL",
    "ANTHROPIC_SMALL_FAST_MODEL_AWS_REGION",
    ai_code::SUBAGENT_MODEL,
];

/// Provider-managed environment variable prefixes (for prefix matching)
pub const PROVIDER_MANAGED_ENV_PREFIXES: &[&str] = &[
    // Per-model Vertex region overrides — scales with model releases
    "VERTEX_REGION_CLAUDE_",
];

lazy_static::lazy_static! {
    static ref PROVIDER_MANAGED_ENV_VARS_SET: HashSet<String> = {
        PROVIDER_MANAGED_ENV_VARS.iter().map(|s| s.to_string()).collect()
    };
}

/// Check if an environment variable is provider-managed
pub fn is_provider_managed_env_var(key: &str) -> bool {
    let upper = key.to_uppercase();
    if PROVIDER_MANAGED_ENV_VARS_SET.contains(&upper) {
        return true;
    }
    for prefix in PROVIDER_MANAGED_ENV_PREFIXES {
        if upper.starts_with(prefix) {
            return true;
        }
    }
    false
}

/// Dangerous shell settings that can execute arbitrary shell code
pub const DANGEROUS_SHELL_SETTINGS: &[&str] = &[
    "apiKeyHelper",
    "awsAuthRefresh",
    "awsCredentialExport",
    "gcpAuthRefresh",
    "otelHeadersHelper",
    "statusLine",
];

/// Safe environment variables that can be applied before trust dialog.
/// These are Claude Code specific settings that don't pose security risks.
///
/// IMPORTANT: This is the source of truth for which env vars are safe.
/// Any env var NOT in this list is considered dangerous and will trigger
/// a security dialog when set via remote managed settings.
pub const SAFE_ENV_VARS: &[&str] = &[
    "ANTHROPIC_CUSTOM_HEADERS",
    "ANTHROPIC_CUSTOM_MODEL_OPTION",
    "ANTHROPIC_CUSTOM_MODEL_OPTION_DESCRIPTION",
    "ANTHROPIC_CUSTOM_MODEL_OPTION_NAME",
    ai::DEFAULT_HAIKU_MODEL,
    "ANTHROPIC_DEFAULT_HAIKU_MODEL_DESCRIPTION",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL_NAME",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL_SUPPORTED_CAPABILITIES",
    ai::DEFAULT_OPUS_MODEL,
    "ANTHROPIC_DEFAULT_OPUS_MODEL_DESCRIPTION",
    "ANTHROPIC_DEFAULT_OPUS_MODEL_NAME",
    "ANTHROPIC_DEFAULT_OPUS_MODEL_SUPPORTED_CAPABILITIES",
    ai::DEFAULT_SONNET_MODEL,
    "ANTHROPIC_DEFAULT_SONNET_MODEL_DESCRIPTION",
    "ANTHROPIC_DEFAULT_SONNET_MODEL_NAME",
    "ANTHROPIC_DEFAULT_SONNET_MODEL_SUPPORTED_CAPABILITIES",
    ai::FOUNDRY_API_KEY,
    ai::MODEL,
    "ANTHROPIC_SMALL_FAST_MODEL_AWS_REGION",
    "ANTHROPIC_SMALL_FAST_MODEL",
    "AWS_DEFAULT_REGION",
    "AWS_PROFILE",
    "AWS_REGION",
    "BASH_DEFAULT_TIMEOUT_MS",
    "BASH_MAX_OUTPUT_LENGTH",
    "BASH_MAX_TIMEOUT_MS",
    "AI_CODE_BASH_MAINTAIN_PROJECT_WORKING_DIR",
    "AI_CODE_API_KEY_HELPER_TTL_MS",
    "AI_CODE_DISABLE_EXPERIMENTAL_BETAS",
    "AI_CODE_DISABLE_NONESSENTIAL_TRAFFIC",
    "AI_CODE_DISABLE_TERMINAL_TITLE",
    "AI_CODE_ENABLE_TELEMETRY",
    "AI_CODE_EXPERIMENTAL_AGENT_TEAMS",
    "AI_CODE_IDE_SKIP_AUTO_INSTALL",
    "AI_CODE_MAX_OUTPUT_TOKENS",
    "AI_CODE_SKIP_BEDROCK_AUTH",
    "AI_CODE_SKIP_FOUNDRY_AUTH",
    "AI_CODE_SKIP_VERTEX_AUTH",
    "AI_CODE_SUBAGENT_MODEL",
    "AI_CODE_USE_BEDROCK",
    "AI_CODE_USE_FOUNDRY",
    "AI_CODE_USE_VERTEX",
    "DISABLE_AUTOUPDATER",
    "DISABLE_BUG_COMMAND",
    "DISABLE_COST_WARNINGS",
    "DISABLE_ERROR_REPORTING",
    "DISABLE_FEEDBACK_COMMAND",
    "DISABLE_TELEMETRY",
    "ENABLE_TOOL_SEARCH",
    "MAX_MCP_OUTPUT_TOKENS",
    "MAX_THINKING_TOKENS",
    "MCP_TIMEOUT",
    "MCP_TOOL_TIMEOUT",
    "OTEL_EXPORTER_OTLP_HEADERS",
    "OTEL_EXPORTER_OTLP_LOGS_HEADERS",
    "OTEL_EXPORTER_OTLP_LOGS_PROTOCOL",
    "OTEL_EXPORTER_OTLP_METRICS_CLIENT_CERTIFICATE",
    "OTEL_EXPORTER_OTLP_METRICS_CLIENT_KEY",
    "OTEL_EXPORTER_OTLP_METRICS_HEADERS",
    "OTEL_EXPORTER_OTLP_METRICS_PROTOCOL",
    "OTEL_EXPORTER_OTLP_PROTOCOL",
    "OTEL_EXPORTER_OTLP_TRACES_HEADERS",
    "OTEL_LOG_TOOL_DETAILS",
    "OTEL_LOG_USER_PROMPTS",
    "OTEL_LOGS_EXPORT_INTERVAL",
    "OTEL_LOGS_EXPORTER",
    "OTEL_METRIC_EXPORT_INTERVAL",
    "OTEL_METRICS_EXPORTER",
    "OTEL_METRICS_INCLUDE_ACCOUNT_UUID",
    "OTEL_METRICS_INCLUDE_SESSION_ID",
    "OTEL_METRICS_INCLUDE_VERSION",
    "OTEL_RESOURCE_ATTRIBUTES",
    "USE_BUILTIN_RIPGREP",
    "VERTEX_REGION_CLAUDE_3_5_HAIKU",
    "VERTEX_REGION_CLAUDE_3_5_SONNET",
    "VERTEX_REGION_CLAUDE_3_7_SONNET",
    "VERTEX_REGION_CLAUDE_4_0_OPUS",
    "VERTEX_REGION_CLAUDE_4_0_SONNET",
    "VERTEX_REGION_CLAUDE_4_1_OPUS",
    "VERTEX_REGION_CLAUDE_4_5_SONNET",
    "VERTEX_REGION_CLAUDE_4_6_SONNET",
    "VERTEX_REGION_CLAUDE_HAIKU_4_5",
];

lazy_static::lazy_static! {
    static ref SAFE_ENV_VARS_SET: HashSet<String> = {
        SAFE_ENV_VARS.iter().map(|s| s.to_string()).collect()
    };
}

/// Check if an environment variable is safe to apply before trust dialog
pub fn is_safe_env_var(key: &str) -> bool {
    SAFE_ENV_VARS_SET.contains(&key.to_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_provider_managed_env_var() {
        assert!(is_provider_managed_env_var(ai::BASE_URL));
        assert!(is_provider_managed_env_var(ai::API_KEY));
        assert!(is_provider_managed_env_var(
            "VERTEX_REGION_CLAUDE_3_5_HAIKU"
        ));
        assert!(!is_provider_managed_env_var("HOME"));
        assert!(!is_provider_managed_env_var("PATH"));
    }

    #[test]
    fn test_is_safe_env_var() {
        assert!(is_safe_env_var(ai::MODEL));
        assert!(is_safe_env_var("DISABLE_TELEMETRY"));
        assert!(is_safe_env_var("MCP_TIMEOUT"));
        assert!(!is_safe_env_var(ai::BASE_URL));
        assert!(!is_safe_env_var(ai::API_KEY));
    }
}
