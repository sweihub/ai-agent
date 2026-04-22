// Source: ~/claudecode/openclaudecode/src/utils/plugins/parseMarketplaceInput.ts
#![allow(dead_code)]

use std::path::PathBuf;

use super::schemas::MarketplaceSource;

/// Parse a marketplace input string and return the appropriate marketplace source type.
pub fn parse_marketplace_input(input: &str) -> Result<Option<MarketplaceSource>, String> {
    let trimmed = input.trim();

    // Handle git SSH URLs with any valid username
    // Pattern: user@host:path or user@host:path.git, optionally with #ref suffix
    if let Some(ssh_match) = parse_ssh_url(trimmed) {
        return Ok(Some(ssh_match));
    }

    // Handle URLs
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return Ok(Some(parse_http_url(trimmed)));
    }

    // Handle local paths
    if is_local_path(trimmed) {
        return parse_local_path(trimmed);
    }

    // Handle GitHub shorthand (owner/repo, owner/repo#ref, or owner/repo@ref)
    if trimmed.contains('/') && !trimmed.starts_with('@') {
        if trimmed.contains(':') {
            return Ok(None);
        }
        return Ok(Some(parse_github_shorthand(trimmed)));
    }

    Ok(None)
}

fn parse_ssh_url(input: &str) -> Option<MarketplaceSource> {
    // Pattern: user@host:path.git or user@host:path, optionally with #ref
    let without_ref = if let Some(hash_pos) = input.find('#') {
        (&input[..hash_pos], Some(&input[hash_pos + 1..]))
    } else if let Some(at_pos) = input.rfind('@') {
        // Check for @ as ref separator (not the username @)
        if let Some(slash_pos) = input.find('/') {
            if at_pos > slash_pos {
                (&input[..at_pos], Some(&input[at_pos + 1..]))
            } else {
                (input, None)
            }
        } else {
            (input, None)
        }
    } else {
        (input, None)
    };

    let (url, ref_) = without_ref;

    // Match SSH pattern: user@host:path
    if let Some(colon_pos) = url.find(':') {
        let before_colon = &url[..colon_pos];
        if before_colon.contains('@') {
            return Some(MarketplaceSource::Git {
                url: url.to_string(),
                ref_: ref_.map(|s| s.to_string()),
                path: None,
            });
        }
    }

    None
}

fn parse_http_url(input: &str) -> MarketplaceSource {
    // Extract fragment (ref) from URL if present
    let (url_without_fragment, ref_) = if let Some(hash_pos) = input.find('#') {
        (&input[..hash_pos], Some(&input[hash_pos + 1..]))
    } else {
        (input, None)
    };

    // Check if it looks like a git repo
    let looks_like_git =
        url_without_fragment.ends_with(".git") || url_without_fragment.contains("/_git/");

    if looks_like_git {
        return MarketplaceSource::Git {
            url: url_without_fragment.to_string(),
            ref_: ref_.map(|s| s.to_string()),
            path: None,
        };
    }

    // Check if it's a GitHub HTTPS URL
    if let Ok(url) = url::Url::parse(url_without_fragment) {
        if url
            .host_str()
            .map_or(false, |h| h == "github.com" || h == "www.github.com")
        {
            let path = url.path();
            if let Some(captures) = regex::Regex::new(r"^/([^/]+/[^/]+?)(/|\.git|$)")
                .ok()
                .and_then(|re| re.captures(path))
            {
                let repo = captures.get(1).map(|m| m.as_str()).unwrap_or("");
                let git_url = if url_without_fragment.ends_with(".git") {
                    url_without_fragment.to_string()
                } else {
                    format!("{}.git", url_without_fragment)
                };

                return MarketplaceSource::Git {
                    url: git_url,
                    ref_: ref_.map(|s| s.to_string()),
                    path: None,
                };
            }
        }
    }

    MarketplaceSource::Url {
        url: url_without_fragment.to_string(),
    }
}

fn is_local_path(input: &str) -> bool {
    input.starts_with("./")
        || input.starts_with("../")
        || input.starts_with('/')
        || input.starts_with('~')
        || (cfg!(windows)
            && (input.starts_with(".\\")
                || input.starts_with("..\\")
                || (input.len() >= 3
                    && input.chars().nth(1) == Some(':')
                    && (input.chars().nth(2) == Some('/') || input.chars().nth(2) == Some('\\')))))
}

fn parse_local_path(input: &str) -> Result<Option<MarketplaceSource>, String> {
    let resolved = if input.starts_with('~') {
        let home = dirs::home_dir().ok_or("Cannot determine home directory")?;
        home.join(&input[1..])
    } else {
        PathBuf::from(input)
    };

    let resolved = resolved.canonicalize().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            format!("Path does not exist: {}", resolved.display())
        } else {
            format!("Cannot access path: {} ({})", resolved.display(), e)
        }
    })?;

    let metadata = std::fs::metadata(&resolved)
        .map_err(|e| format!("Cannot stat path: {} ({})", resolved.display(), e))?;

    if metadata.is_file() {
        if resolved.extension().map_or(false, |e| e == "json") {
            Ok(Some(MarketplaceSource::File {
                path: resolved.to_string_lossy().to_string(),
            }))
        } else {
            Err(format!(
                "File path must point to a .json file, but got: {}",
                resolved.display()
            ))
        }
    } else if metadata.is_dir() {
        Ok(Some(MarketplaceSource::Directory {
            path: resolved.to_string_lossy().to_string(),
        }))
    } else {
        Err(format!(
            "Path is neither a file nor a directory: {}",
            resolved.display()
        ))
    }
}

fn parse_github_shorthand(input: &str) -> MarketplaceSource {
    // Extract ref if present (either #ref or @ref)
    let (repo, ref_) = if let Some(hash_pos) = input.find('#') {
        (&input[..hash_pos], Some(&input[hash_pos + 1..]))
    } else if let Some(at_pos) = input.rfind('@') {
        if let Some(slash_pos) = input.find('/') {
            if at_pos > slash_pos {
                (&input[..at_pos], Some(&input[at_pos + 1..]))
            } else {
                (input, None)
            }
        } else {
            (input, None)
        }
    } else {
        (input, None)
    };

    MarketplaceSource::Github {
        repo: repo.to_string(),
        ref_: ref_.map(|s| s.to_string()),
        path: None,
    }
}
