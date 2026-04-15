use std::collections::HashMap;
use std::sync::Mutex;

static REPO_CACHE: std::sync::LazyLock<Mutex<HashMap<String, Option<ParsedRepository>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Debug)]
pub struct ParsedRepository {
    pub host: String,
    pub owner: String,
    pub name: String,
}

pub fn clear_repository_caches() {
    REPO_CACHE.lock().unwrap().clear();
}

pub fn parse_git_remote(input: &str) -> Option<ParsedRepository> {
    let trimmed = input.trim();

    if let Some(ssh_match) = regex_lite::Regex::new(r"^git@([^:]+):([^/]+)/([^/]+?)(?:\.git)?$")
        .ok()
        .and_then(|re| re.captures(trimmed))
    {
        let host = ssh_match.get(1)?.as_str();
        if !looks_like_real_hostname(host) {
            return None;
        }
        return Some(ParsedRepository {
            host: host.to_string(),
            owner: ssh_match.get(2)?.as_str().to_string(),
            name: ssh_match.get(3)?.as_str().to_string(),
        });
    }

    if let Some(url_match) = regex_lite::Regex::new(
        r"^(https?|ssh|git)://(?:[^@]+@)?([^/:]+(?::\d+)?)/([^/]+)/([^/]+?)(?:\.git)?$",
    )
    .ok()
    .and_then(|re| re.captures(trimmed))
    {
        let protocol = url_match.get(1)?.as_str();
        let host_with_port = url_match.get(2)?.as_str();
        let host = if protocol == "https" || protocol == "http" {
            host_with_port.to_string()
        } else {
            host_with_port.split(':').next().unwrap_or("").to_string()
        };

        if !looks_like_real_hostname(&host) {
            return None;
        }

        return Some(ParsedRepository {
            host,
            owner: url_match.get(3)?.as_str().to_string(),
            name: url_match.get(4)?.as_str().to_string(),
        });
    }

    None
}

fn looks_like_real_hostname(host: &str) -> bool {
    if !host.contains('.') {
        return false;
    }
    let last_segment = host.split('.').last()?;
    last_segment.chars().all(|c| c.is_alphabetic())
}

pub fn parse_github_repository(input: &str) -> Option<String> {
    let trimmed = input.trim();

    if let Some(parsed) = parse_git_remote(trimmed) {
        if parsed.host != "github.com" {
            return None;
        }
        return Some(format!("{}/{}", parsed.owner, parsed.name));
    }

    if !trimmed.contains("://") && !trimmed.contains('@') && trimmed.contains('/') {
        let parts: Vec<&str> = trimmed.split('/').collect();
        if parts.len() == 2 {
            let repo = parts[1].trim_end_matches(".git");
            return Some(format!("{}/{}", parts[0], repo));
        }
    }

    None
}
