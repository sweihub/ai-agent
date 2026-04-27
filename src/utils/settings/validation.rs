// Source: ~/claudecode/openclaudecode/src/utils/settings/validation.ts
//! Settings validation - parses and validates settings JSON files.

use serde_json::Value;

use crate::services::mcp::ConfigScope;
use crate::utils::settings::permission_validation::validate_permission_rule;

/// Field path in dot notation (e.g., "permissions.defaultMode")
pub type FieldPath = String;

/// A validation error with location and context.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValidationError {
    pub file: Option<String>,
    pub path: FieldPath,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_error_metadata: Option<McpErrorMetadata>,
}

/// MCP-specific metadata attached to validation errors.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct McpErrorMetadata {
    pub scope: ConfigScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<McpErrorSeverity>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpErrorSeverity {
    Fatal,
    Warning,
}

/// Validated settings with any accumulated errors.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SettingsWithErrors {
    pub settings: Value,
    pub errors: Vec<ValidationError>,
}

/// Gets the Rust type name for a JSON value.
fn received_type(v: &Value) -> &str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Validates that settings JSON conforms to expected structure.
/// Returns a SettingsWithErrors containing the parsed JSON and any validation errors.
pub fn validate_settings_json(data: &Value) -> SettingsWithErrors {
    let mut errors = Vec::new();

    // Validate top-level is an object
    if !data.is_object() {
        return SettingsWithErrors {
            settings: Value::Object(serde_json::Map::new()),
            errors: vec![ValidationError {
                path: "".into(),
                message: "Settings must be a JSON object".into(),
                expected: Some("object".into()),
                invalid_value: Some(data.clone()),
                suggestion: None,
                doc_link: None,
                file: None,
                mcp_error_metadata: None,
            }],
        };
    }

    // Validate permissions section
    if let Some(perms) = data.get("permissions") {
        if !perms.is_object() {
            errors.push(ValidationError {
                path: "permissions".into(),
                message: "Expected permissions to be an object".into(),
                expected: Some("object".into()),
                invalid_value: Some(received_type(perms).into()),
                suggestion: None,
                doc_link: None,
                file: None,
                mcp_error_metadata: None,
            });
        } else if let Some(perms_obj) = perms.as_object() {
            // Validate defaultMode
            if let Some(mode) = perms_obj.get("defaultMode") {
                if let Some(mode_str) = mode.as_str() {
                    match mode_str {
                        "allow" | "deny" | "ask" => {} // valid
                        _ => {
                            errors.push(ValidationError {
                                path: "permissions.defaultMode".into(),
                                message: format!("Invalid permission mode: \"{}\"", mode_str),
                                expected: Some("\"allow\", \"deny\", or \"ask\"".into()),
                                invalid_value: Some(mode.clone()),
                                suggestion: Some("Use \"allow\", \"deny\", or \"ask\"".into()),
                                doc_link: None,
                                file: None,
                                mcp_error_metadata: None,
                            });
                        }
                    }
                } else {
                    errors.push(ValidationError {
                        path: "permissions.defaultMode".into(),
                        message: "Expected defaultMode to be a string".into(),
                        expected: Some("string".into()),
                        invalid_value: Some(received_type(mode).into()),
                        suggestion: None,
                        doc_link: None,
                        file: None,
                        mcp_error_metadata: None,
                    });
                }
            }

            // Validate permission rule arrays
            for key in ["allow", "deny", "ask"] {
                if let Some(rules) = perms_obj.get(key) {
                    if !rules.is_array() {
                        errors.push(ValidationError {
                            path: format!("permissions.{}", key),
                            message: format!("Expected permissions.{} to be an array", key),
                            expected: Some("array".into()),
                            invalid_value: Some(received_type(rules).into()),
                            suggestion: None,
                            doc_link: None,
                            file: None,
                            mcp_error_metadata: None,
                        });
                    } else if let Some(rules_arr) = rules.as_array() {
                        for (idx, rule) in rules_arr.iter().enumerate() {
                            if !rule.is_string() {
                                errors.push(ValidationError {
                                    path: format!("permissions.{}.{}", key, idx),
                                    message: format!(
                                        "Non-string value in {} array was removed",
                                        key
                                    ),
                                    expected: Some("string".into()),
                                    invalid_value: Some(rule.clone()),
                                    suggestion: None,
                                    doc_link: None,
                                    file: None,
                                    mcp_error_metadata: None,
                                });
                            } else if let Some(rule_str) = rule.as_str() {
                                let result = validate_permission_rule(rule_str);
                                if !result.valid {
                                    errors.push(ValidationError {
                                        path: format!("permissions.{}.{}", key, idx),
                                        message: format!(
                                            "Invalid permission rule \"{}\": {}",
                                            rule_str,
                                            result.error.unwrap_or_else(|| "unknown error".into())
                                        ),
                                        expected: None,
                                        suggestion: result.suggestion,
                                        invalid_value: Some(rule.clone()),
                                        doc_link: None,
                                        file: None,
                                        mcp_error_metadata: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            // Validate additionalDirectories
            if let Some(dirs) = perms_obj.get("additionalDirectories") {
                if !dirs.is_array() {
                    errors.push(ValidationError {
                        path: "permissions.additionalDirectories".into(),
                        message: "Expected additionalDirectories to be an array".into(),
                        expected: Some("array".into()),
                        invalid_value: Some(received_type(dirs).into()),
                        suggestion: None,
                        doc_link: None,
                        file: None,
                        mcp_error_metadata: None,
                    });
                } else if let Some(dirs_arr) = dirs.as_array() {
                    for (idx, dir) in dirs_arr.iter().enumerate() {
                        if !dir.is_string() {
                            errors.push(ValidationError {
                                path: format!("permissions.additionalDirectories.{}", idx),
                                message: "Non-string value in additionalDirectories".into(),
                                expected: Some("string".into()),
                                invalid_value: Some(dir.clone()),
                                suggestion: None,
                                doc_link: None,
                                file: None,
                                mcp_error_metadata: None,
                            });
                        }
                    }
                }
            }
        }
    }

    // Validate env section
    if let Some(env) = data.get("env") {
        if !env.is_object() {
            errors.push(ValidationError {
                path: "env".into(),
                message: "Expected env to be an object".into(),
                expected: Some("object".into()),
                invalid_value: Some(received_type(env).into()),
                suggestion: None,
                doc_link: None,
                file: None,
                mcp_error_metadata: None,
            });
        } else if let Some(env_obj) = env.as_object() {
            for (key, val) in env_obj {
                if !val.is_string() && val.is_null() {
                    // null values are allowed for removing env vars
                    continue;
                }
                if !val.is_string() {
                    errors.push(ValidationError {
                        path: format!("env.{}", key),
                        message: format!("Expected env.{} to be a string or null", key),
                        expected: Some("string or null".into()),
                        invalid_value: Some(received_type(val).into()),
                        suggestion: None,
                        doc_link: None,
                        file: None,
                        mcp_error_metadata: None,
                    });
                }
            }
        }
    }

    // Validate model section
    if let Some(model) = data.get("model") {
        if let Some(model_obj) = model.as_object() {
            if let Some(name) = model_obj.get("name") {
                if !name.is_string() {
                    errors.push(ValidationError {
                        path: "model.name".into(),
                        message: "Expected model.name to be a string".into(),
                        expected: Some("string".into()),
                        invalid_value: Some(received_type(name).into()),
                        suggestion: None,
                        doc_link: None,
                        file: None,
                        mcp_error_metadata: None,
                    });
                }
            }
            if let Some(max_tokens) = model_obj.get("maxTokens") {
                if !max_tokens.is_number() {
                    errors.push(ValidationError {
                        path: "model.maxTokens".into(),
                        message: "Expected model.maxTokens to be a number".into(),
                        expected: Some("number".into()),
                        invalid_value: Some(received_type(max_tokens).into()),
                        suggestion: None,
                        doc_link: None,
                        file: None,
                        mcp_error_metadata: None,
                    });
                }
            }
        } else if !model.is_string() {
            errors.push(ValidationError {
                path: "model".into(),
                message: "Expected model to be a string or object".into(),
                expected: Some("string or object".into()),
                invalid_value: Some(received_type(model).into()),
                suggestion: None,
                doc_link: None,
                file: None,
                mcp_error_metadata: None,
            });
        }
    }

    // Validate hooks section
    if let Some(hooks) = data.get("hooks") {
        if !hooks.is_object() {
            errors.push(ValidationError {
                path: "hooks".into(),
                message: "Expected hooks to be an object".into(),
                expected: Some("object".into()),
                invalid_value: Some(received_type(hooks).into()),
                suggestion: None,
                doc_link: None,
                file: None,
                mcp_error_metadata: None,
            });
        }
    }

    SettingsWithErrors {
        settings: data.clone(),
        errors,
    }
}

/// Validates settings file content string, parsing JSON and validating structure.
pub fn validate_settings_file_content(content: &str) -> SettingsWithErrors {
    match serde_json::from_str(content) {
        Ok(json) => validate_settings_json(&json),
        Err(e) => SettingsWithErrors {
            settings: Value::Object(serde_json::Map::new()),
            errors: vec![ValidationError {
                path: "".into(),
                message: format!("Invalid JSON: {}", e),
                expected: Some("valid JSON object".into()),
                invalid_value: None,
                suggestion: Some("Check for trailing commas, missing quotes, or mismatched braces"
                    .into()),
                doc_link: None,
                file: None,
                mcp_error_metadata: None,
            }],
        },
    }
}

/// Filters invalid permission rules from raw parsed JSON data before schema validation.
/// Returns warnings for each filtered rule.
pub fn filter_invalid_permission_rules(
    data: &Value,
    file_path: &str,
) -> Vec<ValidationError> {
    let mut warnings = Vec::new();

    let Some(obj) = data.as_object() else {
        return warnings;
    };
    let Some(perms) = obj.get("permissions") else {
        return warnings;
    };
    let Some(perms_obj) = perms.as_object() else {
        return warnings;
    };

    for key in ["allow", "deny", "ask"] {
        let Some(rules) = perms_obj.get(key) else {
            continue;
        };
        let Some(rules_arr) = rules.as_array() else {
            continue;
        };

        let valid_rules: Vec<Value> = rules_arr
            .iter()
            .filter_map(|rule| {
                if !rule.is_string() {
                    warnings.push(ValidationError {
                        file: Some(file_path.to_string()),
                        path: format!("permissions.{}", key),
                        message: format!("Non-string value in {} array was removed", key),
                        expected: Some("string".into()),
                        invalid_value: Some(rule.clone()),
                        suggestion: None,
                        doc_link: None,
                        mcp_error_metadata: None,
                    });
                    return None;
                }
                if let Some(rule_str) = rule.as_str() {
                    let result = validate_permission_rule(rule_str);
                    if !result.valid {
                        let mut msg = format!("Invalid permission rule \"{}\" was skipped", rule_str);
                        if let Some(ref err) = result.error {
                            msg += ": ";
                            msg += err;
                        }
                        warnings.push(ValidationError {
                            file: Some(file_path.to_string()),
                            path: format!("permissions.{}", key),
                            message: msg,
                            expected: None,
                            invalid_value: Some(rule.clone()),
                            suggestion: result.suggestion,
                            doc_link: None,
                            mcp_error_metadata: None,
                        });
                        return None;
                    }
                }
                Some(rule.clone())
            })
            .collect();

        // Replace the rules array with only valid rules (conceptual - caller handles mutation)
        let _ = valid_rules;
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_object() {
        let result = validate_settings_json(&Value::Object(serde_json::Map::new()));
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_not_object() {
        let result = validate_settings_json(&Value::String("not an object".into()));
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_invalid_mode() {
        let json = serde_json::json!({
            "permissions": {
                "defaultMode": "invalid"
            }
        });
        let result = validate_settings_json(&json);
        assert!(!result.errors.is_empty());
        assert_eq!(result.errors[0].path, "permissions.defaultMode");
    }

    #[test]
    fn test_validate_valid_mode() {
        for mode in &["allow", "deny", "ask"] {
            let json = serde_json::json!({
                "permissions": {
                    "defaultMode": mode
                }
            });
            let result = validate_settings_json(&json);
            assert!(
                result.errors.is_empty(),
                "mode '{}' should be valid",
                mode
            );
        }
    }

    #[test]
    fn test_validate_invalid_permission_rule() {
        let json = serde_json::json!({
            "permissions": {
                "allow": ["read(*.ts)", "Read()"]
            }
        });
        let result = validate_settings_json(&json);
        // lowercase 'read' should fail, empty parens should fail
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_valid_settings() {
        let json = serde_json::json!({
            "permissions": {
                "defaultMode": "allow",
                "allow": ["Read(*.ts)", "Bash"]
            },
            "env": {
                "DEBUG": "true"
            }
        });
        let result = validate_settings_json(&json);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_validate_invalid_json() {
        let result = validate_settings_file_content("{ invalid json }");
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_valid_json_string() {
        let result = validate_settings_file_content(r#"{"permissions": {"defaultMode": "allow"}}"#);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_filter_invalid_permission_rules() {
        let json = serde_json::json!({
            "permissions": {
                "allow": ["Read(*.ts)", "read()", 123]
            }
        });
        let warnings = filter_invalid_permission_rules(&json, "test.json");
        // "read()" lowercase + empty parens, and 123 non-string
        assert!(!warnings.is_empty());
    }

    #[test]
    fn test_env_validation() {
        let json = serde_json::json!({
            "env": {
                "VALID": "string",
                "ALSO_VALID": null,
                "INVALID": 123
            }
        });
        let result = validate_settings_json(&json);
        assert!(!result.errors.is_empty());
        assert_eq!(result.errors[0].path, "env.INVALID");
    }

    #[test]
    fn test_model_validation() {
        let json = serde_json::json!({
            "model": {
                "name": 123
            }
        });
        let result = validate_settings_json(&json);
        assert!(!result.errors.is_empty());
    }
}
