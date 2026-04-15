// Source: /data/home/swei/claudecode/openclaudecode/src/keybindings/parser.ts
//! Keybindings parser

#[derive(Debug, Clone)]
pub struct ParsedKeystroke {
    pub key: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

pub fn parse_keystroke(keystroke: &str) -> ParsedKeystroke {
    let parts: Vec<&str> = keystroke.to_lowercase().split('+').collect();

    let mut parsed = ParsedKeystroke {
        key: String::new(),
        ctrl: false,
        alt: false,
        shift: false,
        meta: false,
    };

    for part in parts {
        let part = part.trim();
        match part {
            "ctrl" | "control" => parsed.ctrl = true,
            "alt" => parsed.alt = true,
            "shift" => parsed.shift = true,
            "meta" | "cmd" | "command" => parsed.meta = true,
            _ => parsed.key = part.to_string(),
        }
    }

    parsed
}

pub fn chord_to_string(chord: &[ParsedKeystroke]) -> String {
    chord
        .iter()
        .map(|k| {
            let mut parts = Vec::new();
            if k.ctrl {
                parts.push("ctrl");
            }
            if k.alt {
                parts.push("alt");
            }
            if k.shift {
                parts.push("shift");
            }
            if k.meta {
                parts.push("meta");
            }
            parts.push(&k.key);
            parts.join("+")
        })
        .collect::<Vec<_>>()
        .join(" ")
}
