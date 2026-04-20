// Source: ~/claudecode/openclaudecode/src/utils/userAgent.ts
//! User agent string helpers.
//!
//! Single source of truth for all User-Agent values. Returns compile-time
//! `CARGO_PKG_VERSION` so the value is always available.

/// Get the user agent string for all AI agent requests.
pub fn get_user_agent() -> String {
    format!("ai-agent/{}", env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_agent_format() {
        let ua = get_user_agent();
        assert!(ua.starts_with("ai-agent/"));
        assert!(!ua.contains("unknown"));
    }
}
