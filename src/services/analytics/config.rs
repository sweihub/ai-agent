// Source: /data/home/swei/claudecode/openclaudecode/src/services/autoDream/config.ts
pub fn get_analytics_config() -> AnalyticsConfig {
    AnalyticsConfig::default()
}

#[derive(Debug, Clone, Default)]
pub struct AnalyticsConfig {
    pub enabled: bool,
}
