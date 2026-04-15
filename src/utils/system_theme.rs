/**
 * Terminal dark/light mode detection for the 'auto' theme setting.
 *
 * Detection is based on the terminal's actual background color (queried via
 * OSC 11 by systemThemeWatcher.ts) rather than the OS appearance setting --
 * a dark terminal on a light-mode OS should still resolve to 'dark'.
 *
 * The detected theme is cached module-level so callers can resolve 'auto'
 * without awaiting the async OSC round-trip. The cache is seeded from
 * $COLORFGBG (synchronous, set by some terminals at launch) and then
 * updated by the watcher once the OSC 11 response arrives.
 */
use crate::utils::config::ThemeSetting;
use once_cell::sync::Lazy;
use std::env;
use std::sync::Mutex;

/// System theme detected from terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemTheme {
    Dark,
    Light,
}

static CACHED_SYSTEM_THEME: Lazy<Mutex<Option<SystemTheme>>> = Lazy::new(|| Mutex::new(None));

/// Get the current terminal theme. Cached after first detection; the watcher
/// updates the cache on live changes.
pub fn get_system_theme_name() -> SystemTheme {
    let mut cached = CACHED_SYSTEM_THEME.lock().unwrap();
    if cached.is_none() {
        *cached = detect_from_color_fg_bg().or(Some(SystemTheme::Dark));
    }
    cached.unwrap_or(SystemTheme::Dark)
}

/// Update the cached terminal theme. Called by the watcher when the OSC 11
/// query returns so non-React call sites stay in sync.
pub fn set_cached_system_theme(theme: SystemTheme) {
    let mut cached = CACHED_SYSTEM_THEME.lock().unwrap();
    *cached = Some(theme);
}

/// Resolve a ThemeSetting (which may be 'auto') to a concrete theme name.
pub fn resolve_theme_setting(setting: &ThemeSetting) -> &'static str {
    match setting {
        ThemeSetting::System => {
            if get_system_theme_name() == SystemTheme::Light {
                "light"
            } else {
                "dark"
            }
        }
        ThemeSetting::Dark => "dark",
        ThemeSetting::Light => "light",
    }
}

/// Parse an OSC color response data string into a theme.
///
/// Accepts XParseColor formats returned by OSC 10/11 queries:
/// - `rgb:R/G/B` where each component is 1-4 hex digits (each scaled to
///   [0, 16^n - 1] for n digits). This is what xterm, iTerm2, Terminal.app,
///   Ghostty, kitty, Alacritty, etc. return.
/// - `#RRGGBB` / `#RRRRGGGGBBBB` (rare, but cheap to accept).
///
/// Returns None for unrecognized formats so callers can fall back.
pub fn theme_from_osc_color(data: &str) -> Option<SystemTheme> {
    let rgb = parse_osc_rgb(data)?;
    // ITU-R BT.709 relative luminance. Midpoint split: > 0.5 is light.
    let luminance = 0.2126 * rgb.r + 0.7152 * rgb.g + 0.0722 * rgb.b;
    if luminance > 0.5 {
        Some(SystemTheme::Light)
    } else {
        Some(SystemTheme::Dark)
    }
}

#[derive(Debug, Clone, Copy)]
struct Rgb {
    r: f64,
    g: f64,
    b: f64,
}

fn parse_osc_rgb(data: &str) -> Option<Rgb> {
    // rgb:RRRR/GGGG/BBBB -- each component is 1-4 hex digits.
    // Some terminals append an alpha component (rgba:.../.../.../...); ignore it.
    let data_lower = data.to_lowercase();
    if let Some(caps) = regex::Regex::new(r"^rgba?:([0-9a-f]{1,4})/([0-9a-f]{1,4})/([0-9a-f]{1,4})")
        .ok()?
        .captures(&data_lower)
    {
        return Some(Rgb {
            r: hex_component(&caps[1]),
            g: hex_component(&caps[2]),
            b: hex_component(&caps[3]),
        });
    }
    // #RRGGBB or #RRRRGGGGBBBB -- split into three equal hex runs.
    if let Some(caps) = regex::Regex::new(r"^#([0-9a-f]+)$")
        .ok()?
        .captures(&data_lower)
    {
        let hex = &caps[1];
        if hex.len() % 3 == 0 {
            let n = hex.len() / 3;
            return Some(Rgb {
                r: hex_component(&hex[..n]),
                g: hex_component(&hex[n..2 * n]),
                b: hex_component(&hex[2 * n..]),
            });
        }
    }
    None
}

/// Normalize a 1-4 digit hex component to [0, 1].
fn hex_component(hex: &str) -> f64 {
    let max = 16_f64.powi(hex.len() as i32) - 1.0;
    let value = u64::from_str_radix(hex, 16).unwrap_or(0) as f64;
    value / max
}

/// Read $COLORFGBG for a synchronous initial guess before the OSC 11
/// round-trip completes. Format is `fg;bg` (or `fg;other;bg`) where values
/// are ANSI color indices. rxvt convention: bg 0-6 or 8 are dark; bg 7
/// and 9-15 are light. Only set by some terminals (rxvt-family, Konsole,
/// iTerm2 with the option enabled), so this is a best-effort hint.
fn detect_from_color_fg_bg() -> Option<SystemTheme> {
    let colorfgbg = env::var("COLORFGBG").ok()?;
    let parts: Vec<&str> = colorfgbg.split(';').collect();
    let bg = parts.last()?;
    if bg.is_empty() {
        return None;
    }
    let bg_num: i32 = bg.parse().ok()?;
    if bg_num < 0 || bg_num > 15 {
        return None;
    }
    // 0-6 and 8 are dark ANSI colors; 7 (white) and 9-15 (bright) are light.
    if bg_num <= 6 || bg_num == 8 {
        Some(SystemTheme::Dark)
    } else {
        Some(SystemTheme::Light)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_from_osc_color_rgb() {
        // Dark background (low luminance)
        assert_eq!(theme_from_osc_color("rgb:0/0/0"), Some(SystemTheme::Dark));
        // Light background (high luminance)
        assert_eq!(
            theme_from_osc_color("rgb:ff/ff/ff"),
            Some(SystemTheme::Light)
        );
        // Mid-point (0x80 = 128/255 ≈ 0.502 > 0.5, so light)
        assert_eq!(
            theme_from_osc_color("rgb:80/80/80"),
            Some(SystemTheme::Light)
        );
    }

    #[test]
    fn test_theme_from_osc_color_hash() {
        assert_eq!(theme_from_osc_color("#000000"), Some(SystemTheme::Dark));
        assert_eq!(theme_from_osc_color("#ffffff"), Some(SystemTheme::Light));
    }

    #[test]
    fn test_resolve_theme_setting() {
        assert_eq!(resolve_theme_setting(&ThemeSetting::Dark), "dark");
        assert_eq!(resolve_theme_setting(&ThemeSetting::Light), "light");
        // System resolves to current system theme
        let _ = resolve_theme_setting(&ThemeSetting::System);
    }
}
