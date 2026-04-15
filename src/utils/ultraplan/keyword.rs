// Source: /data/home/swei/claudecode/openclaudecode/src/utils/ultraplan/keyword.ts
use regex::Regex;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct TriggerPosition {
    pub word: String,
    pub start: usize,
    pub end: usize,
}

fn find_keyword_trigger_positions(text: &str, keyword: &str) -> Vec<TriggerPosition> {
    let re = Regex::new(&format!("(?i){}", keyword)).unwrap();
    if !re.is_match(text) {
        return Vec::new();
    }

    if text.starts_with('/') {
        return Vec::new();
    }

    let open_to_close: HashSet<char> = ['`', '"', '<', '{', '[', '(', '\''].into();

    let mut quoted_ranges: Vec<(usize, usize)> = Vec::new();
    let mut open_quote: Option<char> = None;
    let mut open_at = 0;

    let is_word = |ch: Option<char>| ch.map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);

    for (i, ch) in text.char_indices() {
        if let Some(quote) = open_quote {
            if quote == '[' && ch == '[' {
                open_at = i;
                continue;
            }

            let close = match quote {
                '`' => '`',
                '"' => '"',
                '<' => '>',
                '{' => '}',
                '[' => ']',
                '(' => ')',
                '\'' => '\'',
                _ => continue,
            };

            if ch != close {
                continue;
            }

            if quote == '\'' && is_word(text.chars().nth(i + 1)) {
                continue;
            }

            quoted_ranges.push((open_at, i + 1));
            open_quote = None;
        } else if (ch == '<'
            && text
                .chars()
                .nth(i + 1)
                .map(|c| c.is_alphabetic() || c == '/')
                .unwrap_or(false))
            || (ch == '\'' && !is_word(text.chars().nth(i)))
            || (ch != '<' && ch != '\'' && open_to_close.contains(&ch))
        {
            open_quote = Some(ch);
            open_at = i;
        }
    }

    let quoted_set: HashSet<usize> = quoted_ranges
        .iter()
        .flat_map(|(start, end)| (*start..*end))
        .collect();

    let word_re = Regex::new(&format!(r"\b{}\b", keyword)).unwrap();
    let mut positions = Vec::new();

    for cap in word_re.captures_iter(text) {
        let start = cap.get(0).map(|m| m.start()).unwrap_or(0);
        let end = cap.get(0).map(|m| m.end()).unwrap_or(0);

        if quoted_set.contains(&start) {
            continue;
        }

        let before = text.chars().nth(start.saturating_sub(1));
        let after = text.chars().nth(end);

        if let Some(c) = before {
            if c == '/' || c == '\\' || c == '-' {
                continue;
            }
        }

        if let Some(c) = after {
            if c == '/' || c == '\\' || c == '-' || c == '?' {
                continue;
            }
            if c == '.' && is_word(text.chars().nth(end + 1)) {
                continue;
            }
        }

        positions.push(TriggerPosition {
            word: cap
                .get(0)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default(),
            start,
            end,
        });
    }

    positions
}

pub fn find_ultraplan_trigger_positions(text: &str) -> Vec<TriggerPosition> {
    find_keyword_trigger_positions(text, "ultraplan")
}

pub fn find_ultrareview_trigger_positions(text: &str) -> Vec<TriggerPosition> {
    find_keyword_trigger_positions(text, "ultrareview")
}

pub fn has_ultraplan_keyword(text: &str) -> bool {
    !find_ultraplan_trigger_positions(text).is_empty()
}

pub fn has_ultrareview_keyword(text: &str) -> bool {
    !find_ultrareview_trigger_positions(text).is_empty()
}

pub fn replace_ultraplan_keyword(text: &str) -> String {
    let triggers = find_ultraplan_trigger_positions(text);
    if let Some(trigger) = triggers.first() {
        let before = &text[..trigger.start];
        let after = &text[trigger.end..];
        if before.trim().is_empty() || after.trim().is_empty() {
            return String::new();
        }
        let suffix = &trigger.word[5..];
        return format!("{}{}{}", before, suffix, after);
    }
    text.to_string()
}
