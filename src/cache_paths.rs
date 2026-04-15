use std::path::PathBuf;

pub struct CachePaths;

impl CachePaths {
    pub fn base_logs(cwd: &str) -> PathBuf {
        let project_dir = sanitize_path(cwd);
        Self::base_cache_dir().join(project_dir)
    }

    pub fn errors(cwd: &str) -> PathBuf {
        Self::base_logs(cwd).join("errors")
    }

    pub fn messages(cwd: &str) -> PathBuf {
        Self::base_logs(cwd).join("messages")
    }

    pub fn mcp_logs(cwd: &str, server_name: &str) -> PathBuf {
        let sanitized_name = sanitize_path(server_name);
        Self::base_logs(cwd).join(format!("mcp-logs-{}", sanitized_name))
    }

    fn base_cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("claude-cli")
    }
}

const MAX_SANITIZED_LENGTH: usize = 200;

fn sanitize_path(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();

    if sanitized.len() <= MAX_SANITIZED_LENGTH {
        return sanitized;
    }

    let hash = djb2_hash(name);
    format!("{}-{}", &sanitized[..MAX_SANITIZED_LENGTH], hash.abs())
}

fn djb2_hash(s: &str) -> i64 {
    let mut hash: i64 = 5381;
    for c in s.bytes() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(c as i64);
    }
    hash
}
