// Source: /data/home/swei/claudecode/openclaudecode/src/ink/termio/ansi.ts
#![allow(dead_code)]

pub fn strip_ansi(s: &str) -> String {
    s.chars().filter(|c| *c != '\x1b').collect()
}

pub fn strip_ansi_with_len(s: &str) -> (String, usize) {
    let mut result = String::new();
    let mut len = 0;
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            result.push(c);
            len += 1;
        }
    }

    (result, len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi() {
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
    }
}
