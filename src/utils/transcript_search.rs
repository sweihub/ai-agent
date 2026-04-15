use std::collections::HashMap;

pub fn search_transcript(
    transcript: &str,
    query: &str,
    case_sensitive: bool,
) -> Vec<TranscriptMatch> {
    let search_text = if case_sensitive {
        query.to_string()
    } else {
        query.to_lowercase()
    };

    let text = if case_sensitive {
        transcript.to_string()
    } else {
        transcript.to_lowercase()
    };

    let mut matches = Vec::new();
    let mut start = 0;

    while let Some(pos) = text[start..].find(&search_text) {
        let absolute_pos = start + pos;
        matches.push(TranscriptMatch {
            position: absolute_pos,
            line_number: transcript[..absolute_pos].matches('\n').count() + 1,
            context: extract_context(transcript, absolute_pos, query.len()),
        });
        start = absolute_pos + 1;
    }

    matches
}

fn extract_context(text: &str, position: usize, match_len: usize) -> String {
    let context_before = 50;
    let context_after = 50;

    let start = text[..position]
        .char_indices()
        .rev()
        .take(context_before)
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0);

    let end = match text[position + match_len..]
        .char_indices()
        .take(context_after)
        .last()
    {
        Some((i, _)) => position + match_len + i,
        None => text.len(),
    };

    format!("...{}...", &text[start..end])
}

#[derive(Debug, Clone)]
pub struct TranscriptMatch {
    pub position: usize,
    pub line_number: usize,
    pub context: String,
}

pub fn highlight_matches(text: &str, query: &str) -> String {
    text.replace(query, &format!("**{}**", query))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_transcript() {
        let transcript = "line1\nline2\nsearch term\nline4";
        let matches = search_transcript(transcript, "search", false);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].line_number, 3);
    }
}
