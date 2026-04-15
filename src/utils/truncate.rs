// Source: /data/home/swei/claudecode/openclaudecode/src/utils/truncate.ts
const ELLIPSIS: &str = "…";

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

pub fn truncate_path_middle(path: &str, max_length: usize) -> String {
    if string_width(path) <= max_length {
        return path.to_string();
    }

    if max_length == 0 {
        return ELLIPSIS.to_string();
    }

    if max_length < 5 {
        return truncate_to_width(path, max_length);
    }

    if let Some(last_slash) = path.rfind('/') {
        let filename = &path[last_slash..];
        let directory = &path[..last_slash];
        let filename_width = string_width(filename);

        if filename_width >= max_length - 1 {
            return truncate_start_to_width(path, max_length);
        }

        let available_for_dir = max_length - 1 - filename_width;
        if available_for_dir <= 0 {
            return truncate_start_to_width(filename, max_length);
        }

        let truncated_dir = truncate_to_width_no_ellipsis(directory, available_for_dir);
        return format!("{}…{}", truncated_dir, filename);
    }

    truncate_start_to_width(path, max_length)
}

pub fn truncate_to_width(text: &str, max_width: usize) -> String {
    if string_width(text) <= max_width {
        return text.to_string();
    }
    if max_width <= 1 {
        return ELLIPSIS.to_string();
    }

    let mut width = 0;
    let mut result = String::new();

    for c in text.chars() {
        let cw = char_width(c);
        if width + cw > max_width - 1 {
            break;
        }
        result.push(c);
        width += cw;
    }

    format!("{}…", result)
}

pub fn truncate_start_to_width(text: &str, max_width: usize) -> String {
    if string_width(text) <= max_width {
        return text.to_string();
    }
    if max_width <= 1 {
        return ELLIPSIS.to_string();
    }

    let chars: Vec<char> = text.chars().collect();
    let mut width = 0;
    let mut start_idx = chars.len();

    for i in (0..chars.len()).rev() {
        let cw = char_width(chars[i]);
        if width + cw > max_width - 1 {
            break;
        }
        width += cw;
        start_idx = i;
    }

    format!(
        "{}{}",
        ELLIPSIS,
        chars[start_idx..].iter().collect::<String>()
    )
}

pub fn truncate_to_width_no_ellipsis(text: &str, max_width: usize) -> String {
    if string_width(text) <= max_width {
        return text.to_string();
    }
    if max_width == 0 {
        return String::new();
    }

    let mut width = 0;
    let mut result = String::new();

    for c in text.chars() {
        let cw = char_width(c);
        if width + cw > max_width {
            break;
        }
        result.push(c);
        width += cw;
    }

    result
}

pub fn truncate(text: &str, max_width: usize, single_line: bool) -> String {
    let mut result = text.to_string();

    if single_line {
        if let Some(first_newline) = result.find('\n') {
            let truncated = result[..first_newline].to_string();
            if string_width(&truncated) + 1 > max_width {
                return truncate_to_width(&truncated, max_width);
            }
            return format!("{}…", truncated);
        }
    }

    if string_width(&result) <= max_width {
        return result;
    }

    truncate_to_width(&result, max_width)
}

pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for c in text.chars() {
        let cw = char_width(c);
        if current_width + cw <= width {
            current_line.push(c);
            current_width += cw;
        } else {
            if !current_line.is_empty() {
                lines.push(current_line.clone());
                current_line.clear();
            }
            current_line.push(c);
            current_width = cw;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short() {
        assert_eq!(truncate("hello", 10, false), "hello");
    }

    #[test]
    fn test_truncate_long() {
        assert_eq!(truncate("hello world", 5, false), "hello…");
    }
}
