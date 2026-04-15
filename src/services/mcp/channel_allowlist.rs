// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/channelAllowlist.ts
//! Approved channel plugins allowlist

/// Channel allowlist entry
#[derive(Debug, Clone)]
pub struct ChannelAllowlistEntry {
    pub marketplace: String,
    pub plugin: String,
}

/// Get the channel allowlist from GrowthBook
/// Note: This is a simplified version - full implementation would integrate with GrowthBook
pub fn get_channel_allowlist() -> Vec<ChannelAllowlistEntry> {
    // TODO: Integrate with GrowthBook feature flag 'tengu_harbor_ledger'
    // In the TypeScript version:
    // const raw = getFeatureValue_CACHED_MAY_BE_STALE<unknown>('tengu_harbor_ledger', [])
    // const parsed = ChannelAllowlistSchema().safeParse(raw)
    // return parsed.success ? parsed.data : []
    Vec::new()
}

/// Overall channels on/off.
/// Checked before any per-server gating - when false, --channels is a no-op
/// Default false; GrowthBook 5-min refresh
pub fn is_channels_enabled() -> bool {
    // TODO: Integrate with GrowthBook feature flag 'tengu_harbor'
    false
}

/// Pure allowlist check keyed off the connection's pluginSource
/// Returns false for undefined pluginSource and for @-less sources
pub fn is_channel_allowlisted(plugin_source: Option<&str>) -> bool {
    if plugin_source.is_none() {
        return false;
    }

    let source = plugin_source.unwrap();

    // Check if source contains @ (marketplace format)
    if !source.contains('@') {
        return false;
    }

    // Parse marketplace:plugin format
    let parts: Vec<&str> = source.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let (marketplace, name) = (parts[0], parts[1]);

    get_channel_allowlist().iter().any(|e| e.plugin == name && e.marketplace == marketplace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_channels_enabled() {
        // Default is false
        assert!(!is_channels_enabled());
    }

    #[test]
    fn test_is_channel_allowlisted_no_source() {
        assert!(!is_channel_allowlisted(None));
    }

    #[test]
    fn test_is_channel_allowlisted_no_marketplace() {
        assert!(!is_channel_allowlisted(Some("plugin-name")));
    }

    #[test]
    fn test_is_channel_allowlisted_with_marketplace() {
        // Empty allowlist returns false
        assert!(!is_channel_allowlisted(Some("marketplace@plugin")));
    }
}