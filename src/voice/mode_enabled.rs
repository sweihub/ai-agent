use std::env;

pub fn is_voice_growth_book_enabled() -> bool {
    env::var("VOICE_MODE")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false)
}

pub fn has_voice_auth() -> bool {
    false
}

pub fn is_voice_mode_enabled() -> bool {
    has_voice_auth() && is_voice_growth_book_enabled()
}
