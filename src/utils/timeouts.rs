// Source: ~/claudecode/openclaudecode/src/utils/timeouts.rs

/// Constants for timeout values.
const DEFAULT_TIMEOUT_MS: u64 = 120_000; // 2 minutes
const MAX_TIMEOUT_MS: u64 = 600_000; // 10 minutes

/// Environment-like type alias.
type EnvLike = std::collections::HashMap<String, String>;

/// Get the default timeout for bash operations in milliseconds.
/// Checks BASH_DEFAULT_TIMEOUT_MS environment variable or returns 2 minutes default.
pub fn get_default_bash_timeout_ms(env: Option<&EnvLike>) -> u64 {
    let env = env.cloned().unwrap_or_else(|| std::env::vars().collect());

    if let Some(env_value) = env.get("BASH_DEFAULT_TIMEOUT_MS") {
        if let Ok(parsed) = env_value.parse::<u64>() {
            if parsed > 0 {
                return parsed;
            }
        }
    }
    DEFAULT_TIMEOUT_MS
}

/// Get the maximum timeout for bash operations in milliseconds.
/// Checks BASH_MAX_TIMEOUT_MS environment variable or returns 10 minutes default.
pub fn get_max_bash_timeout_ms(env: Option<&EnvLike>) -> u64 {
    let env = env.cloned().unwrap_or_else(|| std::env::vars().collect());

    if let Some(env_value) = env.get("BASH_MAX_TIMEOUT_MS") {
        if let Ok(parsed) = env_value.parse::<u64>() {
            if parsed > 0 {
                // Ensure max is at least as large as default
                return parsed.max(get_default_bash_timeout_ms(Some(&env)));
            }
        }
    }
    // Always ensure max is at least as large as default
    MAX_TIMEOUT_MS.max(get_default_bash_timeout_ms(Some(&env)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_timeout() {
        let env = EnvLike::new();
        assert_eq!(get_default_bash_timeout_ms(Some(&env)), DEFAULT_TIMEOUT_MS);
    }

    #[test]
    fn test_default_timeout_from_env() {
        let mut env = EnvLike::new();
        env.insert("BASH_DEFAULT_TIMEOUT_MS".to_string(), "30000".to_string());
        assert_eq!(get_default_bash_timeout_ms(Some(&env)), 30000);
    }

    #[test]
    fn test_default_timeout_invalid() {
        let mut env = EnvLike::new();
        env.insert("BASH_DEFAULT_TIMEOUT_MS".to_string(), "invalid".to_string());
        assert_eq!(get_default_bash_timeout_ms(Some(&env)), DEFAULT_TIMEOUT_MS);
    }

    #[test]
    fn test_max_timeout() {
        let env = EnvLike::new();
        assert_eq!(get_max_bash_timeout_ms(Some(&env)), MAX_TIMEOUT_MS);
    }

    #[test]
    fn test_max_timeout_from_env() {
        let mut env = EnvLike::new();
        env.insert("BASH_MAX_TIMEOUT_MS".to_string(), "300000".to_string());
        assert_eq!(get_max_bash_timeout_ms(Some(&env)), 300000);
    }

    #[test]
    fn test_max_timeout_less_than_default() {
        let mut env = EnvLike::new();
        env.insert("BASH_DEFAULT_TIMEOUT_MS".to_string(), "300000".to_string());
        env.insert("BASH_MAX_TIMEOUT_MS".to_string(), "10000".to_string());
        // Max should be at least as large as default
        assert_eq!(get_max_bash_timeout_ms(Some(&env)), 300000);
    }
}
