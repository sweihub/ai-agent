// Source: ~/claudecode/openclaudecode/src/utils/envValidation.rs

use serde::Serialize;

/// Result of validating a bounded integer environment variable.
#[derive(Debug, Clone, Serialize)]
pub struct EnvVarValidationResult {
    /// The effective value to use.
    pub effective: i64,
    /// Validation status.
    pub status: ValidationStatus,
    /// Optional message describing what happened.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Validation status for environment variable.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Valid,
    Capped,
    Invalid,
}

/// Validate a bounded integer environment variable.
pub fn validate_bounded_int_env_var(
    name: &str,
    value: Option<&str>,
    default_value: i64,
    upper_limit: i64,
) -> EnvVarValidationResult {
    let Some(value) = value else {
        return EnvVarValidationResult {
            effective: default_value,
            status: ValidationStatus::Valid,
            message: None,
        };
    };

    let parsed = value.parse::<i64>();
    if let Ok(parsed) = parsed {
        if parsed <= 0 {
            let message =
                format!("Invalid value \"{}\" (using default: {})", value, default_value);
            eprintln!("{} {}", name, message);
            return EnvVarValidationResult {
                effective: default_value,
                status: ValidationStatus::Invalid,
                message: Some(message),
            };
        }

        if parsed > upper_limit {
            let message = format!("Capped from {} to {}", parsed, upper_limit);
            eprintln!("{} {}", name, message);
            return EnvVarValidationResult {
                effective: upper_limit,
                status: ValidationStatus::Capped,
                message: Some(message),
            };
        }

        return EnvVarValidationResult {
            effective: parsed,
            status: ValidationStatus::Valid,
            message: None,
        };
    }

    let message = format!("Invalid value \"{}\" (using default: {})", value, default_value);
    eprintln!("{} {}", name, message);
    EnvVarValidationResult {
        effective: default_value,
        status: ValidationStatus::Invalid,
        message: Some(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_when_none() {
        let result = validate_bounded_int_env_var("TEST_VAR", None, 10, 100);
        assert_eq!(result.effective, 10);
        assert!(matches!(result.status, ValidationStatus::Valid));
    }

    #[test]
    fn test_valid_value() {
        let result = validate_bounded_int_env_var("TEST_VAR", Some("50"), 10, 100);
        assert_eq!(result.effective, 50);
        assert!(matches!(result.status, ValidationStatus::Valid));
    }

    #[test]
    fn test_capped_value() {
        let result = validate_bounded_int_env_var("TEST_VAR", Some("200"), 10, 100);
        assert_eq!(result.effective, 100);
        assert!(matches!(result.status, ValidationStatus::Capped));
    }

    #[test]
    fn test_invalid_value() {
        let result = validate_bounded_int_env_var("TEST_VAR", Some("abc"), 10, 100);
        assert_eq!(result.effective, 10);
        assert!(matches!(result.status, ValidationStatus::Invalid));
    }

    #[test]
    fn test_negative_value() {
        let result = validate_bounded_int_env_var("TEST_VAR", Some("-5"), 10, 100);
        assert_eq!(result.effective, 10);
        assert!(matches!(result.status, ValidationStatus::Invalid));
    }
}
