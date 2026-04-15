// Source: ~/claudecode/openclaudecode/src/utils/tokenBudget.ts

use regex::Regex;
use std::sync::LazyLock;

/// Shorthand (+500k) anchored to start/end to avoid false positives in natural language.
/// Verbose (use/spend 2M tokens) matches anywhere.
static SHORTHAND_START_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*\+(\d+(?:\.\d+)?)\s*(k|m|b)\b").unwrap());

static SHORTHAND_END_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\s\+(\d+(?:\.\d+)?)\s*(k|m|b)\s*[.!?]?\s*$").unwrap()
});

static VERBOSE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:use|spend)\s+(\d+(?:\.\d+)?)\s*(k|m|b)\s*tokens?\b").unwrap()
});

static VERBOSE_RE_G: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:use|spend)\s+(\d+(?:\.\d+)?)\s*(k|m|b)\s*tokens?\b").unwrap()
});

const MULTIPLIERS: &[(&str, f64)] = &[("k", 1_000.0), ("m", 1_000_000.0), ("b", 1_000_000_000.0)];

fn parse_budget_match(value: &str, suffix: &str) -> f64 {
    let value: f64 = value.parse().unwrap_or(0.0);
    let multiplier = MULTIPLIERS
        .iter()
        .find(|(s, _)| *s == suffix.to_lowercase())
        .map(|(_, m)| *m)
        .unwrap_or(1.0);
    value * multiplier
}

/// Parse a token budget from text.
pub fn parse_token_budget(text: &str) -> Option<f64> {
    // Check start shorthand
    if let Some(caps) = SHORTHAND_START_RE.captures(text) {
        let value = caps.get(1)?.as_str();
        let suffix = caps.get(2)?.as_str();
        return Some(parse_budget_match(value, suffix));
    }

    // Check end shorthand
    if let Some(caps) = SHORTHAND_END_RE.captures(text) {
        let value = caps.get(1)?.as_str();
        let suffix = caps.get(2)?.as_str();
        return Some(parse_budget_match(value, suffix));
    }

    // Check verbose
    if let Some(caps) = VERBOSE_RE.captures(text) {
        let value = caps.get(1)?.as_str();
        let suffix = caps.get(2)?.as_str();
        return Some(parse_budget_match(value, suffix));
    }

    None
}

/// Find positions of token budget mentions in text.
pub fn find_token_budget_positions(text: &str) -> Vec<(usize, usize)> {
    let mut positions = Vec::new();

    // Check start shorthand
    if let Some(m) = SHORTHAND_START_RE.find(text) {
        let offset = m.start() + m.as_str().len() - m.as_str().trim_start().len();
        positions.push((offset, m.end()));
    }

    // Check end shorthand
    if let Some(m) = SHORTHAND_END_RE.find(text) {
        let end_start = m.start() + 1; // +1: regex includes leading \s
        let already_covered = positions
            .iter()
            .any(|(start, end)| end_start >= *start && end_start < *end);
        if !already_covered {
            positions.push((end_start, m.end()));
        }
    }

    // Check verbose (all matches)
    for m in VERBOSE_RE_G.find_iter(text) {
        positions.push((m.start(), m.end()));
    }

    positions
}

/// Get budget continuation message.
pub fn get_budget_contination_message(pct: f64, turn_tokens: u64, budget: f64) -> String {
    format!(
        "Stopped at {pct}% of token target ({turn_tokens} / {budget}). Keep working \u{2014} do not summarize."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_token_budget_shorthand_start() {
        assert_eq!(parse_token_budget("+500k"), Some(500_000.0));
        assert_eq!(parse_token_budget("+2m"), Some(2_000_000.0));
        assert_eq!(parse_token_budget("+1.5b"), Some(1_500_000_000.0));
    }

    #[test]
    fn test_parse_token_budget_shorthand_end() {
        assert_eq!(parse_token_budget("I want +500k."), Some(500_000.0));
    }

    #[test]
    fn test_parse_token_budget_verbose() {
        assert_eq!(
            parse_token_budget("use 2M tokens"),
            Some(2_000_000.0)
        );
        assert_eq!(
            parse_token_budget("spend 500k tokens"),
            Some(500_000.0)
        );
    }

    #[test]
    fn test_parse_token_budget_none() {
        assert!(parse_token_budget("hello world").is_none());
    }

    #[test]
    fn test_find_positions() {
        let positions = find_token_budget_positions("+500k");
        assert!(!positions.is_empty());
    }

    #[test]
    fn test_budget_continuation_message() {
        let msg = get_budget_contination_message(80.0, 160000, 200000);
        assert!(msg.contains("80%"));
        assert!(msg.contains("160000"));
        assert!(msg.contains("200000"));
    }
}
