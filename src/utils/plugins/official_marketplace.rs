// Source: ~/claudecode/openclaudecode/src/utils/plugins/officialMarketplace.ts
#![allow(dead_code)]

use once_cell::sync::Lazy;

use super::schemas::MarketplaceSource;

static OFFICIAL_MARKETPLACE_SOURCE_INSTANCE: Lazy<MarketplaceSource> = Lazy::new(|| {
    MarketplaceSource::Github {
        repo: "anthropics/ai-plugins-official".to_string(),
        ref_: None,
        path: None,
    }
});

/// Source configuration for the official Anthropic plugins marketplace.
pub fn official_marketplace_source() -> MarketplaceSource {
    OFFICIAL_MARKETPLACE_SOURCE_INSTANCE.clone()
}

/// Reference to the official marketplace source.
pub fn get_official_marketplace_source() -> &'static MarketplaceSource {
    &OFFICIAL_MARKETPLACE_SOURCE_INSTANCE
}

/// Display name for the official marketplace.
pub const OFFICIAL_MARKETPLACE_NAME: &str = "ai-plugins-official";
