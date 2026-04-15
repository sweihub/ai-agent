// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/envExpansion.ts
//! Shared utilities for expanding environment variables in MCP server configurations

use std::env;

/// Expand environment variables in a string value
/// Handles ${VAR} and ${VAR:-default} syntax
/// Returns Object with expanded string and list of missing variables
pub fn expand_env_vars_in_string(value: &str) -> (String, Vec<String>) {
    let mut missing_vars = Vec::new();
    let mut result = String::new();
    let mut chars = value.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            // Found ${ - parse variable
            chars.next(); // consume '{'
            let mut var_content = String::new();

            while let Some(ch) = chars.next() {
                if ch == '}' {
                    break;
                }
                var_content.push(ch);
            }

            // Split on :- to support default values
            let parts: Vec<&str> = var_content.splitn(2, ":-").collect();
            let var_name = parts[0];
            let default_value = parts.get(1);

            if let Ok(env_value) = env::var(var_name) {
                result.push_str(&env_value);
            } else if let Some(default) = default_value {
                result.push_str(default);
            } else {
                // Track missing variable for error reporting
                missing_vars.push(var_name.to_string());
                // Return original if not found
                result.push_str(&format!("${{{}}}", var_content));
            }
        } else {
            result.push(c);
        }
    }

    (result, missing_vars)
}

/// Expand environment variables in a map of strings
pub fn expand_env_vars_in_map(
    env: &std::collections::HashMap<String, String>,
) -> (std::collections::HashMap<String, String>, Vec<String>) {
    let mut missing_vars = Vec::new();
    let mut result = std::collections::HashMap::new();

    for (key, value) in env {
        let (expanded, mut missing) = expand_env_vars_in_string(value);
        missing_vars.append(&mut missing);
        result.insert(key.clone(), expanded);
    }

    (result, missing_vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_env_vars_simple() {
        // Set a test env var
        unsafe { env::set_var("TEST_VAR", "test_value") };

        let (result, missing) = expand_env_vars_in_string("prefix_${TEST_VAR}_suffix");
        assert_eq!(result, "prefix_test_value_suffix");
        assert!(missing.is_empty());

        unsafe { env::remove_var("TEST_VAR") };
    }

    #[test]
    fn test_expand_env_vars_with_default() {
        let (result, missing) = expand_env_vars_in_string("value is ${NONEXISTENT:-default_val}");
        assert_eq!(result, "value is default_val");
        assert!(missing.is_empty());
    }

    #[test]
    fn test_expand_env_vars_missing() {
        let (result, missing) = expand_env_vars_in_string("value is ${MISSING_VAR}");
        assert_eq!(result, "value is ${MISSING_VAR}");
        assert_eq!(missing, vec!["MISSING_VAR"]);
    }

    #[test]
    fn test_expand_env_vars_in_map() {
        unsafe { env::set_var("MY_VAR", "my_value") };

        let mut env = std::collections::HashMap::new();
        env.insert("KEY1".to_string(), "${MY_VAR}".to_string());
        env.insert("KEY2".to_string(), "static".to_string());

        let (result, missing) = expand_env_vars_in_map(&env);
        assert_eq!(result.get("KEY1"), Some(&"my_value".to_string()));
        assert_eq!(result.get("KEY2"), Some(&"static".to_string()));
        assert!(missing.is_empty());

        unsafe { env::remove_var("MY_VAR") };
    }
}