use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarValidationResult {
    pub effective: i64,
    pub status: EnvVarValidationStatus,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnvVarValidationStatus {
    Valid,
    Capped,
    Invalid,
}

pub fn validate_bounded_int_env_var(
    name: &str,
    value: Option<&str>,
    default_value: i64,
    upper_limit: i64,
) -> EnvVarValidationResult {
    let value = match value {
        Some(v) => v,
        None => {
            return EnvVarValidationResult {
                effective: default_value,
                status: EnvVarValidationStatus::Valid,
                message: None,
            }
        }
    };

    let parsed: i64 = match value.parse() {
        Ok(p) => p,
        Err(_) => {
            return EnvVarValidationResult {
                effective: default_value,
                status: EnvVarValidationStatus::Invalid,
                message: Some(format!(
                    "Invalid value \"{}\" (using default: {})",
                    value, default_value
                )),
            };
        }
    };

    if parsed <= 0 {
        return EnvVarValidationResult {
            effective: default_value,
            status: EnvVarValidationStatus::Invalid,
            message: Some(format!(
                "Invalid value \"{}\" (using default: {})",
                value, default_value
            )),
        };
    }

    if parsed > upper_limit {
        return EnvVarValidationResult {
            effective: upper_limit,
            status: EnvVarValidationStatus::Capped,
            message: Some(format!("Capped from {} to {}", parsed, upper_limit)),
        };
    }

    EnvVarValidationResult {
        effective: parsed,
        status: EnvVarValidationStatus::Valid,
        message: None,
    }
}
