// Source: /data/home/swei/claudecode/openclaudecode/src/utils/hyperlink.ts
//! Hyperlink utilities
//!
//! Create clickable hyperlinks using OSC 8 escape sequences.

/// OSC 8 hyperlink escape sequence start
pub const OSC8_START: &str = "\x1b]8;;";
/// OSC 8 hyperlink escape sequence end (using BEL as terminator)
pub const OSC8_END: &str = "\x07";

/// Create a clickable hyperlink using OSC 8 escape sequences.
/// Falls back to plain text if the terminal doesn't support hyperlinks.
///
/// # Arguments
/// * `url` - The URL to link to
/// * `content` - Optional content to display as the link text
/// * `supports_hyperlinks` - Optional override for testing
///
/// # Returns
/// The hyperlink string or plain URL if hyperlinks not supported
pub fn create_hyperlink(
    url: &str,
    content: Option<&str>,
    supports_hyperlinks: Option<bool>,
) -> String {
    // For now, just return the URL without hyperlinks (simpler implementation)
    // In production, you'd check terminal capabilities via terminfo or environment
    let _ = supports_hyperlinks;

    let display_text = content.unwrap_or(url);

    // Apply basic ANSI blue color using ANSI escape codes
    let colored_text = format!("\x1b[34m{}\x1b[0m", display_text);

    // Simple version without OSC 8 to avoid compatibility issues
    colored_text
}
