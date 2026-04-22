//! Environment utilities

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

/// Get the Claude config home directory (memoized)
/// Keyed off AI_CONFIG_DIR so tests that change the env var get a fresh value
static CLAUDE_CONFIG_HOME_DIR: Lazy<String> = Lazy::new(|| {
    let config_dir = env::var("AI_CONFIG_DIR")
        .or_else(|_| env::var("CLAUDE_CONFIG_DIR"))
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .map(|p| p.join(".ai").to_string_lossy().to_string())
                .unwrap_or_else(|| ".ai".to_string())
        });
    config_dir.normalize_nfc()
});

/// Get the teams directory
pub fn get_teams_dir() -> PathBuf {
    PathBuf::from(get_claude_config_home_dir()).join("teams")
}

/// Get the Claude config home directory
pub fn get_claude_config_home_dir() -> String {
    CLAUDE_CONFIG_HOME_DIR.clone()
}

/// Check if NODE_OPTIONS contains a specific flag.
/// Splits on whitespace and checks for exact match to avoid false positives.
pub fn has_node_option(flag: &str) -> bool {
    if let Ok(node_options) = env::var("NODE_OPTIONS") {
        node_options.split_whitespace().any(|opt| opt == flag)
    } else {
        false
    }
}

/// Check if an environment variable value is truthy
pub fn is_env_truthy(env_var: Option<&str>) -> bool {
    let Some(value) = env_var else {
        return false;
    };

    if value.is_empty() {
        return false;
    }

    let normalized = value.to_lowercase().trim().to_string();
    matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
}

/// Check if an environment variable is defined as falsy
pub fn is_env_defined_falsy(env_var: Option<&str>) -> bool {
    let Some(value) = env_var else {
        return false;
    };

    if value.is_empty() {
        return false;
    }

    let normalized = value.to_lowercase().trim().to_string();
    matches!(normalized.as_str(), "0" | "false" | "no" | "off")
}

/// Check if bare mode is enabled (--bare / AI_CODE_SIMPLE)
/// Skip hooks, LSP, plugin sync, skill dir-walk, attribution, background prefetches,
/// and ALL keychain/credential reads.
/// Auth is strictly ANTHROPIC_API_KEY env or apiKeyHelper from --settings.
/// Explicit CLI flags (--plugin-dir, --add-dir, --mcp-config) still honored.
pub fn is_bare_mode() -> bool {
    let is_simple = is_env_truthy(env::var("AI_CODE_SIMPLE").ok().as_deref());

    // Check argv directly (in addition to the env var) because several gates
    // run before main action handler sets AI_CODE_SIMPLE=1 from --bare
    let has_bare_arg = env::args().any(|arg| arg == "--bare");

    is_simple || has_bare_arg
}

/// Parses an array of environment variable strings into a key-value object
/// # Arguments
/// * `raw_env_args` - Array of strings in KEY=VALUE format
/// # Returns
/// Object with key-value pairs
/// # Errors
/// Returns an error if the format is invalid
pub fn parse_env_vars(
    raw_env_args: Option<Vec<String>>,
) -> Result<HashMap<String, String>, String> {
    let mut parsed_env: HashMap<String, String> = HashMap::new();

    if let Some(env_args) = raw_env_args {
        for env_str in env_args {
            if let Some((key, value)) = env_str.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                if key.is_empty() || value.is_empty() {
                    return Err(format!(
                        "Invalid environment variable format: {}, environment variables should be added as: -e KEY1=value1 -e KEY2=value2",
                        env_str
                    ));
                }
                // Handle KEY=VALUE where VALUE might contain = characters
                let full_value = env_str[key.len() + 1..].trim().to_string();
                parsed_env.insert(key.to_string(), full_value);
            } else {
                return Err(format!(
                    "Invalid environment variable format: {}, environment variables should be added as: -e KEY1=value1 -e KEY2=value2",
                    env_str
                ));
            }
        }
    }

    Ok(parsed_env)
}

/// Get the AWS region with fallback to default
/// Matches the AWS SDK's region behavior
pub fn get_aws_region() -> String {
    env::var("AI_AWS_REGION")
        .or_else(|_| env::var("AWS_REGION"))
        .or_else(|_| env::var("AI_AWS_DEFAULT_REGION"))
        .or_else(|_| env::var("AWS_DEFAULT_REGION"))
        .unwrap_or_else(|_| "us-east-1".to_string())
}

/// Get the default Vertex AI region
pub fn get_default_vertex_region() -> String {
    env::var("AI_CLOUD_ML_REGION")
        .or_else(|_| env::var("CLOUD_ML_REGION"))
        .unwrap_or_else(|_| "us-east5".to_string())
}

/// Check if bash commands should maintain project working directory
/// (reset to original after each command)
pub fn should_maintain_project_working_dir() -> bool {
    is_env_truthy(
        env::var("AI_BASH_MAINTAIN_PROJECT_WORKING_DIR")
            .ok()
            .as_deref(),
    ) || is_env_truthy(
        env::var("CLAUDE_BASH_MAINTAIN_PROJECT_WORKING_DIR")
            .ok()
            .as_deref(),
    )
}

/// Check if running on Homespace (ant-internal cloud environment)
pub fn is_running_on_homespace() -> bool {
    let user_type = env::var("USER_TYPE").unwrap_or_default();
    (user_type == "ant" && is_env_truthy(env::var("AI COO_RUNNING_ON_HOMESPACE").ok().as_deref()))
        || is_env_truthy(env::var("COO_RUNNING_ON_HOMESPACE").ok().as_deref())
}

