//! Semantic boolean parsing utilities.
//!
//! Provides boolean parsing that accepts both boolean values and string literals "true"/"false".
//! This is useful for handling model-generated JSON where booleans might be quoted.

use serde::{Deserialize, Deserializer, Serializer};

/// Parses a value that can be either a boolean or a string "true"/"false".
/// Returns None if the value is neither.
pub fn parse_semantic_bool(value: &serde_json::Value) -> Option<bool> {
    match value {
        serde_json::Value::Bool(b) => Some(*b),
        serde_json::Value::String(s) => match s.as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

/// Coerce a value to a boolean, treating string "true"/"false" as boolean values.
/// Uses default value for other cases.
pub fn coerce_bool(value: &serde_json::Value, default: bool) -> bool {
    parse_semantic_bool(value).unwrap_or(default)
}

/// Custom deserializer that accepts both boolean and string "true"/"false".
#[derive(Debug, Clone, Copy, Default)]
pub struct SemanticBool(bool);

impl SemanticBool {
    pub fn new(value: bool) -> Self {
        Self(value)
    }

    pub fn from_json(value: &serde_json::Value) -> Self {
        Self(parse_semantic_bool(value).unwrap_or(false))
    }

    pub fn get(&self) -> bool {
        self.0
    }
}

impl<'de> Deserialize<'de> for SemanticBool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(Self::from_json(&value))
    }
}

impl Serialize for SemanticBool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(self.0)
    }
}

impl From<SemanticBool> for bool {
    fn from(s: SemanticBool) -> bool {
        s.0
    }
}

impl AsRef<bool> for SemanticBool {
    fn as_ref(&self) -> &bool {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_semantic_bool() {
        assert_eq!(parse_semantic_bool(&serde_json::json!(true)), Some(true));
        assert_eq!(parse_semantic_bool(&serde_json::json!(false)), Some(false));
        assert_eq!(parse_semantic_bool(&serde_json::json!("true")), Some(true));
        assert_eq!(
            parse_semantic_bool(&serde_json::json!("false")),
            Some(false)
        );
        assert_eq!(parse_semantic_bool(&serde_json::json!("invalid")), None);
        assert_eq!(parse_semantic_bool(&serde_json::json!(123)), None);
        assert_eq!(parse_semantic_bool(&serde_json::json!(null)), None);
    }

    #[test]
    fn test_coerce_bool() {
        assert_eq!(coerce_bool(&serde_json::json!(true), false), true);
        assert_eq!(coerce_bool(&serde_json::json!("true"), false), true);
        assert_eq!(coerce_bool(&serde_json::json!("false"), true), false);
        assert_eq!(coerce_bool(&serde_json::json!(123), true), true);
        assert_eq!(coerce_bool(&serde_json::json!(null), false), false);
    }
}
