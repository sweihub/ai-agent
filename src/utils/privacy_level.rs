use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacyLevel {
    Default,
    NoTelemetry,
    EssentialTraffic,
}

impl PrivacyLevel {
    pub fn from_env() -> Self {
        if env::var("AI_CODE_DISABLE_NONESSENTIAL_TRAFFIC").is_ok() {
            return PrivacyLevel::EssentialTraffic;
        }
        if env::var("DISABLE_TELEMETRY").is_ok() {
            return PrivacyLevel::NoTelemetry;
        }
        PrivacyLevel::Default
    }
}

pub fn get_privacy_level() -> PrivacyLevel {
    PrivacyLevel::from_env()
}

pub fn is_essential_traffic_only() -> bool {
    get_privacy_level() == PrivacyLevel::EssentialTraffic
}

pub fn is_telemetry_disabled() -> bool {
    get_privacy_level() != PrivacyLevel::Default
}

pub fn get_essential_traffic_only_reason() -> Option<String> {
    if env::var("AI_CODE_DISABLE_NONESSENTIAL_TRAFFIC").is_ok() {
        Some("AI_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_privacy() {
        // Clear env vars for test
        env::remove_var("AI_CODE_DISABLE_NONESSENTIAL_TRAFFIC");
        env::remove_var("DISABLE_TELEMETRY");

        assert_eq!(get_privacy_level(), PrivacyLevel::Default);
        assert!(!is_essential_traffic_only());
        assert!(!is_telemetry_disabled());
    }
}
