// Source: ~/claudecode/openclaudecode/src/tools/BashTool/commentLabel.ts
/// If the first line of a bash command is a `# comment` (not a `#!` shebang),
/// return the comment text stripped of the `#` prefix. Otherwise `None`.
///
/// Under fullscreen mode this is the non-verbose tool-use label AND the
/// collapse-group hint -- it's what the agent wrote for the human to read.
pub fn extract_bash_comment_label(command: &str) -> Option<String> {
    let nl = command.find('\n');
    let first_line = match nl {
        None => command.trim(),
        Some(pos) => command[..pos].trim(),
    };

    if !first_line.starts_with('#') || first_line.starts_with("#!") {
        return None;
    }

    let stripped = first_line.trim_start_matches('#').trim_start();
    if stripped.is_empty() {
        None
    } else {
        Some(stripped.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_comment() {
        assert_eq!(
            extract_bash_comment_label("# hello world"),
            Some("hello world".to_string())
        );
    }

    #[test]
    fn test_shebang_returns_none() {
        assert_eq!(extract_bash_comment_label("#!/bin/bash"), None);
    }

    #[test]
    fn test_multiline_command() {
        assert_eq!(
            extract_bash_comment_label("# run tests\ncargo test"),
            Some("run tests".to_string())
        );
    }

    #[test]
    fn test_non_comment_returns_none() {
        assert_eq!(extract_bash_comment_label("cargo test"), None);
    }

    #[test]
    fn test_multiple_hashes() {
        assert_eq!(
            extract_bash_comment_label("### important note"),
            Some("important note".to_string())
        );
    }
}
