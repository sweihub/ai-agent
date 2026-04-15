pub fn get_extra_usage_info() -> ExtraUsageInfo {
    ExtraUsageInfo {
        enabled: false,
        remaining: 0,
        total: 0,
    }
}

#[derive(Clone, Debug)]
pub struct ExtraUsageInfo {
    pub enabled: bool,
    pub remaining: u64,
    pub total: u64,
}

pub fn is_extra_usage_enabled() -> bool {
    false
}

pub fn check_extra_usage_balance() -> Result<(), String> {
    Err("Extra usage not available".to_string())
}
