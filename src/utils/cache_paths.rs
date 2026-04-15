use std::path::PathBuf;

pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("claude")
}

pub fn projects(_project_path: &str) -> PathBuf {
    cache_dir().join("projects")
}

pub fn errors() -> PathBuf {
    cache_dir().join("errors")
}

pub fn mcp_logs(_server_name: &str) -> PathBuf {
    cache_dir().join("mcp_logs")
}

pub fn session_logs(_session_id: &str) -> PathBuf {
    cache_dir().join("session_logs")
}
