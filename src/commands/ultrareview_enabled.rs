use crate::constants::env::ai;

pub fn is_ultrareview_enabled() -> bool {
    std::env::var(ai::ULTRAREVIEW_ENABLED)
        .map(|v| v == "true")
        .unwrap_or(false)
}
