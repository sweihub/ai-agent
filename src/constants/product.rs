// Source: /data/home/swei/claudecode/openclaudecode/src/constants/product.ts
pub const PRODUCT_URL: &str = "https://claude.com/claude-code";

pub const CLAUDE_AI_BASE_URL: &str = "https://claude.ai";
pub const CLAUDE_AI_STAGING_BASE_URL: &str = "https://claude-ai.staging.ant.dev";
pub const CLAUDE_AI_LOCAL_BASE_URL: &str = "http://localhost:4000";

pub fn is_remote_session_staging(session_id: Option<&str>, ingress_url: Option<&str>) -> bool {
    session_id.map(|s| s.contains("_staging_")).unwrap_or(false)
        || ingress_url.map(|u| u.contains("staging")).unwrap_or(false)
}

pub fn is_remote_session_local(session_id: Option<&str>, ingress_url: Option<&str>) -> bool {
    session_id.map(|s| s.contains("_local_")).unwrap_or(false)
        || ingress_url
            .map(|u| u.contains("localhost"))
            .unwrap_or(false)
}

pub fn get_claude_ai_base_url(session_id: Option<&str>, ingress_url: Option<&str>) -> String {
    if is_remote_session_local(session_id, ingress_url) {
        CLAUDE_AI_LOCAL_BASE_URL.to_string()
    } else if is_remote_session_staging(session_id, ingress_url) {
        CLAUDE_AI_STAGING_BASE_URL.to_string()
    } else {
        CLAUDE_AI_BASE_URL.to_string()
    }
}

pub fn get_remote_session_url(session_id: &str, ingress_url: Option<&str>) -> String {
    let base_url = get_claude_ai_base_url(Some(session_id), ingress_url);
    format!("{}/code/{}", base_url, session_id)
}
