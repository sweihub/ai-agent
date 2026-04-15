//! Env vars to strip from subprocess environments when running inside GitHub
//! Actions. This prevents prompt-injection attacks from exfiltrating secrets
//! via shell expansion (e.g., ${AI_API_KEY}) in Bash tool commands.
//!
//! The parent claude process keeps these vars (needed for API calls, lazy
//! credential reads). Only child processes (bash, shell snapshot, MCP stdio, LSP, hooks) are scrubbed.
//!
//! GITHUB_TOKEN / GH_TOKEN are intentionally NOT scrubbed — wrapper scripts
//! (gh.sh) need them to call the GitHub API. That token is job-scoped and
//! expires when the workflow ends.

use crate::constants::env::{ai, ai_code};
use std::collections::HashMap;
use std::env;

/// Env vars to strip from subprocess environments when running inside GitHub Actions
pub static GHA_SUBPROCESS_SCRUB: &[&str] = &[
    // Anthropic auth — claude re-reads these per-request, subprocesses don't need them
    ai::API_KEY,
    ai_code::OAUTH_TOKEN,
    ai::AUTH_TOKEN,
    ai::FOUNDRY_API_KEY,
    "ANTHROPIC_CUSTOM_HEADERS",
    // OTLP exporter headers — documented to carry Authorization=Bearer tokens
    // for monitoring backends; read in-process by OTEL SDK, subprocesses never need them
    "OTEL_EXPORTER_OTLP_HEADERS",
    "OTEL_EXPORTER_OTLP_LOGS_HEADERS",
    "OTEL_EXPORTER_OTLP_METRICS_HEADERS",
    "OTEL_EXPORTER_OTLP_TRACES_HEADERS",
    // Cloud provider creds — same pattern (lazy SDK reads)
    "AWS_SECRET_ACCESS_KEY",
    "AWS_SESSION_TOKEN",
    "AWS_BEARER_TOKEN_BEDROCK",
    "GOOGLE_APPLICATION_CREDENTIALS",
    "AZURE_CLIENT_SECRET",
    "AZURE_CLIENT_CERTIFICATE_PATH",
    // GitHub Actions OIDC — consumed by the action's JS before claude spawns;
    // leaking these allows minting an App installation token → repo takeover
    "ACTIONS_ID_TOKEN_REQUEST_TOKEN",
    "ACTIONS_ID_TOKEN_REQUEST_URL",
    // GitHub Actions artifact/cache API — cache poisoning → supply-chain pivot
    "ACTIONS_RUNTIME_TOKEN",
    "ACTIONS_RUNTIME_URL",
    // claude-code-action-specific duplicates — action JS consumes these during
    // prepare, before spawning claude. ALL_INPUTS contains anthropic_api_key as JSON.
    "ALL_INPUTS",
    "OVERRIDE_GITHUB_TOKEN",
    "DEFAULT_WORKFLOW_TOKEN",
    "SSH_SIGNING_KEY",
];

/// Registered by init.ts after the upstreamproxy module is dynamically imported
/// in CCR sessions. Stays None in non-CCR startups so we never pull in the
/// upstreamproxy module graph (upstreamproxy.ts + relay.ts) via a static import.
static UPSTREAM_PROXY_ENV_FN: std::sync::OnceLock<
    Box<dyn Fn() -> HashMap<String, String> + Send + Sync>,
> = std::sync::OnceLock::new();

/// Called from init.ts to wire up the proxy env function after the upstreamproxy
/// module has been lazily loaded. Must be called before any subprocess is spawned.
pub fn register_upstream_proxy_env_fn<F>(fn_: F)
where
    F: Fn() -> HashMap<String, String> + Send + Sync + 'static,
{
    let _ = UPSTREAM_PROXY_ENV_FN.set(Box::new(fn_));
}

/// Returns a copy of process.env with sensitive secrets stripped, for use when
/// spawning subprocesses (Bash tool, shell snapshot, MCP stdio servers, LSP
/// servers, shell hooks).
///
/// Gated on AI_CODE_SUBPROCESS_ENV_SCRUB. claude-code-action sets this
/// automatically when `allowed_non_write_users` is configured — the flag that
/// exposes a workflow to untrusted content (prompt injection surface).
pub fn subprocess_env() -> HashMap<String, String> {
    // CCR upstreamproxy: inject HTTPS_PROXY + CA bundle vars so curl/gh/python
    // in agent subprocesses route through the local relay. Returns empty when the
    // proxy is disabled or not registered (non-CCR), so this is a no-op outside
    // CCR containers.
    let proxy_env = UPSTREAM_PROXY_ENV_FN.get().map(|f| f()).unwrap_or_default();

    let should_scrub = env::var("AI_CODE_SUBPROCESS_ENV_SCRUB")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false);

    if !should_scrub {
        if proxy_env.is_empty() {
            return env::vars().collect();
        }
        let mut env: HashMap<String, String> = env::vars().collect();
        env.extend(proxy_env);
        return env;
    }

    let mut env: HashMap<String, String> = env::vars().collect();
    env.extend(proxy_env);

    for k in GHA_SUBPROCESS_SCRUB {
        env.remove(*k);
        // GitHub Actions auto-creates INPUT_<NAME> for `with:` inputs, duplicating
        // secrets like INPUT_ANTHROPIC_API_KEY. No-op for vars that aren't action inputs.
        env.remove(&format!("INPUT_{}", k));
    }

    env
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subprocess_env_no_scrub() {
        // When AI_CODE_SUBPROCESS_ENV_SCRUB is not set, should return all env vars
        let env = subprocess_env();
        assert!(!env.is_empty());
    }

    #[test]
    fn test_subprocess_env_with_scrub() {
        // Set the scrub flag
        unsafe { env::set_var("AI_CODE_SUBPROCESS_ENV_SCRUB", "1") };

        let env = subprocess_env();

        // These should be removed
        assert!(!env.contains_key(ai::API_KEY));
        assert!(!env.contains_key("AWS_SECRET_ACCESS_KEY"));

        // PATH should still be there
        assert!(env.contains_key("PATH"));

        // Clean up
        unsafe { env::remove_var("AI_CODE_SUBPROCESS_ENV_SCRUB") };
    }
}
