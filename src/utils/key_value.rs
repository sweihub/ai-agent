#![allow(dead_code)]

use std::collections::HashMap;

pub fn parse_key_value(s: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in s.lines() {
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    map
}

pub fn to_key_value(map: &HashMap<String, String>) -> String {
    map.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let map = parse_key_value("a=1\nb=2");
        assert_eq!(map.get("a"), Some(&"1".to_string()));
    }
}
