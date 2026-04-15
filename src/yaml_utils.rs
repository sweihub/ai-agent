use serde::{Deserialize, Serialize};

pub fn parse_yaml<T: for<'de> Deserialize<'de>>(input: &str) -> Result<T, serde_yaml::Error> {
    serde_yaml::from_str(input)
}

pub fn to_yaml<T: Serialize>(value: &T) -> Result<String, serde_yaml::Error> {
    serde_yaml::to_string(value)
}

pub fn to_yaml_string(value: &impl Serialize) -> String {
    serde_yaml::to_string(value).unwrap_or_default()
}
