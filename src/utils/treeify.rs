// Source: /data/home/swei/claudecode/openclaudecode/src/utils/treeify.ts
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub value: serde_json::Value,
}

impl TreeNode {
    pub fn new(value: serde_json::Value) -> Self {
        Self { value }
    }
}

#[derive(Debug, Clone)]
pub struct TreeifyOptions {
    pub show_values: bool,
    pub hide_functions: bool,
}

impl Default for TreeifyOptions {
    fn default() -> Self {
        Self {
            show_values: true,
            hide_functions: false,
        }
    }
}

const BRANCH: &str = "├";
const LAST_BRANCH: &str = "└";
const LINE: &str = "│";
const EMPTY: &str = " ";

pub fn treeify(obj: &serde_json::Value, options: TreeifyOptions) -> String {
    match obj {
        serde_json::Value::Object(map) => treeify_object(map, &options, "", true),
        serde_json::Value::Array(arr) => treeify_array(arr, &options, "", true),
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn treeify_object(
    obj: &serde_json::Map<String, serde_json::Value>,
    options: &TreeifyOptions,
    prefix: &str,
    is_last: bool,
) -> String {
    let mut result = String::new();
    let keys: Vec<&String> = obj.keys().collect();

    if keys.is_empty() {
        return "(empty)".to_string();
    }

    for (i, key) in keys.iter().enumerate() {
        let is_last_key = i == keys.len() - 1;
        let value = obj.get(key).unwrap();
        let tree_char = if is_last_key { LAST_BRANCH } else { BRANCH };
        let continuation = if is_last_key { EMPTY } else { LINE };

        let current_prefix = if i == 0 { "" } else { prefix };

        if let serde_json::Value::Object(nested) = value {
            writeln!(result, "{}{} {}", current_prefix, tree_char, key).ok();
            let next_prefix = format!("{}{} ", current_prefix, continuation);
            result.push_str(&treeify_object(nested, options, &next_prefix, true));
        } else if let serde_json::Value::Array(arr) = value {
            writeln!(
                result,
                "{}{} {}: [Array({})]",
                current_prefix,
                tree_char,
                key,
                arr.len()
            )
            .ok();
        } else if options.show_values {
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => "null".to_string(),
                other => other.to_string(),
            };
            writeln!(
                result,
                "{}{} {}: {}",
                current_prefix, tree_char, key, value_str
            )
            .ok();
        }
    }

    result
}

fn treeify_array(
    arr: &[serde_json::Value],
    options: &TreeifyOptions,
    prefix: &str,
    is_last: bool,
) -> String {
    if arr.is_empty() {
        return "[]".to_string();
    }

    let mut result = String::new();

    for (i, item) in arr.iter().enumerate() {
        let is_last_item = i == arr.len() - 1;
        let tree_char = if is_last_item { LAST_BRANCH } else { BRANCH };
        let continuation = if is_last_item { EMPTY } else { LINE };

        match item {
            serde_json::Value::Object(map) => {
                writeln!(result, "{}{} [{}]", prefix, tree_char, i).ok();
                let next_prefix = format!("{}{} ", prefix, continuation);
                result.push_str(&treeify_object(map, options, &next_prefix, is_last_item));
            }
            serde_json::Value::Array(nested) => {
                writeln!(
                    result,
                    "{}{} [{}]: [Array({})]",
                    prefix,
                    tree_char,
                    i,
                    nested.len()
                )
                .ok();
            }
            other if options.show_values => {
                writeln!(result, "{}{} [{}]: {}", prefix, tree_char, i, other).ok();
            }
            _ => {}
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_treeify_simple() {
        let obj = serde_json::json!({
            "a": "value1",
            "b": "value2"
        });
        let result = treeify(&obj, TreeifyOptions::default());
        assert!(result.contains("a"));
        assert!(result.contains("b"));
    }
}
