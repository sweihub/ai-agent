use regex::Regex;

pub fn is_match(pattern: &str, text: &str) -> bool {
    Regex::new(pattern)
        .map(|re| re.is_match(text))
        .unwrap_or(false)
}

pub fn find_match(pattern: &str, text: &str) -> Option<String> {
    Regex::new(pattern)
        .ok()
        .and_then(|re| re.find(text).map(|m| m.as_str().to_string()))
}

pub fn find_all_matches(pattern: &str, text: &str) -> Vec<String> {
    Regex::new(pattern)
        .ok()
        .map(|re| re.find_iter(text).map(|m| m.as_str().to_string()).collect())
        .unwrap_or_default()
}

pub fn replace_all(pattern: &str, text: &str, replacement: &str) -> String {
    Regex::new(pattern)
        .map(|re| re.replace_all(text, replacement).to_string())
        .unwrap_or_else(|_| text.to_string())
}

pub fn capture_groups(pattern: &str, text: &str) -> Vec<Option<String>> {
    Regex::new(pattern)
        .ok()
        .and_then(|re| re.captures(text))
        .map(|caps| {
            caps.iter()
                .skip(1)
                .map(|m| m.map(|m| m.as_str().to_string()))
                .collect()
        })
        .unwrap_or_default()
}

pub fn split_by_pattern(pattern: &str, text: &str) -> Vec<String> {
    Regex::new(pattern)
        .map(|re| re.split(text).map(|s| s.to_string()).collect())
        .unwrap_or_else(|_| vec![text.to_string()])
}
