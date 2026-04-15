//! Voice keyterms - STT keyword hints for voice mode.
//!
////! Translates voiceKeyterms.ts from claude code.

const MAX_KEYTERMS: usize = 50;

const GLOBAL_KEYTERMS: &[&str] = &[
    "MCP",
    "symlink",
    "grep",
    "regex",
    "localhost",
    "codebase",
    "TypeScript",
    "JSON",
    "OAuth",
    "webhook",
    "gRPC",
    "dotfiles",
    "subagent",
    "worktree",
];

pub fn split_identifier(name: &str) -> Vec<String> {
    let result: String = name
        .chars()
        .map(|c| {
            if c.is_lowercase() && c.is_alphabetic() {
                c.to_string()
            } else if c.is_uppercase() && c.is_alphabetic() {
                format!(" {}", c)
            } else {
                c.to_string()
            }
        })
        .collect();

    result
        .split(|c: char| c == '-' || c == '_' || c == '/' || c == '.' || c == ' ')
        .map(|s| s.trim().to_string())
        .filter(|s| s.len() > 2 && s.len() <= 20)
        .collect()
}

fn file_name_words(file_path: &str) -> Vec<String> {
    let stem = std::path::Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    split_identifier(&stem)
}

pub fn get_voice_keyterms(recent_files: Option<&[String]>) -> Vec<String> {
    let mut terms: std::collections::HashSet<String> =
        GLOBAL_KEYTERMS.iter().map(|s| s.to_string()).collect();

    if let Some(files) = recent_files {
        for file_path in files.iter() {
            if terms.len() >= MAX_KEYTERMS {
                break;
            }
            for word in file_name_words(file_path) {
                terms.insert(word);
            }
        }
    }

    let mut result: Vec<String> = terms.into_iter().collect();
    result.truncate(MAX_KEYTERMS);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_identifier_camel_case() {
        let words = split_identifier("camelCase");
        assert!(words.len() > 0);
    }

    #[test]
    fn test_split_identifier_snake_case() {
        let words = split_identifier("snake_case");
        assert!(words.contains(&"snake".to_string()) || words.contains(&"snake_case".to_string()));
    }

    #[test]
    fn test_split_identifier_kebab_case() {
        let words = split_identifier("kebab-case");
        assert!(words.len() > 0);
    }

    #[test]
    fn test_file_name_words() {
        let words = file_name_words("/path/to/MyFile.ts");
        assert!(!words.is_empty());
    }

    #[test]
    fn test_get_voice_keyterms_empty() {
        let terms = get_voice_keyterms(None);
        assert!(!terms.is_empty());
        assert!(terms.len() <= MAX_KEYTERMS);
    }

    #[test]
    fn test_get_voice_keyterms_with_files() {
        let files = vec![
            "/path/to/main.rs".to_string(),
            "/path/to/utils.rs".to_string(),
        ];
        let terms = get_voice_keyterms(Some(&files));
        assert!(terms.len() <= MAX_KEYTERMS);
    }
}
