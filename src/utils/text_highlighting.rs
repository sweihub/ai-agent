use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextHighlight {
    pub start: usize,
    pub end: usize,
    pub color: Option<String>,
    #[serde(default)]
    pub dim_color: Option<bool>,
    #[serde(default)]
    pub inverse: Option<bool>,
    #[serde(default)]
    pub shimmer_color: Option<String>,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSegment {
    pub text: String,
    pub start: usize,
    #[serde(default)]
    pub highlight: Option<TextHighlight>,
}

pub fn segment_text_by_highlights(text: &str, highlights: Vec<TextHighlight>) -> Vec<TextSegment> {
    if highlights.is_empty() {
        return vec![TextSegment {
            text: text.to_string(),
            start: 0,
            highlight: None,
        }];
    }

    let mut sorted = highlights;
    sorted.sort_by(|a, b| {
        if a.start != b.start {
            a.start.cmp(&b.start)
        } else {
            b.priority.cmp(&a.priority)
        }
    });

    let mut resolved: Vec<TextHighlight> = Vec::new();
    let mut used_ranges: Vec<(usize, usize)> = Vec::new();

    for highlight in sorted {
        if highlight.start == highlight.end {
            continue;
        }

        let overlaps = used_ranges.iter().any(|(s, e)| {
            (highlight.start >= *s && highlight.start < *e)
                || (highlight.end > *s && highlight.end <= *e)
                || (highlight.start <= *s && highlight.end >= *e)
        });

        if !overlaps {
            resolved.push(highlight);
            used_ranges.push((highlight.start, highlight.end));
        }
    }

    segment_by_resolved(text, &resolved)
}

fn segment_by_resolved(text: &str, highlights: &[TextHighlight]) -> Vec<TextSegment> {
    let mut segments = Vec::new();
    let mut pos = 0;
    let text_chars: Vec<char> = text.chars().collect();

    for highlight in highlights {
        if highlight.start > pos {
            let text_part: String = text_chars[pos..highlight.start.min(text_chars.len())]
                .iter()
                .collect();
            segments.push(TextSegment {
                text: text_part,
                start: pos,
                highlight: None,
            });
        }

        let end = highlight.end.min(text_chars.len());
        if highlight.start < end {
            let highlighted: String = text_chars[highlight.start..end].iter().collect();
            segments.push(TextSegment {
                text: highlighted,
                start: highlight.start,
                highlight: Some(highlight.clone()),
            });
        }

        pos = highlight.end;
    }

    if pos < text_chars.len() {
        let remaining: String = text_chars[pos..].iter().collect();
        segments.push(TextSegment {
            text: remaining,
            start: pos,
            highlight: None,
        });
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_highlights() {
        let result = segment_text_by_highlights("hello", vec![]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, "hello");
    }

    #[test]
    fn test_with_highlight() {
        let highlights = vec![TextHighlight {
            start: 0,
            end: 3,
            color: Some("red".to_string()),
            priority: 1,
            ..Default::default()
        }];
        let result = segment_text_by_highlights("hello", highlights);
        assert_eq!(result.len(), 2);
    }
}