/// Conservative check for whether AI Code is running inside a protected
/// (privileged or ASL3+) COO namespace or cluster.
/// Conservative means: when signals are ambiguous, assume protected.
/// Used for telemetry to measure auto-mode usage in sensitive environments.
/// Note: This is a stub - the actual implementation would require platform-specific code
pub fn is_in_protected_namespace() -> bool {
    false
}

/// Get USER_TYPE environment variable
pub fn get_user_type() -> Option<String> {
    env::var("USER_TYPE").ok()
}

/// Check if running in ant-internal build
pub fn is_ant_user() -> bool {
    get_user_type().as_deref() == Some("ant")
}

/// Check if running in test mode
pub fn is_test_mode() -> bool {
    env::var("NODE_ENV").map(|v| v == "test").unwrap_or(false)
}

/// Get platform name
pub fn get_platform() -> String {
    env::consts::OS.to_string()
}

/// Model prefix to env var for Vertex region overrides.
/// Order matters: more specific prefixes must come before less specific ones
/// (e.g., 'claude-opus-4-1' before 'claude-opus-4').
const VERTEX_REGION_OVERRIDES: &[(&str, &str)] = &[
    ("claude-haiku-4-5", "AI_VERTEX_REGION_CLAUDE_HAIKU_4_5"),
    ("claude-3-5-haiku", "AI_VERTEX_REGION_CLAUDE_3_5_HAIKU"),
    ("claude-3-5-sonnet", "AI_VERTEX_REGION_CLAUDE_3_5_SONNET"),
    ("claude-3-7-sonnet", "AI_VERTEX_REGION_CLAUDE_3_7_SONNET"),
    ("claude-opus-4-1", "AI_VERTEX_REGION_CLAUDE_4_1_OPUS"),
    ("claude-opus-4", "AI_VERTEX_REGION_CLAUDE_4_0_OPUS"),
    ("claude-sonnet-4-6", "AI_VERTEX_REGION_CLAUDE_4_6_SONNET"),
    ("claude-sonnet-4-5", "AI_VERTEX_REGION_CLAUDE_4_5_SONNET"),
    ("claude-sonnet-4", "AI_VERTEX_REGION_CLAUDE_4_0_SONNET"),
];

/// Get the Vertex AI region for a specific model.
/// Different models may be available in different regions.
pub fn get_vertex_region_for_model(model: Option<&str>) -> Option<String> {
    let model = model?;

    for (prefix, env_var) in VERTEX_REGION_OVERRIDES {
        if model.starts_with(prefix) {
            // Check AI_ prefixed version first, then original
            let region = env::var(&format!("AI_{}", env_var.trim_start_matches("AI_")))
                .or_else(|_| env::var(*env_var))
                .ok();

            return Some(region.unwrap_or_else(get_default_vertex_region));
        }
    }

    Some(get_default_vertex_region())
}

// Helper trait for NFC normalization
trait NfcNormalize {
    fn normalize_nfc(&self) -> String;
}

impl NfcNormalize for String {
    fn normalize_nfc(&self) -> String {
        // Rust strings are already UTF-8, NFC normalization is primarily relevant
        // for strings that will be displayed or compared
        // For most use cases, the string as-is is fine
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_env_truthy() {
        assert!(is_env_truthy(Some("1")));
        assert!(is_env_truthy(Some("true")));
        assert!(is_env_truthy(Some("True")));
        assert!(is_env_truthy(Some("yes")));
        assert!(is_env_truthy(Some("on")));

        assert!(!is_env_truthy(Some("0")));
        assert!(!is_env_truthy(Some("false")));
        assert!(!is_env_truthy(Some("no")));
        assert!(!is_env_truthy(Some("off")));
        assert!(!is_env_truthy(None));
        assert!(!is_env_truthy(Some("")));
    }

    #[test]
    fn test_is_env_defined_falsy() {
        assert!(is_env_defined_falsy(Some("0")));
        assert!(is_env_defined_falsy(Some("false")));
        assert!(is_env_defined_falsy(Some("no")));
        assert!(is_env_defined_falsy(Some("off")));

        assert!(!is_env_defined_falsy(Some("1")));
        assert!(!is_env_defined_falsy(Some("true")));
        assert!(!is_env_defined_falsy(None));
    }

    #[test]
    fn test_parse_env_vars() {
        let result = parse_env_vars(Some(vec![
            "KEY1=value1".to_string(),
            "KEY2=value2".to_string(),
        ]))
        .unwrap();

        assert_eq!(result.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(result.get("KEY2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_parse_env_vars_with_equals() {
        let result = parse_env_vars(Some(vec!["KEY=foo=bar=baz".to_string()])).unwrap();

        assert_eq!(result.get("KEY"), Some(&"foo=bar=baz".to_string()));
    }

    #[test]
    fn test_parse_env_vars_invalid() {
        assert!(parse_env_vars(Some(vec!["=value".to_string()])).is_err());
        assert!(parse_env_vars(Some(vec!["KEY=".to_string()])).is_err());
    }

    #[test]
    fn test_get_aws_region() {
        // Without env vars set, should return default
        let region = get_aws_region();
        assert_eq!(region, "us-east-1");
    }

    #[test]
    fn test_get_vertex_region_for_model() {
        assert_eq!(
            get_vertex_region_for_model(Some("claude-3-5-sonnet-20241022")),
            Some("us-east5".to_string())
        );
        assert_eq!(get_vertex_region_for_model(None), None);
    }
}
