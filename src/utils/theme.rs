// Source: /data/home/swei/claudecode/openclaudecode/src/commands/theme/theme.tsx
/**
 * Theme definitions for the TUI
 */
use once_cell::sync::Lazy;

/// Theme color type - can be RGB or ANSI
#[derive(Debug, Clone, PartialEq)]
pub enum ThemeColor {
    Rgb(u8, u8, u8),
    Ansi(AnsiColor),
}

impl ThemeColor {
    pub fn to_ansi_string(&self) -> String {
        match self {
            ThemeColor::Ansi(color) => color.to_string(),
            ThemeColor::Rgb(r, g, b) => format!("rgb({},{},{})", r, g, b),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnsiColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BlackBright,
    RedBright,
    GreenBright,
    YellowBright,
    BlueBright,
    MagentaBright,
    CyanBright,
    WhiteBright,
}

impl AnsiColor {
    fn to_string(&self) -> String {
        match self {
            AnsiColor::Black => "ansi:black".to_string(),
            AnsiColor::Red => "ansi:red".to_string(),
            AnsiColor::Green => "ansi:green".to_string(),
            AnsiColor::Yellow => "ansi:yellow".to_string(),
            AnsiColor::Blue => "ansi:blue".to_string(),
            AnsiColor::Magenta => "ansi:magenta".to_string(),
            AnsiColor::Cyan => "ansi:cyan".to_string(),
            AnsiColor::White => "ansi:white".to_string(),
            AnsiColor::BlackBright => "ansi:blackBright".to_string(),
            AnsiColor::RedBright => "ansi:redBright".to_string(),
            AnsiColor::GreenBright => "ansi:greenBright".to_string(),
            AnsiColor::YellowBright => "ansi:yellowBright".to_string(),
            AnsiColor::BlueBright => "ansi:blueBright".to_string(),
            AnsiColor::MagentaBright => "ansi:magentaBright".to_string(),
            AnsiColor::CyanBright => "ansi:cyanBright".to_string(),
            AnsiColor::WhiteBright => "ansi:whiteBright".to_string(),
        }
    }
}

/// Theme struct containing all color definitions
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub auto_accept: ThemeColor,
    pub bash_border: ThemeColor,
    pub claude: ThemeColor,
    pub claude_shimmer: ThemeColor,
    pub claude_blue_for_system_spinner: ThemeColor,
    pub claude_blue_shimmer_for_system_spinner: ThemeColor,
    pub permission: ThemeColor,
    pub permission_shimmer: ThemeColor,
    pub plan_mode: ThemeColor,
    pub ide: ThemeColor,
    pub prompt_border: ThemeColor,
    pub prompt_border_shimmer: ThemeColor,
    pub text: ThemeColor,
    pub inverse_text: ThemeColor,
    pub inactive: ThemeColor,
    pub inactive_shimmer: ThemeColor,
    pub subtle: ThemeColor,
    pub suggestion: ThemeColor,
    pub remember: ThemeColor,
    pub background: ThemeColor,
    // Semantic colors
    pub success: ThemeColor,
    pub error: ThemeColor,
    pub warning: ThemeColor,
    pub merged: ThemeColor,
    pub warning_shimmer: ThemeColor,
    // Diff colors
    pub diff_added: ThemeColor,
    pub diff_removed: ThemeColor,
    pub diff_added_dimmed: ThemeColor,
    pub diff_removed_dimmed: ThemeColor,
    // Word-level diff highlighting
    pub diff_added_word: ThemeColor,
    pub diff_removed_word: ThemeColor,
    // Agent colors
    pub red_for_subagents_only: ThemeColor,
    pub blue_for_subagents_only: ThemeColor,
    pub green_for_subagents_only: ThemeColor,
    pub yellow_for_subagents_only: ThemeColor,
    pub purple_for_subagents_only: ThemeColor,
    pub orange_for_subagents_only: ThemeColor,
    pub pink_for_subagents_only: ThemeColor,
    pub cyan_for_subagents_only: ThemeColor,
    // Grove colors
    pub professional_blue: ThemeColor,
    // Chrome colors
    pub chrome_yellow: ThemeColor,
    // TUI V2 colors
    pub clawd_body: ThemeColor,
    pub clawd_background: ThemeColor,
    pub user_message_background: ThemeColor,
    pub user_message_background_hover: ThemeColor,
    pub message_actions_background: ThemeColor,
    pub selection_bg: ThemeColor,
    pub bash_message_background_color: ThemeColor,
    pub memory_background_color: ThemeColor,
    pub rate_limit_fill: ThemeColor,
    pub rate_limit_empty: ThemeColor,
    pub fast_mode: ThemeColor,
    pub fast_mode_shimmer: ThemeColor,
    // Brief/assistant mode label colors
    pub brief_label_you: ThemeColor,
    pub brief_label_claude: ThemeColor,
    // Rainbow colors for ultrathink keyword highlighting
    pub rainbow_red: ThemeColor,
    pub rainbow_orange: ThemeColor,
    pub rainbow_yellow: ThemeColor,
    pub rainbow_green: ThemeColor,
    pub rainbow_blue: ThemeColor,
    pub rainbow_indigo: ThemeColor,
    pub rainbow_violet: ThemeColor,
    pub rainbow_red_shimmer: ThemeColor,
    pub rainbow_orange_shimmer: ThemeColor,
    pub rainbow_yellow_shimmer: ThemeColor,
    pub rainbow_green_shimmer: ThemeColor,
    pub rainbow_blue_shimmer: ThemeColor,
    pub rainbow_indigo_shimmer: ThemeColor,
    pub rainbow_violet_shimmer: ThemeColor,
}

/// Helper to create RGB color
fn rgb(r: u8, g: u8, b: u8) -> ThemeColor {
    ThemeColor::Rgb(r, g, b)
}

/// Helper to create ANSI color
fn ansi(color: AnsiColor) -> ThemeColor {
    ThemeColor::Ansi(color)
}

/// Light theme using explicit RGB values to avoid inconsistencies
/// from users' custom terminal ANSI color definitions
pub static LIGHT_THEME: Lazy<Theme> = Lazy::new(|| Theme {
    auto_accept: rgb(135, 0, 255),                     // Electric violet
    bash_border: rgb(255, 0, 135),                     // Vibrant pink
    claude: rgb(215, 119, 87),                         // Claude orange
    claude_shimmer: rgb(245, 149, 117),                // Lighter claude orange for shimmer effect
    claude_blue_for_system_spinner: rgb(87, 105, 247), // Medium blue for system spinner
    claude_blue_shimmer_for_system_spinner: rgb(117, 135, 255), // Lighter blue for system spinner shimmer
    permission: rgb(87, 105, 247),                              // Medium blue
    permission_shimmer: rgb(137, 155, 255),                     // Lighter blue for shimmer effect
    plan_mode: rgb(0, 102, 102),                                // Muted teal
    ide: rgb(71, 130, 200),                                     // Muted blue
    prompt_border: rgb(153, 153, 153),                          // Medium gray
    prompt_border_shimmer: rgb(183, 183, 183),                  // Lighter gray for shimmer effect
    text: rgb(0, 0, 0),                                         // Black
    inverse_text: rgb(255, 255, 255),                           // White
    inactive: rgb(102, 102, 102),                               // Dark gray
    inactive_shimmer: rgb(142, 142, 142),                       // Lighter gray for shimmer effect
    subtle: rgb(175, 175, 175),                                 // Light gray
    suggestion: rgb(87, 105, 247),                              // Medium blue
    remember: rgb(0, 0, 255),                                   // Blue
    background: rgb(0, 153, 153),                               // Cyan
    success: rgb(44, 122, 57),                                  // Green
    error: rgb(171, 43, 63),                                    // Red
    warning: rgb(150, 108, 30),                                 // Amber
    merged: rgb(135, 0, 255), // Electric violet (matches autoAccept)
    warning_shimmer: rgb(200, 158, 80), // Lighter amber for shimmer effect
    diff_added: rgb(105, 219, 124), // Light green
    diff_removed: rgb(255, 168, 180), // Light red
    diff_added_dimmed: rgb(199, 225, 203), // Very light green
    diff_removed_dimmed: rgb(253, 210, 216), // Very light red
    diff_added_word: rgb(47, 157, 68), // Medium green
    diff_removed_word: rgb(209, 69, 75), // Medium red
    // Agent colors
    red_for_subagents_only: rgb(220, 38, 38),     // Red 600
    blue_for_subagents_only: rgb(37, 99, 235),    // Blue 600
    green_for_subagents_only: rgb(22, 163, 74),   // Green 600
    yellow_for_subagents_only: rgb(202, 138, 4),  // Yellow 600
    purple_for_subagents_only: rgb(147, 51, 234), // Purple 600
    orange_for_subagents_only: rgb(234, 88, 12),  // Orange 600
    pink_for_subagents_only: rgb(219, 39, 119),   // Pink 600
    cyan_for_subagents_only: rgb(8, 145, 178),    // Cyan 600
    // Grove colors
    professional_blue: rgb(106, 155, 204),
    // Chrome colors
    chrome_yellow: rgb(251, 188, 4), // Chrome yellow
    // TUI V2 colors
    clawd_body: rgb(215, 119, 87),
    clawd_background: rgb(0, 0, 0),
    user_message_background: rgb(240, 240, 240), // Slightly darker grey for optimal contrast
    user_message_background_hover: rgb(252, 252, 252), // >=250 to quantize distinct from base at 256-color level
    message_actions_background: rgb(232, 236, 244), // cool gray -- darker than userMsg 240 (visible on white), slight blue toward `suggestion`
    selection_bg: rgb(180, 213, 255), // classic light-mode selection blue (macOS/VS Code-ish); dark fgs stay readable
    bash_message_background_color: rgb(250, 245, 250),
    memory_background_color: rgb(230, 245, 250),
    rate_limit_fill: rgb(87, 105, 247),   // Medium blue
    rate_limit_empty: rgb(39, 47, 111),   // Dark blue
    fast_mode: rgb(255, 106, 0),          // Electric orange
    fast_mode_shimmer: rgb(255, 150, 50), // Lighter orange for shimmer
    // Brief/assistant mode
    brief_label_you: rgb(37, 99, 235),     // Blue
    brief_label_claude: rgb(215, 119, 87), // Brand orange
    rainbow_red: rgb(235, 95, 87),
    rainbow_orange: rgb(245, 139, 87),
    rainbow_yellow: rgb(250, 195, 95),
    rainbow_green: rgb(145, 200, 130),
    rainbow_blue: rgb(130, 170, 220),
    rainbow_indigo: rgb(155, 130, 200),
    rainbow_violet: rgb(200, 130, 180),
    rainbow_red_shimmer: rgb(250, 155, 147),
    rainbow_orange_shimmer: rgb(255, 185, 137),
    rainbow_yellow_shimmer: rgb(255, 225, 155),
    rainbow_green_shimmer: rgb(185, 230, 180),
    rainbow_blue_shimmer: rgb(180, 205, 240),
    rainbow_indigo_shimmer: rgb(195, 180, 230),
    rainbow_violet_shimmer: rgb(230, 180, 210),
});

/// Light ANSI theme using only the 16 standard ANSI colors
/// for terminals without true color support
pub static LIGHT_ANSI_THEME: Lazy<Theme> = Lazy::new(|| Theme {
    auto_accept: ansi(AnsiColor::Magenta),
    bash_border: ansi(AnsiColor::Magenta),
    claude: ansi(AnsiColor::RedBright),
    claude_shimmer: ansi(AnsiColor::YellowBright),
    claude_blue_for_system_spinner: ansi(AnsiColor::Blue),
    claude_blue_shimmer_for_system_spinner: ansi(AnsiColor::BlueBright),
    permission: ansi(AnsiColor::Blue),
    permission_shimmer: ansi(AnsiColor::BlueBright),
    plan_mode: ansi(AnsiColor::Cyan),
    ide: ansi(AnsiColor::BlueBright),
    prompt_border: ansi(AnsiColor::White),
    prompt_border_shimmer: ansi(AnsiColor::WhiteBright),
    text: ansi(AnsiColor::Black),
    inverse_text: ansi(AnsiColor::White),
    inactive: ansi(AnsiColor::BlackBright),
    inactive_shimmer: ansi(AnsiColor::White),
    subtle: ansi(AnsiColor::BlackBright),
    suggestion: ansi(AnsiColor::Blue),
    remember: ansi(AnsiColor::Blue),
    background: ansi(AnsiColor::Cyan),
    success: ansi(AnsiColor::Green),
    error: ansi(AnsiColor::Red),
    warning: ansi(AnsiColor::Yellow),
    merged: ansi(AnsiColor::Magenta),
    warning_shimmer: ansi(AnsiColor::YellowBright),
    diff_added: ansi(AnsiColor::Green),
    diff_removed: ansi(AnsiColor::Red),
    diff_added_dimmed: ansi(AnsiColor::Green),
    diff_removed_dimmed: ansi(AnsiColor::Red),
    diff_added_word: ansi(AnsiColor::GreenBright),
    diff_removed_word: ansi(AnsiColor::RedBright),
    // Agent colors
    red_for_subagents_only: ansi(AnsiColor::Red),
    blue_for_subagents_only: ansi(AnsiColor::Blue),
    green_for_subagents_only: ansi(AnsiColor::Green),
    yellow_for_subagents_only: ansi(AnsiColor::Yellow),
    purple_for_subagents_only: ansi(AnsiColor::Magenta),
    orange_for_subagents_only: ansi(AnsiColor::RedBright),
    pink_for_subagents_only: ansi(AnsiColor::MagentaBright),
    cyan_for_subagents_only: ansi(AnsiColor::Cyan),
    // Grove colors
    professional_blue: ansi(AnsiColor::BlueBright),
    // Chrome colors
    chrome_yellow: ansi(AnsiColor::Yellow),
    // TUI V2 colors
    clawd_body: ansi(AnsiColor::RedBright),
    clawd_background: ansi(AnsiColor::Black),
    user_message_background: ansi(AnsiColor::White),
    user_message_background_hover: ansi(AnsiColor::WhiteBright),
    message_actions_background: ansi(AnsiColor::White),
    selection_bg: ansi(AnsiColor::Cyan),
    bash_message_background_color: ansi(AnsiColor::WhiteBright),
    memory_background_color: ansi(AnsiColor::White),
    rate_limit_fill: ansi(AnsiColor::Yellow),
    rate_limit_empty: ansi(AnsiColor::Black),
    fast_mode: ansi(AnsiColor::Red),
    fast_mode_shimmer: ansi(AnsiColor::RedBright),
    brief_label_you: ansi(AnsiColor::Blue),
    brief_label_claude: ansi(AnsiColor::RedBright),
    rainbow_red: ansi(AnsiColor::Red),
    rainbow_orange: ansi(AnsiColor::RedBright),
    rainbow_yellow: ansi(AnsiColor::Yellow),
    rainbow_green: ansi(AnsiColor::Green),
    rainbow_blue: ansi(AnsiColor::Cyan),
    rainbow_indigo: ansi(AnsiColor::Blue),
    rainbow_violet: ansi(AnsiColor::Magenta),
    rainbow_red_shimmer: ansi(AnsiColor::RedBright),
    rainbow_orange_shimmer: ansi(AnsiColor::Yellow),
    rainbow_yellow_shimmer: ansi(AnsiColor::YellowBright),
    rainbow_green_shimmer: ansi(AnsiColor::GreenBright),
    rainbow_blue_shimmer: ansi(AnsiColor::CyanBright),
    rainbow_indigo_shimmer: ansi(AnsiColor::BlueBright),
    rainbow_violet_shimmer: ansi(AnsiColor::MagentaBright),
});

/// Dark ANSI theme using only the 16 standard ANSI colors
/// for terminals without true color support
pub static DARK_ANSI_THEME: Lazy<Theme> = Lazy::new(|| Theme {
    auto_accept: ansi(AnsiColor::MagentaBright),
    bash_border: ansi(AnsiColor::MagentaBright),
    claude: ansi(AnsiColor::RedBright),
    claude_shimmer: ansi(AnsiColor::YellowBright),
    claude_blue_for_system_spinner: ansi(AnsiColor::BlueBright),
    claude_blue_shimmer_for_system_spinner: ansi(AnsiColor::BlueBright),
    permission: ansi(AnsiColor::BlueBright),
    permission_shimmer: ansi(AnsiColor::BlueBright),
    plan_mode: ansi(AnsiColor::CyanBright),
    ide: ansi(AnsiColor::Blue),
    prompt_border: ansi(AnsiColor::White),
    prompt_border_shimmer: ansi(AnsiColor::WhiteBright),
    text: ansi(AnsiColor::WhiteBright),
    inverse_text: ansi(AnsiColor::Black),
    inactive: ansi(AnsiColor::White),
    inactive_shimmer: ansi(AnsiColor::WhiteBright),
    subtle: ansi(AnsiColor::White),
    suggestion: ansi(AnsiColor::BlueBright),
    remember: ansi(AnsiColor::BlueBright),
    background: ansi(AnsiColor::CyanBright),
    success: ansi(AnsiColor::GreenBright),
    error: ansi(AnsiColor::RedBright),
    warning: ansi(AnsiColor::YellowBright),
    merged: ansi(AnsiColor::MagentaBright),
    warning_shimmer: ansi(AnsiColor::YellowBright),
    diff_added: ansi(AnsiColor::Green),
    diff_removed: ansi(AnsiColor::Red),
    diff_added_dimmed: ansi(AnsiColor::Green),
    diff_removed_dimmed: ansi(AnsiColor::Red),
    diff_added_word: ansi(AnsiColor::GreenBright),
    diff_removed_word: ansi(AnsiColor::RedBright),
    // Agent colors
    red_for_subagents_only: ansi(AnsiColor::RedBright),
    blue_for_subagents_only: ansi(AnsiColor::BlueBright),
    green_for_subagents_only: ansi(AnsiColor::GreenBright),
    yellow_for_subagents_only: ansi(AnsiColor::YellowBright),
    purple_for_subagents_only: ansi(AnsiColor::MagentaBright),
    orange_for_subagents_only: ansi(AnsiColor::RedBright),
    pink_for_subagents_only: ansi(AnsiColor::MagentaBright),
    cyan_for_subagents_only: ansi(AnsiColor::CyanBright),
    // Grove colors
    professional_blue: rgb(106, 155, 204),
    // Chrome colors
    chrome_yellow: ansi(AnsiColor::YellowBright),
    // TUI V2 colors
    clawd_body: ansi(AnsiColor::RedBright),
    clawd_background: ansi(AnsiColor::Black),
    user_message_background: ansi(AnsiColor::BlackBright),
    user_message_background_hover: ansi(AnsiColor::White),
    message_actions_background: ansi(AnsiColor::BlackBright),
    selection_bg: ansi(AnsiColor::Blue),
    bash_message_background_color: ansi(AnsiColor::Black),
    memory_background_color: ansi(AnsiColor::BlackBright),
    rate_limit_fill: ansi(AnsiColor::Yellow),
    rate_limit_empty: ansi(AnsiColor::White),
    fast_mode: ansi(AnsiColor::RedBright),
    fast_mode_shimmer: ansi(AnsiColor::RedBright),
    brief_label_you: ansi(AnsiColor::BlueBright),
    brief_label_claude: ansi(AnsiColor::RedBright),
    rainbow_red: ansi(AnsiColor::Red),
    rainbow_orange: ansi(AnsiColor::RedBright),
    rainbow_yellow: ansi(AnsiColor::Yellow),
    rainbow_green: ansi(AnsiColor::Green),
    rainbow_blue: ansi(AnsiColor::Cyan),
    rainbow_indigo: ansi(AnsiColor::Blue),
    rainbow_violet: ansi(AnsiColor::Magenta),
    rainbow_red_shimmer: ansi(AnsiColor::RedBright),
    rainbow_orange_shimmer: ansi(AnsiColor::Yellow),
    rainbow_yellow_shimmer: ansi(AnsiColor::YellowBright),
    rainbow_green_shimmer: ansi(AnsiColor::GreenBright),
    rainbow_blue_shimmer: ansi(AnsiColor::CyanBright),
    rainbow_indigo_shimmer: ansi(AnsiColor::BlueBright),
    rainbow_violet_shimmer: ansi(AnsiColor::MagentaBright),
});

/// Light daltonized theme (color-blind friendly) using explicit RGB values
/// to avoid inconsistencies from users' custom terminal ANSI color definitions
pub static LIGHT_DALTONIZED_THEME: Lazy<Theme> = Lazy::new(|| Theme {
    auto_accept: rgb(135, 0, 255),                     // Electric violet
    bash_border: rgb(0, 102, 204),                     // Blue instead of pink
    claude: rgb(255, 153, 51),                         // Orange adjusted for deuteranopia
    claude_shimmer: rgb(255, 183, 101),                // Lighter orange for shimmer effect
    claude_blue_for_system_spinner: rgb(51, 102, 255), // Bright blue for system spinner
    claude_blue_shimmer_for_system_spinner: rgb(101, 152, 255), // Lighter bright blue for system spinner shimmer
    permission: rgb(51, 102, 255),                              // Bright blue
    permission_shimmer: rgb(101, 152, 255),                     // Lighter bright blue for shimmer
    plan_mode: rgb(51, 102, 102), // Muted blue-gray (works for color-blind)
    ide: rgb(71, 130, 200),       // Muted blue
    prompt_border: rgb(153, 153, 153), // Medium gray
    prompt_border_shimmer: rgb(183, 183, 183), // Lighter gray for shimmer
    text: rgb(0, 0, 0),           // Black
    inverse_text: rgb(255, 255, 255), // White
    inactive: rgb(102, 102, 102), // Dark gray
    inactive_shimmer: rgb(142, 142, 142), // Lighter gray for shimmer effect
    subtle: rgb(175, 175, 175),   // Light gray
    suggestion: rgb(51, 102, 255), // Bright blue
    remember: rgb(51, 102, 255),  // Bright blue
    background: rgb(0, 153, 153), // Cyan (color-blind friendly)
    success: rgb(0, 102, 153),    // Blue instead of green for deuteranopia
    error: rgb(204, 0, 0),        // Pure red for better distinction
    warning: rgb(255, 153, 0),    // Orange adjusted for deuteranopia
    merged: rgb(135, 0, 255),     // Electric violet (matches autoAccept)
    warning_shimmer: rgb(255, 183, 50), // Lighter orange for shimmer
    diff_added: rgb(153, 204, 255), // Light blue instead of green
    diff_removed: rgb(255, 204, 204), // Light red
    diff_added_dimmed: rgb(209, 231, 253), // Very light blue
    diff_removed_dimmed: rgb(255, 233, 233), // Very light red
    diff_added_word: rgb(51, 102, 204), // Medium blue (less intense than deep blue)
    diff_removed_word: rgb(153, 51, 51), // Softer red (less intense than deep red)
    // Agent colors (daltonism-friendly)
    red_for_subagents_only: rgb(204, 0, 0),      // Pure red
    blue_for_subagents_only: rgb(0, 102, 204),   // Pure blue
    green_for_subagents_only: rgb(0, 204, 0),    // Pure green
    yellow_for_subagents_only: rgb(255, 204, 0), // Golden yellow
    purple_for_subagents_only: rgb(128, 0, 128), // True purple
    orange_for_subagents_only: rgb(255, 128, 0), // True orange
    pink_for_subagents_only: rgb(255, 102, 178), // Adjusted pink
    cyan_for_subagents_only: rgb(0, 178, 178),   // Adjusted cyan
    // Grove colors
    professional_blue: rgb(106, 155, 204),
    // Chrome colors
    chrome_yellow: rgb(251, 188, 4), // Chrome yellow
    // TUI V2 colors
    clawd_body: rgb(215, 119, 87),
    clawd_background: rgb(0, 0, 0),
    user_message_background: rgb(220, 220, 220), // Slightly darker grey for optimal contrast
    user_message_background_hover: rgb(232, 232, 232), // >=230 to quantize distinct from base at 256-color level
    message_actions_background: rgb(210, 216, 226), // cool gray -- darker than userMsg 220, slight blue
    selection_bg: rgb(180, 213, 255), // light selection blue; daltonized fgs are yellows/blues, both readable on light blue
    bash_message_background_color: rgb(250, 245, 250),
    memory_background_color: rgb(230, 245, 250),
    rate_limit_fill: rgb(51, 102, 255),    // Bright blue
    rate_limit_empty: rgb(23, 46, 114),    // Dark blue
    fast_mode: rgb(255, 106, 0),           // Electric orange (color-blind safe)
    fast_mode_shimmer: rgb(255, 150, 50),  // Lighter orange for shimmer
    brief_label_you: rgb(37, 99, 235),     // Blue
    brief_label_claude: rgb(255, 153, 51), // Orange adjusted for deuteranopia (matches claude)
    rainbow_red: rgb(235, 95, 87),
    rainbow_orange: rgb(245, 139, 87),
    rainbow_yellow: rgb(250, 195, 95),
    rainbow_green: rgb(145, 200, 130),
    rainbow_blue: rgb(130, 170, 220),
    rainbow_indigo: rgb(155, 130, 200),
    rainbow_violet: rgb(200, 130, 180),
    rainbow_red_shimmer: rgb(250, 155, 147),
    rainbow_orange_shimmer: rgb(255, 185, 137),
    rainbow_yellow_shimmer: rgb(255, 225, 155),
    rainbow_green_shimmer: rgb(185, 230, 180),
    rainbow_blue_shimmer: rgb(180, 205, 240),
    rainbow_indigo_shimmer: rgb(195, 180, 230),
    rainbow_violet_shimmer: rgb(230, 180, 210),
});

/// Dark theme using explicit RGB values to avoid inconsistencies
/// from users' custom terminal ANSI color definitions
pub static DARK_THEME: Lazy<Theme> = Lazy::new(|| Theme {
    auto_accept: rgb(175, 135, 255),                    // Electric violet
    bash_border: rgb(253, 93, 177),                     // Bright pink
    claude: rgb(215, 119, 87),                          // Claude orange
    claude_shimmer: rgb(235, 159, 127),                 // Lighter claude orange for shimmer effect
    claude_blue_for_system_spinner: rgb(147, 165, 255), // Blue for system spinner
    claude_blue_shimmer_for_system_spinner: rgb(177, 195, 255), // Lighter blue for system spinner shimmer
    permission: rgb(177, 185, 249),                             // Light blue-purple
    permission_shimmer: rgb(207, 215, 255),                     // Lighter blue-purple for shimmer
    plan_mode: rgb(72, 150, 140),                               // Muted sage green
    ide: rgb(71, 130, 200),                                     // Muted blue
    prompt_border: rgb(136, 136, 136),                          // Medium gray
    prompt_border_shimmer: rgb(166, 166, 166),                  // Lighter gray for shimmer
    text: rgb(255, 255, 255),                                   // White
    inverse_text: rgb(0, 0, 0),                                 // Black
    inactive: rgb(153, 153, 153),                               // Light gray
    inactive_shimmer: rgb(193, 193, 193),                       // Lighter gray for shimmer effect
    subtle: rgb(80, 80, 80),                                    // Dark gray
    suggestion: rgb(177, 185, 249),                             // Light blue-purple
    remember: rgb(177, 185, 249),                               // Light blue-purple
    background: rgb(0, 204, 204),                               // Bright cyan
    success: rgb(78, 186, 101),                                 // Bright green
    error: rgb(255, 107, 128),                                  // Bright red
    warning: rgb(255, 193, 7),                                  // Bright amber
    merged: rgb(175, 135, 255), // Electric violet (matches autoAccept)
    warning_shimmer: rgb(255, 223, 57), // Lighter amber for shimmer
    diff_added: rgb(34, 92, 43), // Dark green
    diff_removed: rgb(122, 41, 54), // Dark red
    diff_added_dimmed: rgb(71, 88, 74), // Very dark green
    diff_removed_dimmed: rgb(105, 72, 77), // Very dark red
    diff_added_word: rgb(56, 166, 96), // Medium green
    diff_removed_word: rgb(179, 89, 107), // Softer red (less intense than bright red)
    // Agent colors
    red_for_subagents_only: rgb(220, 38, 38),     // Red 600
    blue_for_subagents_only: rgb(37, 99, 235),    // Blue 600
    green_for_subagents_only: rgb(22, 163, 74),   // Green 600
    yellow_for_subagents_only: rgb(202, 138, 4),  // Yellow 600
    purple_for_subagents_only: rgb(147, 51, 234), // Purple 600
    orange_for_subagents_only: rgb(234, 88, 12),  // Orange 600
    pink_for_subagents_only: rgb(219, 39, 119),   // Pink 600
    cyan_for_subagents_only: rgb(8, 145, 178),    // Cyan 600
    // Grove colors
    professional_blue: rgb(106, 155, 204),
    // Chrome colors
    chrome_yellow: rgb(251, 188, 4), // Chrome yellow
    // TUI V2 colors
    clawd_body: rgb(215, 119, 87),
    clawd_background: rgb(0, 0, 0),
    user_message_background: rgb(55, 55, 55), // Lighter grey for better visual contrast
    user_message_background_hover: rgb(70, 70, 70),
    message_actions_background: rgb(44, 50, 62), // cool gray, slight blue
    selection_bg: rgb(38, 79, 120), // classic dark-mode selection blue (VS Code dark default); light fgs stay readable
    bash_message_background_color: rgb(65, 60, 65),
    memory_background_color: rgb(55, 65, 70),
    rate_limit_fill: rgb(177, 185, 249),   // Light blue-purple
    rate_limit_empty: rgb(80, 83, 112),    // Medium blue-purple
    fast_mode: rgb(255, 120, 20),          // Electric orange for dark bg
    fast_mode_shimmer: rgb(255, 165, 70),  // Lighter orange for shimmer
    brief_label_you: rgb(122, 180, 232),   // Light blue
    brief_label_claude: rgb(215, 119, 87), // Brand orange
    rainbow_red: rgb(235, 95, 87),
    rainbow_orange: rgb(245, 139, 87),
    rainbow_yellow: rgb(250, 195, 95),
    rainbow_green: rgb(145, 200, 130),
    rainbow_blue: rgb(130, 170, 220),
    rainbow_indigo: rgb(155, 130, 200),
    rainbow_violet: rgb(200, 130, 180),
    rainbow_red_shimmer: rgb(250, 155, 147),
    rainbow_orange_shimmer: rgb(255, 185, 137),
    rainbow_yellow_shimmer: rgb(255, 225, 155),
    rainbow_green_shimmer: rgb(185, 230, 180),
    rainbow_blue_shimmer: rgb(180, 205, 240),
    rainbow_indigo_shimmer: rgb(195, 180, 230),
    rainbow_violet_shimmer: rgb(230, 180, 210),
});

/// Dark daltonized theme (color-blind friendly) using explicit RGB values
/// to avoid inconsistencies from users' custom terminal ANSI color definitions
pub static DARK_DALTONIZED_THEME: Lazy<Theme> = Lazy::new(|| Theme {
    auto_accept: rgb(175, 135, 255),                    // Electric violet
    bash_border: rgb(51, 153, 255),                     // Bright blue
    claude: rgb(255, 153, 51),                          // Orange adjusted for deuteranopia
    claude_shimmer: rgb(255, 183, 101),                 // Lighter orange for shimmer effect
    claude_blue_for_system_spinner: rgb(153, 204, 255), // Light blue for system spinner
    claude_blue_shimmer_for_system_spinner: rgb(183, 224, 255), // Lighter blue for system spinner shimmer
    permission: rgb(153, 204, 255),                             // Light blue
    permission_shimmer: rgb(183, 224, 255),                     // Lighter blue for shimmer
    plan_mode: rgb(102, 153, 153), // Muted gray-teal (works for color-blind)
    ide: rgb(71, 130, 200),        // Muted blue
    prompt_border: rgb(136, 136, 136), // Medium gray
    prompt_border_shimmer: rgb(166, 166, 166), // Lighter gray for shimmer
    text: rgb(255, 255, 255),      // White
    inverse_text: rgb(0, 0, 0),    // Black
    inactive: rgb(153, 153, 153),  // Light gray
    inactive_shimmer: rgb(193, 193, 193), // Lighter gray for shimmer effect
    subtle: rgb(80, 80, 80),       // Dark gray
    suggestion: rgb(153, 204, 255), // Light blue
    remember: rgb(153, 204, 255),  // Light blue
    background: rgb(0, 204, 204),  // Bright cyan (color-blind friendly)
    success: rgb(51, 153, 255),    // Blue instead of green
    error: rgb(255, 102, 102),     // Bright red
    warning: rgb(255, 204, 0),     // Yellow-orange for deuteranopia
    merged: rgb(175, 135, 255),    // Electric violet (matches autoAccept)
    warning_shimmer: rgb(255, 234, 50), // Lighter yellow-orange for shimmer
    diff_added: rgb(0, 68, 102),   // Dark blue
    diff_removed: rgb(102, 0, 0),  // Dark red
    diff_added_dimmed: rgb(62, 81, 91), // Dimmed blue
    diff_removed_dimmed: rgb(62, 44, 44), // Dimmed red
    diff_added_word: rgb(0, 119, 179), // Medium blue
    diff_removed_word: rgb(179, 0, 0), // Medium red
    // Agent colors (daltonism-friendly, dark mode)
    red_for_subagents_only: rgb(255, 102, 102), // Bright red
    blue_for_subagents_only: rgb(102, 178, 255), // Bright blue
    green_for_subagents_only: rgb(102, 255, 102), // Bright green
    yellow_for_subagents_only: rgb(255, 255, 102), // Bright yellow
    purple_for_subagents_only: rgb(178, 102, 255), // Bright purple
    orange_for_subagents_only: rgb(255, 178, 102), // Bright orange
    pink_for_subagents_only: rgb(255, 153, 204), // Bright pink
    cyan_for_subagents_only: rgb(102, 204, 204), // Bright cyan
    // Grove colors
    professional_blue: rgb(106, 155, 204),
    // Chrome colors
    chrome_yellow: rgb(251, 188, 4), // Chrome yellow
    // TUI V2 colors
    clawd_body: rgb(215, 119, 87),
    clawd_background: rgb(0, 0, 0),
    user_message_background: rgb(55, 55, 55), // Lighter grey for better visual contrast
    user_message_background_hover: rgb(70, 70, 70),
    message_actions_background: rgb(44, 50, 62), // cool gray, slight blue
    selection_bg: rgb(38, 79, 120), // classic dark-mode selection blue (VS Code dark default); light fgs stay readable
    bash_message_background_color: rgb(65, 60, 65),
    memory_background_color: rgb(55, 65, 70),
    rate_limit_fill: rgb(153, 204, 255),   // Light blue
    rate_limit_empty: rgb(69, 92, 115),    // Dark blue
    fast_mode: rgb(255, 120, 20),          // Electric orange for dark bg (color-blind safe)
    fast_mode_shimmer: rgb(255, 165, 70),  // Lighter orange for shimmer
    brief_label_you: rgb(122, 180, 232),   // Light blue
    brief_label_claude: rgb(255, 153, 51), // Orange adjusted for deuteranopia (matches claude)
    rainbow_red: rgb(235, 95, 87),
    rainbow_orange: rgb(245, 139, 87),
    rainbow_yellow: rgb(250, 195, 95),
    rainbow_green: rgb(145, 200, 130),
    rainbow_blue: rgb(130, 170, 220),
    rainbow_indigo: rgb(155, 130, 200),
    rainbow_violet: rgb(200, 130, 180),
    rainbow_red_shimmer: rgb(250, 155, 147),
    rainbow_orange_shimmer: rgb(255, 185, 137),
    rainbow_yellow_shimmer: rgb(255, 225, 155),
    rainbow_green_shimmer: rgb(185, 230, 180),
    rainbow_blue_shimmer: rgb(180, 205, 240),
    rainbow_indigo_shimmer: rgb(195, 180, 230),
    rainbow_violet_shimmer: rgb(230, 180, 210),
});

/// Theme names
pub const THEME_NAMES: &[&str] = &[
    "dark",
    "light",
    "light-daltonized",
    "dark-daltonized",
    "light-ansi",
    "dark-ansi",
];

/// Get theme by name
pub fn get_theme(theme_name: &str) -> &'static Theme {
    match theme_name {
        "light" => &LIGHT_THEME,
        "light-ansi" => &LIGHT_ANSI_THEME,
        "dark-ansi" => &DARK_ANSI_THEME,
        "light-daltonized" => &LIGHT_DALTONIZED_THEME,
        "dark-daltonized" => &DARK_DALTONIZED_THEME,
        _ => &DARK_THEME,
    }
}

