#![allow(dead_code)]

pub fn parse_json<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(s)
}

pub fn to_json<T: serde::Serialize>(v: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(v)
}

pub fn to_json_pretty<T: serde::Serialize>(v: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json() {
        let v: HashMap<String, i32> = [("a".to_string(), 1)].into_iter().collect();
        let s = to_json(&v).unwrap();
        assert!(s.contains("a"));
        let v2: HashMap<String, i32> = parse_json(&s).unwrap();
        assert_eq!(v2.get("a"), Some(&1));
    }
}
