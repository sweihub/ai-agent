// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/channelNotification.ts
//! Channel notifications - lets an MCP server push user messages into the conversation

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Channel message notification parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMessageParams {
    pub content: String,
    /// Opaque passthrough - thread_id, user, whatever the channel wants the model to see
    #[serde(default)]
    pub meta: Option<HashMap<String, String>>,
}

/// Channel message notification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMessageNotification {
    method: String,
    params: ChannelMessageParams,
}

/// Channel entry from bootstrap state
#[derive(Debug, Clone)]
pub struct ChannelEntry {
    pub name: String,
    pub plugin_source: Option<String>,
}

/// Get allowed channels from bootstrap state
/// Note: This would integrate with bootstrap state in full implementation
pub fn get_allowed_channels() -> Vec<ChannelEntry> {
    // TODO: Integrate with bootstrap state
    Vec::new()
}

/// Check if channels are enabled (feature gate + OAuth check)
pub fn is_channels_enabled() -> bool {
    // TODO: Check feature gates and OAuth
    // feature('KAIROS') || feature('KAIROS_CHANNELS')
    // Also check claude.ai OAuth
    false
}

/// Check if channels are allowed for user/org
pub fn can_use_channels() -> bool {
    // Check if channels enabled and user has OAuth
    is_channels_enabled()
}

/// Parse channel message notification
pub fn parse_channel_message(notification: &ChannelMessageNotification) -> Option<&ChannelMessageParams> {
    if notification.method == "notifications/claude/channel" {
        Some(&notification.params)
    } else {
        None
    }
}

/// Format channel message as XML for display
pub fn format_channel_message(content: &str, meta: Option<&HashMap<String, String>>) -> String {
    let mut attrs = String::new();
    if let Some(m) = meta {
        for (k, v) in m {
            attrs.push_str(&format!(" {}=\"{}\"", k, v.replace('"', "&quot;")));
        }
    }
    format!("<channel{}>{}</channel>", attrs, content)
}