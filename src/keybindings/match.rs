// Source: /data/home/swei/claudecode/openclaudecode/src/keybindings/match.ts
//! Keybinding match utilities

pub fn match_keybinding(key: &str, pattern: &str) -> bool {
    let key_parts: Vec<&str> = key.to_lowercase().split('+').collect();
    let pattern_parts: Vec<&str> = pattern.to_lowercase().split('+').collect();

    if key_parts.len() != pattern_parts.len() {
        return false;
    }

    for (i, part) in pattern_parts.iter().enumerate() {
        match *part {
            "ctrl" | "control" => {
                if !key_parts.contains(&"ctrl") && !key_parts.contains(&"control") {
                    return false;
                }
            }
            "alt" => {
                if !key_parts.contains(&"alt") {
                    return false;
                }
            }
            "shift" => {
                if !key_parts.contains(&"shift") {
                    return false;
                }
            }
            "meta" | "cmd" => {
                if !key_parts.contains(&"meta") && !key_parts.contains(&"cmd") {
                    return false;
                }
            }
            _ => {
                if key_parts.get(i) != Some(part) {
                    return false;
                }
            }
        }
    }

    true
}
