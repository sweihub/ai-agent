//! Bridge status utilities.
//!
//! Translated from openclaudecode/src/bridge/bridgeStatusUtil.ts

use std::time::{SystemTime, UNIX_EPOCH};

/// Bridge status state machine states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum StatusState {
    Idle,
    Attached,
    Titled,
    Reconnecting,
    Failed,
}

impl StatusState {
    pub fn as_str(&self) -> &'static str {
        match self {
            StatusState::Idle => "idle",
            StatusState::Attached => "attached",
            StatusState::Titled => "titled",
            StatusState::Reconnecting => "reconnecting",
            StatusState::Failed => "failed",
        }
    }
}

/// How long a tool activity line stays visible after last tool_start (ms).
pub const TOOL_DISPLAY_EXPIRY_MS: u64 = 30_000;

/// Interval for the shimmer animation tick (ms).
pub const SHIMMER_INTERVAL_MS: u64 = 150;

/// Get current timestamp as HH:MM:SS
pub fn timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let h = (now / 3600) % 24;
    let m = (now / 60) % 60;
    let s = now % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

/// Format duration in human-readable form
pub fn format_duration(ms: u64) -> String {
    let seconds = ms / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes % 60)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds % 60)
    } else {
        format!("{}s", seconds)
    }
}

/// Truncate text to a specific visual width
pub fn truncate_to_width(text: &str, max_width: usize) -> String {
    // Simplified: just use character count
    if text.len() <= max_width {
        text.to_string()
    } else {
        format!("{}...", &text[..max_width.saturating_sub(3)])
    }
}

/// Abbreviate a tool activity summary for the trail display.
pub fn abbreviate_activity(summary: &str) -> String {
    truncate_to_width(summary, 30)
}

/// Build the connect URL shown when the bridge is idle.
pub fn build_bridge_connect_url(environment_id: &str, ingress_url: Option<&str>) -> String {
    let base_url = get_claude_ai_base_url(None, ingress_url);
    format!("{}/code?bridge={}", base_url, environment_id)
}

/// Get Claude AI base URL
fn get_claude_ai_base_url(_env: Option<&str>, ingress_url: Option<&str>) -> String {
    ingress_url
        .map(|s| s.to_string())
        .unwrap_or_else(|| "https://claude.ai".to_string())
}

/// Build the session URL shown when a session is attached.
/// Appends the v1-specific ?bridge={environmentId} query.
pub fn build_bridge_session_url(
    session_id: &str,
    environment_id: &str,
    ingress_url: Option<&str>,
) -> String {
    let base = get_remote_session_url(session_id, ingress_url);
    format!("{}?bridge={}", base, environment_id)
}

/// Get remote session URL
fn get_remote_session_url(session_id: &str, ingress_url: Option<&str>) -> String {
    let base_url = get_claude_ai_base_url(None, ingress_url);
    // Convert cse_ prefix to session_ for compat gateway
    let compat_id = session_id.replace("cse_", "session_");
    format!("{}/code/{}", base_url, compat_id)
}

/// Compute the glimmer index for a reverse-sweep shimmer animation.
pub fn compute_glimmer_index(tick: u64, message_width: u64) -> u64 {
    let cycle_length = message_width + 20;
    message_width + 10 - (tick % cycle_length)
}

/// Split text into three segments by visual column position for shimmer rendering.
///
/// Uses grapheme segmentation and string width so the split is correct for
/// multi-byte characters, emoji, and CJK glyphs.
///
/// Returns (before, shimmer, after) strings. Both renderers (chalk in
/// bridgeUI.ts and React/Ink in bridge.tsx) apply their own coloring to
/// these segments.
pub fn compute_shimmer_segments(text: &str, glimmer_index: u64) -> (String, String, String) {
    let message_width = string_width(text) as u64;
    let shimmer_start = glimmer_index as i64 - 1;
    let shimmer_end = glimmer_index as i64 + 1;

    // When shimmer is offscreen, return all text as "before"
    if shimmer_start >= message_width as i64 || shimmer_end < 0 {
        return (text.to_string(), String::new(), String::new());
    }

    // Split into at most 3 segments by visual column position
    let clamped_start = shimmer_start.max(0) as usize;
    let mut col_pos = 0usize;
    let mut before = String::new();
    let mut shimmer = String::new();
    let mut after = String::new();

    // Simplified: just use character iteration
    for c in text.chars() {
        let seg_width = string_width(&c.to_string()) as usize;
        if col_pos + seg_width <= clamped_start {
            before.push(c);
        } else if col_pos > shimmer_end as usize {
            after.push(c);
        } else {
            shimmer.push(c);
        }
        col_pos += seg_width;
    }

    (before, shimmer, after)
}

