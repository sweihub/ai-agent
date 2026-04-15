// Source: ~/claudecode/openclaudecode/src/utils/plugins/frontmatterParser.ts
//! Frontmatter parser for plugin markdown files.
//!
//! Parses YAML frontmatter (delimited by `---` markers) from markdown content
//! and returns the parsed frontmatter as a JSON object along with the remaining
//! markdown body.

use serde_yaml;

/// Result of parsing frontmatter: (frontmatter map, remaining markdown content)
pub fn parse_frontmatter<'a>(content: &'a str, path: &str) -> (serde_json::Map<String, serde_json::Value>, &'a str) {
    let content = content.trim_start();

    // Content must start with the frontmatter delimiter
    if !content.starts_with("---") {
        return (serde_json::Map::new(), content);
    }

    // Find the closing delimiter
    let rest = &content[3..]; // skip opening "---"
    if let Some(end_pos) = rest.find("\n---") {
        let frontmatter_yaml = &rest[..end_pos];
        let markdown_body = &rest[end_pos + 4..]; // skip "\n---" and newline after

        // Parse the YAML frontmatter
        match serde_yaml::from_str::<serde_yaml::Value>(frontmatter_yaml) {
            Ok(yaml_value) => {
                let fm = yaml_to_json_map(&yaml_value);
                (fm, markdown_body.trim_start())
            }
            Err(e) => {
                log::warn!(
                    "[frontmatter] Failed to parse frontmatter in {}: {}",
                    path,
                    e
                );
                // Return empty frontmatter with full content
                (serde_json::Map::new(), content)
            }
        }
    } else {
        // No closing delimiter found - treat entire content as markdown
        (serde_json::Map::new(), content)
    }
}

/// Convert a serde_yaml::Value to a serde_json::Map<String, serde_json::Value>.
fn yaml_to_json_map(value: &serde_yaml::Value) -> serde_json::Map<String, serde_json::Value> {
    match value {
        serde_yaml::Value::Mapping(map) => {
            let mut result = serde_json::Map::new();
            for (k, v) in map {
                if let Some(key) = k.as_str() {
                    result.insert(key.to_string(), yaml_value_to_json(v));
                }
            }
            result
        }
        _ => serde_json::Map::new(),
    }
}

/// Convert a single serde_yaml::Value to serde_json::Value.
fn yaml_value_to_json(value: &serde_yaml::Value) -> serde_json::Value {
    match value {
        serde_yaml::Value::Null => serde_json::Value::Null,
        serde_yaml::Value::Bool(b) => serde_json::Value::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::Value::Number(serde_json::Number::from(i))
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Null
            }
        }
        serde_yaml::Value::String(s) => serde_json::Value::String(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            serde_json::Value::Array(seq.iter().map(yaml_value_to_json).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                if let Some(key) = k.as_str() {
                    obj.insert(key.to_string(), yaml_value_to_json(v));
                }
            }
            serde_json::Value::Object(obj)
        }
        serde_yaml::Value::Tagged(tagged) => {
            // Handle tagged values (e.g., !!str "value") - unwrap the inner value
            yaml_value_to_json(&tagged.value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_basic() {
        let content = "---\nname: My Agent\ndescription: A test agent\n---\n\n# Agent body";
        let (fm, body) = parse_frontmatter(content, "test.md");
        assert_eq!(fm.get("name").and_then(|v| v.as_str()), Some("My Agent"));
        assert_eq!(fm.get("description").and_then(|v| v.as_str()), Some("A test agent"));
        assert_eq!(body, "# Agent body");
    }

    #[test]
    fn test_parse_frontmatter_no_frontmatter() {
        let content = "# Just markdown";
        let (fm, body) = parse_frontmatter(content, "test.md");
        assert!(fm.is_empty());
        assert_eq!(body, "# Just markdown");
    }

    #[test]
    fn test_parse_frontmatter_no_closing_delimiter() {
        let content = "---\nname: unclosed";
        let (fm, body) = parse_frontmatter(content, "test.md");
        assert!(fm.is_empty());
        assert_eq!(body, "---\nname: unclosed");
    }

    #[test]
    fn test_parse_frontmatter_array_value() {
        let content = "---\nname: Test\ntools:\n  - Read\n  - Bash\n---\n\nBody";
        let (fm, _body) = parse_frontmatter(content, "test.md");
        let tools = fm.get("tools").and_then(|v| v.as_array());
        assert!(tools.is_some());
        let tools = tools.unwrap();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].as_str(), Some("Read"));
        assert_eq!(tools[1].as_str(), Some("Bash"));
    }

    #[test]
    fn test_parse_frontmatter_empty() {
        let content = "";
        let (fm, body) = parse_frontmatter(content, "test.md");
        assert!(fm.is_empty());
        assert_eq!(body, "");
    }

    #[test]
    fn test_parse_frontmatter_leading_whitespace() {
        let content = "  \n---\nname: Test\n---\nBody";
        let (fm, body) = parse_frontmatter(content, "test.md");
        assert_eq!(fm.get("name").and_then(|v| v.as_str()), Some("Test"));
        assert_eq!(body, "Body");
    }

    #[test]
    fn test_parse_frontmatter_boolean_value() {
        let content = "---\nname: Test\nbackground: true\n---\n\nBody";
        let (fm, _body) = parse_frontmatter(content, "test.md");
        assert_eq!(fm.get("background").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn test_parse_frontmatter_nested_object() {
        let content = "---\nname: Test\nconfig:\n  key: value\n---\n\nBody";
        let (fm, _body) = parse_frontmatter(content, "test.md");
        let config = fm.get("config").and_then(|v| v.as_object());
        assert!(config.is_some());
        let config = config.unwrap();
        assert_eq!(config.get("key").and_then(|v| v.as_str()), Some("value"));
    }
}
