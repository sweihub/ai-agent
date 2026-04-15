use regex::Regex;

static ANSI_ESCAPE_RE: once_cell::sync::Lazy<Regex> =
    once_cell::sync::Lazy::new(|| Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap());

static ANSI_CSI_RE: once_cell::sync::Lazy<Regex> =
    once_cell::sync::Lazy::new(|| Regex::new(r"\x1b\[([0-9;]*)[A-Za-z]").unwrap());

static ANSI_OSC_RE: once_cell::sync::Lazy<Regex> =
    once_cell::sync::Lazy::new(|| Regex::new(r"\x1b\]([^\x07]*)(\x07|\x1b\\)").unwrap());

pub fn strip_ansi_codes(s: &str) -> String {
    ANSI_ESCAPE_RE.replace_all(s, "").to_string()
}

pub fn strip_ansi_codes_with_lengths(s: &str) -> (String, Vec<(usize, usize)>) {
    let mut result = String::with_capacity(s.len());
    let mut positions = Vec::new();
    let mut last_end = 0;

    for cap in ANSI_CSI_RE.captures_iter(s) {
        let m = cap.get(0).unwrap();
        result.push_str(&s[last_end..m.start()]);
        positions.push((m.start(), m.end()));
        last_end = m.end();
    }
    result.push_str(&s[last_end..]);

    (result, positions)
}

pub fn measure_text_width(s: &str) -> usize {
    strip_ansi_codes(s).chars().count()
}

pub fn is_ansi_escape_sequence(s: &str) -> bool {
    ANSI_ESCAPE_RE.is_match(s)
}

pub fn parse_ansi_params(s: &str) -> Option<Vec<u32>> {
    if let Some(cap) = ANSI_OSC_RE.captures(s) {
        let params_str = cap.get(1)?.as_str();
        if params_str.is_empty() {
            return Some(vec![]);
        }
        let params: Vec<u32> = params_str
            .split(';')
            .filter_map(|p| p.parse().ok())
            .collect();
        return Some(params);
    }
    None
}