/// Converts a theme color to an ANSI escape sequence for use with asciichart.
pub fn theme_color_to_ansi(theme_color: &str) -> String {
    // Try to parse rgb(r, g, b) format
    if let Some(caps) = regex::Regex::new(r"rgb\(\s?(\d+),\s?(\d+),\s?(\d+)\s?\)")
        .ok()
        .and_then(|r| r.captures(theme_color))
    {
        let r: u8 = caps[1].parse().unwrap_or(0);
        let g: u8 = caps[2].parse().unwrap_or(0);
        let b: u8 = caps[3].parse().unwrap_or(0);
        // Convert to 256-color ANSI escape
        if r == g && g == b {
            // Grayscale
            let gray = if r < 8 {
                16
            } else if r > 248 {
                231
            } else {
                ((r as f64 - 8.0) / 10.0).floor() as u8 + 232
            };
            return format!("\x1b[38;5;{}m", gray);
        }
        // Color cube
        let r_level = ((r as f64) / 51.0).round() as u8;
        let g_level = ((g as f64) / 51.0).round() as u8;
        let b_level = ((b as f64) / 51.0).round() as u8;
        let color = 16 + r_level * 36 + g_level * 6 + b_level;
        return format!("\x1b[38;5;{}m", color);
    }
    // Fallback to magenta if parsing fails
    "\x1b[35m".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_theme() {
        // Theme retrieval works - just check they return the same pointer
        let dark1 = get_theme("dark");
        let dark2 = get_theme("dark");
        assert!(std::ptr::eq(dark1, dark2));
    }

    #[test]
    fn test_theme_color_to_ansi() {
        // Black
        let result = theme_color_to_ansi("rgb(0, 0, 0)");
        assert!(result.contains("38;5;"));

        // White
        let result = theme_color_to_ansi("rgb(255, 255, 255)");
        assert!(result.contains("38;5;"));

        // Red
        let result = theme_color_to_ansi("rgb(255, 0, 0)");
        assert!(result.contains("38;5;"));
    }
}
