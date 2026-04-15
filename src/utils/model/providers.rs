// Source: ~/claudecode/openclaudecode/src/utils/model/providers.ts

use serde::Serialize;

/// API Provider type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ApiProvider {
    FirstParty,
    Bedrock,
    Vertex,
    Foundry,
}

/// Check if an environment variable is truthy.
fn is_env_truthy(value: Option<String>) -> bool {
    match value {
        Some(v) => {
            let v = v.to_lowercase();
            v == "1" || v == "true" || v == "yes" || v == "on"
        }
        None => false,
    }
}

/// Get the current API provider.
pub fn get_api_provider() -> ApiProvider {
    // Localized: CLAUDE_CODE_* -> AI_CODE_*
    if is_env_truthy(std::env::var("AI_CODE_USE_BEDROCK").ok()) {
        return ApiProvider::Bedrock;
    }
    if is_env_truthy(std::env::var("AI_CODE_USE_VERTEX").ok()) {
        return ApiProvider::Vertex;
    }
    if is_env_truthy(std::env::var("AI_CODE_USE_FOUNDRY").ok()) {
        return ApiProvider::Foundry;
    }
    ApiProvider::FirstParty
}

/// Check if AI_BASE_URL is a first-party Anthropic API URL.
/// Returns true if not set (default API) or points to api.anthropic.com
/// (or api-staging.anthropic.com for ant users).
pub fn is_first_party_anthropic_base_url() -> bool {
    let Some(base_url) = std::env::var("AI_BASE_URL").ok() else {
        return true;
    };

    match url::Url::parse(&base_url) {
        Ok(parsed) => {
            let host = parsed.host_str().unwrap_or("");
            let mut allowed_hosts = vec!["api.anthropic.com"];
            let user_type = std::env::var("USER_TYPE").ok();
            if user_type.as_deref() == Some("ant") {
                allowed_hosts.push("api-staging.anthropic.com");
            }
            allowed_hosts.contains(&host)
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_env_truthy() {
        assert!(is_env_truthy(Some("1".to_string())));
        assert!(is_env_truthy(Some("true".to_string())));
        assert!(is_env_truthy(Some("TRUE".to_string())));
        assert!(!is_env_truthy(None));
        assert!(!is_env_truthy(Some("0".to_string())));
        assert!(!is_env_truthy(Some("false".to_string())));
    }

    #[test]
    fn test_get_api_provider_default() {
        std::env::remove_var("AI_CODE_USE_BEDROCK");
        std::env::remove_var("AI_CODE_USE_VERTEX");
        std::env::remove_var("AI_CODE_USE_FOUNDRY");
        assert_eq!(get_api_provider(), ApiProvider::FirstParty);
    }

    #[test]
    fn test_is_first_party_anthropic_base_url_unset() {
        std::env::remove_var("AI_BASE_URL");
        assert!(is_first_party_anthropic_base_url());
    }
}
