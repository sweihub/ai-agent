// Source: /data/home/swei/claudecode/openclaudecode/src/utils/yaml.ts
//! YAML parsing wrapper.
//!
//! Uses serde_yaml for parsing YAML content.

use serde_yaml::Value;

/// Parse YAML string into a generic value.
/// Returns a serde_yaml::Value that can be further processed.
pub fn parse_yaml(input: &str) -> Result<Value, serde_yaml::Error> {
    serde_yaml::from_str(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml() {
        let yaml = "key: value\nlist:\n  - item1\n  - item2";
        let result = parse_yaml(yaml).unwrap();
        assert!(result.get("key").is_some());
    }
}
