// Source: /data/home/swei/claudecode/openclaudecode/src/utils/glob.ts
#![allow(dead_code)]

pub fn glob_match(pattern: &str, text: &str) -> bool {
    let mut pattern_chars = pattern.chars().peekable();
    let mut text_chars = text.chars().peekable();

    while pattern_chars.peek().is_some() || text_chars.peek().is_some() {
        match (pattern_chars.peek(), text_chars.peek()) {
            (Some('*'), _) => {
                pattern_chars.next();
                if pattern_chars.peek().is_none() {
                    return true;
                }
                while text_chars.peek().is_some() {
                    if glob_match(
                        &pattern_chars.clone().collect::<String>(),
                        &text_chars.clone().collect::<String>(),
                    ) {
                        return true;
                    }
                    text_chars.next();
                }
                return false;
            }
            (Some('?'), Some(c)) => {
                pattern_chars.next();
                text_chars.next();
                if c != '.' {
                    return false;
                }
            }
            (Some(p), Some(t)) if p == t => {
                pattern_chars.next();
                text_chars.next();
            }
            (None, None) => return true,
            _ => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob() {
        assert!(glob_match("*.txt", "file.txt"));
        assert!(glob_match("test?", "test1"));
        assert!(!glob_match("*.js", "file.txt"));
    }
}
