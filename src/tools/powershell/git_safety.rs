// Source: /data/home/swei/claudecode/openclaudecode/src/tools/PowerShellTool/gitSafety.ts
//! Git security checks for PowerShell tool
//!
//! Git can be weaponized for sandbox escape via two vectors:
//! 1. Bare-repo attack: if cwd contains HEAD + objects/ + refs/ but no valid
//!    .git/HEAD, Git treats cwd as a bare repository and runs hooks from cwd.
//! 2. Git-internal write + git: a compound command creates HEAD/objects/refs/
//!    hooks/ then runs git — the git subcommand executes the freshly-created
//!    malicious hooks.

use std::path::Path;

/// Git internal prefixes that could be used for attacks
const GIT_INTERNAL_PREFIXES: &[&str] = &["head", "objects", "refs", "hooks"];

/// Normalize PowerShell path argument for git internal matching
fn normalize_git_path_arg(arg: &str) -> String {
    let mut s = arg.to_string();

    // Normalize parameter prefixes: dash chars and forward-slash
    // /Path:hooks/pre-commit → extract colon-bound value
    if !s.is_empty() {
        let first_char = s.chars().next().unwrap();
        if first_char == '/' || "–—―".contains(first_char) {
            if let Some(c) = s[1..].find(':') {
                s = s[c + 1..].to_string();
            }
        }
    }

    // Strip surrounding quotes
    s = s.trim_matches('"').trim_matches('\'').to_string();

    // Strip backtick escapes
    s = s.replace('`', "");

    // PS provider-qualified path: FileSystem::hooks/pre-commit → hooks/pre-commit
    // Also handles: Microsoft.PowerShell.Core\FileSystem::path
    let provider_prefixes = ["FileSystem::", "Microsoft.PowerShell.Core\\FileSystem::"];
    for prefix in provider_prefixes {
        if s.to_lowercase().starts_with(&prefix.to_lowercase()) {
            if let Some(pos) = s.find(prefix) {
                s = s[pos + prefix.len()..].to_string();
            }
        }
    }

    // Drive-relative C:foo (no separator after colon) is cwd-relative
    // C:\foo (WITH separator) is absolute
    if s.len() >= 2 && s.chars().nth(1) == Some(':') {
        if !s
            .chars()
            .nth(2)
            .map(|c| c == '/' || c == '\\')
            .unwrap_or(false)
        {
            s = s[2..].to_string();
        }
    }

    // Normalize backslashes to forward slashes
    s = s.replace('\\', "/");

    // Win32 CreateFileW per-component: strip trailing spaces, then trailing dots
    // But preserve . and ..
    let parts: Vec<String> = s
        .split('/')
        .map(|c| {
            if c.is_empty() || c == "." || c == ".." {
                c.to_string()
            } else {
                let mut c = c.trim_end().to_string();
                while c.ends_with('.') && c != "." && c != ".." {
                    c.pop();
                }
                if c.is_empty() { ".".to_string() } else { c }
            }
        })
        .collect();

    s = parts.join("/");

    // Normalize path (resolve .. and .)
    s = normalize_path(&s);

    // Remove leading ./
    if s.starts_with("./") {
        s = s[2..].to_string();
    }

    s.to_lowercase()
}

/// Simple path normalization
fn normalize_path(path: &str) -> String {
    let mut result = Vec::new();
    for part in path.split('/') {
        match part {
            "." | "" => continue,
            ".." => {
                if !result.is_empty() {
                    result.pop();
                }
            }
            _ => result.push(part),
        }
    }
    let result = result.join("/");
    if result.is_empty() {
        ".".to_string()
    } else {
        result
    }
}

/// Get cwd basename
fn get_cwd_basename() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()))
        .unwrap_or_default()
}

/// If a normalized path starts with `../<cwd-basename>/`, resolve it to cwd-relative
fn resolve_cwd_reentry(normalized: &str) -> String {
    if !normalized.starts_with("../") {
        return normalized.to_string();
    }

    let cwd_base = get_cwd_basename();
    if cwd_base.is_empty() {
        return normalized.to_string();
    }

    let prefix = format!("../{}/", cwd_base);
    let mut s = normalized.to_string();
    while s.starts_with(&prefix) {
        s = s[prefix.len()..].to_string();
    }

    // Also handle exact `../<cwd-basename>` (no trailing slash)
    let exact = format!("../{}", cwd_base);
    if s == exact {
        return ".".to_string();
    }

    s
}

/// Check if path matches git internal prefix
fn matches_git_internal_prefix(n: &str) -> bool {
    if n == "head" || n == ".git" {
        return true;
    }
    if n.starts_with(".git/") || n.starts_with("git~") {
        return true;
    }
    for p in GIT_INTERNAL_PREFIXES {
        if p == &"head" {
            continue;
        }
        if n == *p || n.starts_with(&format!("{}/", p)) {
            return true;
        }
    }
    false
}

/// Check if path matches .git prefix
fn matches_dot_git_prefix(n: &str) -> bool {
    if n == ".git" || n.starts_with(".git/") {
        return true;
    }
    // NTFS 8.3 short names
    regex::Regex::new(r"^git~\d+($|/)")
        .ok()
        .map(|re| re.is_match(n))
        .unwrap_or(false)
}

/// Resolve escaping path to cwd-relative
fn resolve_escaping_path_to_cwd_relative(n: &str) -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let cwd_str = cwd.to_string_lossy();

    // Resolve path against cwd
    let resolved = cwd.join(n);
    let resolved_str = resolved.to_string_lossy().to_lowercase();
    let cwd_lower = cwd_str.to_lowercase();

    if resolved_str == cwd_lower {
        return Some(".".to_string());
    }

    let cwd_with_sep = if cwd_lower.ends_with('/') || cwd_lower.ends_with('\\') {
        cwd_lower.clone()
    } else {
        format!("{}/", cwd_lower)
    };

    if !resolved_str.starts_with(&cwd_with_sep) {
        return None;
    }

    let rel = &resolved_str[cwd_with_sep.len()..];
    Some(rel.replace('\\', "/"))
}

/// True if arg resolves to a git-internal path in cwd
pub fn is_git_internal_path_ps(arg: &str) -> bool {
    let n = resolve_cwd_reentry(&normalize_git_path_arg(arg));
    if matches_git_internal_prefix(&n) {
        return true;
    }

    // SECURITY: leading `../` or absolute paths
    if n.starts_with("../") || n.starts_with('/') || n.len() >= 2 && n.chars().nth(1) == Some(':') {
        if let Some(rel) = resolve_escaping_path_to_cwd_relative(&n) {
            if matches_git_internal_prefix(&rel) {
                return true;
            }
        }
    }

    false
}

/// True if arg resolves to a path inside .git/
pub fn is_dot_git_path_ps(arg: &str) -> bool {
    let n = resolve_cwd_reentry(&normalize_git_path_arg(arg));
    if matches_dot_git_prefix(&n) {
        return true;
    }

    // SECURITY: same cwd-resolution as isGitInternalPathPS
    if n.starts_with("../") || n.starts_with('/') || n.len() >= 2 && n.chars().nth(1) == Some(':') {
        if let Some(rel) = resolve_escaping_path_to_cwd_relative(&n) {
            if matches_dot_git_prefix(&rel) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_git_path() {
        assert_eq!(
            normalize_git_path_arg("hooks/pre-commit"),
            "hooks/pre-commit"
        );
        assert_eq!(normalize_git_path_arg(".git/hooks"), ".git/hooks");
    }
}
