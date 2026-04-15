use std::env;

pub fn has_embedded_search_tools() -> bool {
    if !is_env_truthy("EMBEDDED_SEARCH_TOOLS") {
        return false;
    }
    if let Ok(e) = env::var("AI_CODE_ENTRYPOINT") {
        return e != "sdk-ts" && e != "sdk-py" && e != "sdk-cli" && e != "local-agent";
    }
    true
}

pub fn embedded_search_tools_binary_path() -> String {
    env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn is_env_truthy(var: &str) -> bool {
    env::var(var)
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false)
}