/// Get visual width of a string (simplified)
fn string_width(s: &str) -> usize {
    // Simplified: count actual characters
    // Full implementation would handle Unicode, emoji, CJK
    s.chars().count()
}

/// Bridge status label and color from connection state.
#[derive(Debug, Clone)]
pub struct BridgeStatusInfo {
    pub label: BridgeStatusLabel,
    pub color: BridgeStatusColor,
}

/// Bridge status label
#[derive(Debug, Clone)]
pub enum BridgeStatusLabel {
    RemoteControlFailed,
    RemoteControlReconnecting,
    RemoteControlActive,
    RemoteControlConnecting,
}

impl BridgeStatusLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            BridgeStatusLabel::RemoteControlFailed => "Remote Control failed",
            BridgeStatusLabel::RemoteControlReconnecting => "Remote Control reconnecting",
            BridgeStatusLabel::RemoteControlActive => "Remote Control active",
            BridgeStatusLabel::RemoteControlConnecting => "Remote Control connecting...",
        }
    }
}

/// Bridge status color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeStatusColor {
    Error,
    Warning,
    Success,
}

/// Derive a status label and color from the bridge connection state.
pub fn get_bridge_status(params: &GetBridgeStatusParams) -> BridgeStatusInfo {
    if params.error.is_some() {
        return BridgeStatusInfo {
            label: BridgeStatusLabel::RemoteControlFailed,
            color: BridgeStatusColor::Error,
        };
    }
    if params.reconnecting {
        return BridgeStatusInfo {
            label: BridgeStatusLabel::RemoteControlReconnecting,
            color: BridgeStatusColor::Warning,
        };
    }
    if params.session_active || params.connected {
        return BridgeStatusInfo {
            label: BridgeStatusLabel::RemoteControlActive,
            color: BridgeStatusColor::Success,
        };
    }
    BridgeStatusInfo {
        label: BridgeStatusLabel::RemoteControlConnecting,
        color: BridgeStatusColor::Warning,
    }
}

/// Parameters for get_bridge_status
pub struct GetBridgeStatusParams<'a> {
    pub error: Option<&'a str>,
    pub connected: bool,
    pub session_active: bool,
    pub reconnecting: bool,
}

/// Footer text shown when bridge is idle (Ready state).
pub fn build_idle_footer_text(url: &str) -> String {
    format!("Code everywhere with the Claude app or {}", url)
}

/// Footer text shown when a session is active (Connected state).
pub fn build_active_footer_text(url: &str) -> String {
    format!("Continue coding in the Claude app or {}", url)
}

/// Footer text shown when the bridge has failed.
pub const FAILED_FOOTER_TEXT: &str = "Something went wrong, please try again";

/// Wrap text in an OSC 8 terminal hyperlink. Zero visual width for layout purposes.
/// strip-ansi (used by stringWidth) correctly strips these sequences.
pub fn wrap_with_osc8_link(text: &str, url: &str) -> String {
    format!("\x1b]8;;{}\x07{}\x1b]8;;\x07", url, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp() {
        let ts = timestamp();
        assert_eq!(ts.len(), 8);
        assert!(ts.contains(':'));
    }

    #[test]
    fn test_truncate_to_width() {
        assert_eq!(truncate_to_width("hello", 10), "hello");
        assert_eq!(truncate_to_width("hello world", 8), "hello...");
    }

    #[test]
    fn test_compute_glimmer_index() {
        assert_eq!(compute_glimmer_index(0, 50), 60);
        assert_eq!(compute_glimmer_index(10, 50), 50);
    }
}
