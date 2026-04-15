use regex::Regex;

const SHORTHAND_START_RE: &str = r"^\s*\+(\d+(?:\.\d+)?)\s*(k|m|b)\b";
const SHORTHAND_END_RE: &str = r"\s\+(\d+(?:\.\d+)?)\s*(k|m|b)\s*[.!?]?\s*$";
const VERBOSE_RE: &str = r"\b(?:use|spend)\s+(\d+(?:\.\d+)?)\s*(k|m|b)\s*tokens?\b";

const MULTIPLIERS: &[(&str, u64); 3] = &[("k", 1_000), ("m", 1_000_000), ("b", 1_000_000_000)];

fn get_multiplier(suffix: &str) -> u64 {
    for (c, m) in MULTIPLIERS {
        if c.eq_ignore_ascii_case(suffix) {
            return *m;
        }
    }
    1
}

fn parse_budget_match(value: &str, suffix: &str) -> u64 {
    let parsed: f64 = value.parse().unwrap_or(0.0);
    (parsed * get_multiplier(suffix) as f64) as u64
}

pub fn parse_token_budget(text: &str) -> Option<u64> {
    let re_start = Regex::new(SHORTHAND_START_RE).unwrap();
    if let Some(caps) = re_start.captures(text) {
        return Some(parse_budget_match(&caps[1], &caps[2]));
    }

    let re_end = Regex::new(SHORTHAND_END_RE).unwrap();
    if let Some(caps) = re_end.captures(text) {
        return Some(parse_budget_match(&caps[1], &caps[2]));
    }

    let re_verbose = Regex::new(VERBOSE_RE).unwrap();
    if let Some(caps) = re_verbose.captures(text) {
        return Some(parse_budget_match(&caps[1], &caps[2]));
    }

    None
}

#[derive(Debug)]
pub struct BudgetPosition {
    pub start: usize,
    pub end: usize,
}

pub fn find_token_budget_positions(text: &str) -> Vec<BudgetPosition> {
    let mut positions = Vec::new();

    let re_start = Regex::new(SHORTHAND_START_RE).unwrap();
    if let Some(m) = re_start.find(text) {
        let offset = m.start() + m.as_str().len() - m.as_str().trim_start().len();
        positions.push(BudgetPosition {
            start: offset,
            end: m.end(),
        });
    }

    let re_end = Regex::new(SHORTHAND_END_RE).unwrap();
    if let Some(m) = re_end.find(text) {
        let end_start = m.start() + 1;
        let already_covered = positions
            .iter()
            .any(|p| end_start >= p.start && end_start < p.end);
        if !already_covered {
            positions.push(BudgetPosition {
                start: end_start,
                end: m.end(),
            });
        }
    }

    let re_verbose_g = Regex::new(&format!("{}g", VERBOSE_RE)).unwrap();
    for m in re_verbose_g.find_iter(text) {
        positions.push(BudgetPosition {
            start: m.start(),
            end: m.end(),
        });
    }

    positions
}

pub fn get_budget_continuation_message(pct: f64, turn_tokens: u64, budget: u64) -> String {
    format!(
        "Stopped at {}% of token target ({} / {}). Keep working — do not summarize.",
        pct, turn_tokens, budget
    )
}
