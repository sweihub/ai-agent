// Source: /data/home/swei/claudecode/openclaudecode/src/utils/terminal.ts
const MAX_LINES_TO_SHOW: usize = 3;
const PADDING_TO_PREVENT_OVERFLOW: usize = 10;

fn char_width(c: char) -> usize {
    if c.is_ascii() {
        1
    } else {
        2
    }
}

fn string_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

pub fn wrap_text(text: &str, wrap_width: usize) -> (String, usize) {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut wrapped_lines: Vec<String> = Vec::new();

    for line in lines {
        let visible_width = string_width(line);
        if visible_width <= wrap_width {
            wrapped_lines.push(line.trim_end().to_string());
        } else {
            let mut position = 0;
            while position < visible_width {
                let chunk = slice_ansi(line, position, position + wrap_width);
                wrapped_lines.push(chunk.trim_end().to_string());
                position += wrap_width;
            }
        }
    }

    let remaining_lines = wrapped_lines.len().saturating_sub(MAX_LINES_TO_SHOW);

    if remaining_lines == 1 {
        let above = wrapped_lines[..=MAX_LINES_TO_SHOW].join("\n");
        return (above.trim_end().to_string(), 0);
    }

    let above = wrapped_lines[..MAX_LINES_TO_SHOW].join("\n");
    (above.trim_end().to_string(), remaining_lines)
}

fn slice_ansi(text: &str, start: usize, end: usize) -> String {
    let mut result = String::new();
    let mut current_pos = 0;
    let mut in_ansi = false;
    let mut ansi_buffer = String::new();

    for c in text.chars() {
        if c == '\u{1b}' {
            in_ansi = true;
            ansi_buffer.clear();
            ansi_buffer.push(c);
        } else if in_ansi {
            ansi_buffer.push(c);
            if c.is_ascii_alphabetic() {
                in_ansi = false;
                if current_pos >= start && current_pos < end {
                    result.push_str(&ansi_buffer);
                }
            }
        } else {
            if current_pos >= start && current_pos < end {
                if !result.is_empty() || c != '\n' {
                    result.push(c);
                }
            }
            current_pos += char_width(c);
        }
    }

    result
}

pub fn render_truncated_content(
    content: &str,
    terminal_width: usize,
    suppress_expand_hint: bool,
) -> String {
    let trimmed = content.trim_end();
    if trimmed.is_empty() {
        return String::new();
    }

    let wrap_width = terminal_width
        .saturating_sub(PADDING_TO_PREVENT_OVERFLOW)
        .max(10);
    let max_chars = MAX_LINES_TO_SHOW * wrap_width * 4;
    let pre_truncated = trimmed.len() > max_chars;

    let content_for_wrapping = if pre_truncated {
        &trimmed[..max_chars.min(trimmed.len())]
    } else {
        trimmed
    };

    let (above_the_fold, remaining_lines) = wrap_text(content_for_wrapping, wrap_width);

    let estimated_remaining = if pre_truncated {
        remaining_lines.max((trimmed.len() / wrap_width).saturating_sub(MAX_LINES_TO_SHOW))
    } else {
        remaining_lines
    };

    if estimated_remaining > 0 {
        let hint = if suppress_expand_hint {
            String::new()
        } else {
            format!(" (+{} more lines, ctrl+o to expand)", estimated_remaining)
        };
        format!("{}\n…{}", above_the_fold, hint)
    } else {
        above_the_fold
    }
}

pub fn is_output_line_truncated(content: &str) -> bool {
    let mut pos = 0;
    for _ in 0..=MAX_LINES_TO_SHOW {
        match content[pos..].find('\n') {
            Some(idx) => pos += idx + 1,
            None => return false,
        }
    }
    pos < content.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_short() {
        let (content, remaining) = wrap_text("hello", 80);
        assert_eq!(content, "hello");
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_truncated_content() {
        let result = render_truncated_content("line1\nline2\nline3\nline4", 80, false);
        assert!(!result.is_empty());
    }
}
