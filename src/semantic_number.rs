use regex::Regex;
use serde::{Deserialize, Deserializer, Serializer};

lazy_static::lazy_static! {
    static ref NUMBER_REGEX: Regex = Regex::new(r"^-?\d+(\.\d+)?$").unwrap();
}

pub fn parse_semantic_number(value: &serde_json::Value) -> Option<f64> {
    match value {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => {
            if NUMBER_REGEX.is_match(s) {
                let n: f64 = s.parse().ok()?;
                if n.is_finite() {
                    return Some(n);
                }
            }
            None
        }
        _ => None,
    }
}

pub fn coerce_number(value: &serde_json::Value, default: f64) -> f64 {
    parse_semantic_number(value).unwrap_or(default)
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SemanticNumber(f64);

impl SemanticNumber {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn from_json(value: &serde_json::Value) -> Self {
        Self(parse_semantic_number(value).unwrap_or(0.0))
    }

    pub fn get(&self) -> f64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for SemanticNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(Self::from_json(&value))
    }
}

impl Serialize for SemanticNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.0)
    }
}

impl From<SemanticNumber> for f64 {
    fn from(s: SemanticNumber) -> f64 {
        s.0
    }
}

impl AsRef<f64> for SemanticNumber {
    fn as_ref(&self) -> &f64 {
        &self.0
    }
}
