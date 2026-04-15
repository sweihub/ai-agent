// Source: /data/home/swei/claudecode/openclaudecode/src/constants/figures.ts
use std::env;

/// Returns the appropriate black circle character for the current OS.
/// The former is better vertically aligned, but isn't usually supported on Windows/Linux
pub fn black_circle() -> &'static str {
    if env::consts::OS == "macos" {
        "⏺"
    } else {
        "●"
    }
}

pub const BULLET_OPERATOR: &str = "∙";
pub const TEARDROP_ASTERISK: &str = "✻";
pub const UP_ARROW: &str = "\u{2191}"; // ↑ - used for opus 1m merge notice
pub const DOWN_ARROW: &str = "\u{2193}"; // ↓ - used for scroll hint
pub const LIGHTNING_BOLT: &str = "\u{21af}"; // used for fast mode indicator
pub const EFFORT_LOW: &str = "\u{25cb}"; // effort level: low
pub const EFFORT_MEDIUM: &str = "\u{25d0}"; // effort level: medium
pub const EFFORT_HIGH: &str = "\u{25cf}"; // effort level: high
pub const EFFORT_MAX: &str = "\u{25c9}"; // effort level: max (Opus 4.6 only)

// Media/trigger status indicators
pub const PLAY_ICON: &str = "\u{25b6}"; // ▶
pub const PAUSE_ICON: &str = "\u{23f8}"; // ⏸

// MCP subscription indicators
pub const REFRESH_ARROW: &str = "\u{21bb}"; // ↻ - used for resource update indicator
pub const CHANNEL_ARROW: &str = "\u{2190}"; // ← - inbound channel message indicator
pub const INJECTED_ARROW: &str = "\u{2192}"; // → - cross-session injected message indicator
pub const FORK_GLYPH: &str = "\u{2442}"; // ⑂ - fork directive indicator

// Review status indicators (ultrareview diamond states)
pub const DIAMOND_OPEN: &str = "\u{25c7}"; // ◇ - running
pub const DIAMOND_FILLED: &str = "\u{25c6}"; // ◆ - completed/failed
pub const REFERENCE_MARK: &str = "\u{203b}"; // ※ - komejirushi, away-summary recap marker

// Issue flag indicator
pub const FLAG_ICON: &str = "\u{2691}"; // ⚑ - used for issue flag banner

// Blockquote indicator
pub const BLOCKQUOTE_BAR: &str = "\u{258e}"; // ▎ - left one-quarter block, used as blockquote line prefix
pub const HEAVY_HORIZONTAL: &str = "\u{2501}"; // ━ - heavy box-drawing horizontal

// Bridge status indicators
pub const BRIDGE_SPINNER_FRAMES: &[&str] = &[
    "\u{00b7}|\u{00b7}",
    "\u{00b7}/\u{00b7}",
    "\u{00b7}\u{2014}\u{00b7}",
    "\u{00b7}\\\u{00b7}",
];
pub const BRIDGE_READY_INDICATOR: &str = "\u{00b7}\u{2714}\u{fe0e}\u{00b7}";
pub const BRIDGE_FAILED_INDICATOR: &str = "\u{00d7}";
