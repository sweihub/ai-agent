// Source: ~/claudecode/openclaudecode/src/utils/ink.ts

/// Default agent theme color for unknown colors.
const DEFAULT_AGENT_THEME_COLOR: &str = "cyan_FOR_SUBAGENTS_ONLY";

/// Convert a color string to Ink color format.
/// Colors are typically AgentColorName values like 'blue', 'green', etc.
/// This converts them to theme keys so they respect the current theme.
/// Falls back to the raw ANSI color if the color is not a known agent color.
pub fn to_ink_color(color: Option<&str>) -> String {
    let Some(color) = color else {
        return DEFAULT_AGENT_THEME_COLOR.to_string();
    };

    // Try to map to a theme color if it's a known agent color
    if let Some(theme_color) = agent_color_to_theme_color(color) {
        return theme_color.to_string();
    }

    // Fall back to raw ANSI color for unknown colors
    format!("ansi:{color}")
}

/// Map agent color names to theme colors.
fn agent_color_to_theme_color(color: &str) -> Option<&'static str> {
    match color {
        "blue" => Some("blue"),
        "green" => Some("green"),
        "red" => Some("red"),
        "yellow" => Some("yellow"),
        "cyan" => Some("cyan"),
        "magenta" => Some("magenta"),
        "white" => Some("white"),
        "gray" => Some("gray"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_ink_color_none() {
        assert_eq!(to_ink_color(None), DEFAULT_AGENT_THEME_COLOR);
    }

    #[test]
    fn test_to_ink_color_known() {
        assert_eq!(to_ink_color(Some("blue")), "blue");
    }

    #[test]
    fn test_to_ink_color_unknown() {
        assert_eq!(to_ink_color(Some("orange")), "ansi:orange");
    }
}
