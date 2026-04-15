// Source: ~/claudecode/openclaudecode/src/utils/userAgent.rs

/// Get the AI Code user agent string.
///
/// Kept dependency-free so SDK-bundled code (bridge, cli/transports) can
/// import without pulling in auth and its transitive dependency tree.
pub fn get_ai_code_user_agent() -> String {
    let version = std::env::var("AI_CODE_VERSION")
        .ok()
        .unwrap_or_else(|| "unknown".to_string());
    format!("ai-code/{version}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_agent_format() {
        let ua = get_ai_code_user_agent();
        assert!(ua.starts_with("ai-code/"));
    }
}
