//! AI.md loader implementation

use crate::ai_md::types::*;
use crate::ai_md::AI_MD_INSTRUCTION_PROMPT;
use crate::error::AgentError;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Filename for AI.md files
pub const AI_MD_FILENAME: &str = "AI.md";
/// Alias for AI_MD_FILENAME
pub const CLAUDE_MD_FILENAME: &str = "AI.md";
/// Filename for local AI.md files
pub const AI_MD_LOCAL_FILENAME: &str = "AI.local.md";
/// Alias for AI_MD_LOCAL_FILENAME
pub const CLAUDE_LOCAL_MD_FILENAME: &str = "AI.local.md";
/// Rules directory name (for project-level rules)
pub const PROJECT_RULES_DIR: &str = ".ai/rules";

/// Maximum depth for @include recursion
const MAX_INCLUDE_DEPTH: usize = 5;

/// Home directory cache
static HOME_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Get home directory
fn get_home_dir() -> Option<&'static PathBuf> {
    HOME_DIR.get_or_init(|| dirs::home_dir()).as_ref()
}

/// Get config directory (~/.ai)
fn get_config_dir() -> Option<PathBuf> {
    get_home_dir().map(|h| h.join(".ai"))
}

/// Get managed rules directory (/etc/ai-code/.ai/rules)
fn get_managed_rules_dir() -> Option<PathBuf> {
    PathBuf::from("/etc/ai-code/.ai/rules")
        .exists()
        .then_some(PathBuf::from("/etc/ai-code/.ai/rules"))
}

/// Get user rules directory (~/.ai/rules)
fn get_user_rules_dir() -> Option<PathBuf> {
    get_config_dir()
        .map(|p| p.join("rules"))
        .filter(|p| p.exists())
}

/// Check if a file extension is text
#[allow(dead_code)]
fn is_text_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    is_allowed_extension(&format!(".{}", ext))
}

/// Parse frontmatter from markdown content
fn parse_frontmatter(content: &str) -> ParsedAiMd {
    if !content.starts_with("---") {
        return ParsedAiMd {
            frontmatter: AiMdFrontmatter::default(),
            content: content.to_string(),
        };
    }

    // Find closing ---
    if let Some(end_idx) = content[3..].find("---") {
        let frontmatter_str = &content[3..end_idx + 3];
        let body = &content[end_idx + 6..];

        // Parse YAML frontmatter (simple parsing)
        let mut frontmatter = AiMdFrontmatter::default();

        for line in frontmatter_str.lines() {
            let line = line.trim();
            if line.starts_with("paths:") {
                let value = line["paths:".len()..].trim();
                if value.starts_with('[') && value.ends_with(']') {
                    let inner = &value[1..value.len() - 1];
                    frontmatter.paths = Some(
                        inner
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
                    );
                }
            } else if line.starts_with("name:") {
                frontmatter.name = Some(line["name:".len()..].trim().to_string());
            } else if line.starts_with("description:") {
                frontmatter.description = Some(line["description:".len()..].trim().to_string());
            } else if line.starts_with("type:") {
                frontmatter.r#type = Some(line["type:".len()..].trim().to_string());
            }
        }

        ParsedAiMd {
            frontmatter,
            content: body.trim_start().to_string(),
        }
    } else {
        ParsedAiMd {
            frontmatter: AiMdFrontmatter::default(),
            content: content.to_string(),
        }
    }
}

/// Extract @include paths from content
fn extract_include_paths(content: &str, base_path: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let include_regex = regex::Regex::new(r"(?:^|\s)@((?:[^\s\\]|\\ )+)").ok();

    if let Some(re) = include_regex {
        for cap in re.captures_iter(content) {
            if let Some(path_match) = cap.get(1) {
                let mut path = path_match.as_str().to_string();

                // Remove fragment identifiers
                if let Some(idx) = path.find('#') {
                    path.truncate(idx);
                }
                if path.is_empty() {
                    continue;
                }

                // Unescape spaces
                path = path.replace("\\ ", " ");

                // Resolve path
                let resolved = resolve_include_path(&path, base_path);
                if let Some(resolved) = resolved {
                    paths.push(resolved);
                }
            }
        }
    }

    paths
}

/// Resolve include path (supports @path, @./path, @~/path, @/path)
fn resolve_include_path(path: &str, base_dir: &Path) -> Option<PathBuf> {
    let path = path.trim();

    if path.starts_with("@~/") || (path.starts_with("~/") && !path.starts_with("@")) {
        // Home directory path
        let rel = path.trim_start_matches("~/").trim_start_matches("@~/");
        get_home_dir().map(|h| h.join(rel))
    } else if path.starts_with("@/") {
        // Absolute path
        Some(PathBuf::from(path.trim_start_matches("@/")))
    } else if path.starts_with("@./") || path.starts_with("./") {
        // Relative to current file
        let rel = path.trim_start_matches("@./").trim_start_matches("./");
        Some(base_dir.join(rel))
    } else if path.starts_with('@') {
        // @path (relative)
        let rel = path.trim_start_matches('@');
        Some(base_dir.join(rel))
    } else if !path.starts_with('@')
        && !path.starts_with('#')
        && !path.starts_with('%')
        && !path.starts_with('^')
        && !path.starts_with('&')
        && !path.starts_with('*')
        && !path.starts_with('(')
        && !path.starts_with(')')
        && path
            .chars()
            .next()
            .map(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-')
            .unwrap_or(false)
    {
        // Plain path (relative)
        Some(base_dir.join(path))
    } else {
        None
    }
}

/// Process a single AI.md file
pub fn process_ai_md_file(
    path: &Path,
    md_type: AiMdType,
    processed_paths: &mut HashSet<String>,
    depth: usize,
    parent: Option<&str>,
) -> Result<Vec<AiMdFile>, AgentError> {
    // Check depth limit
    if depth >= MAX_INCLUDE_DEPTH {
        return Ok(Vec::new());
    }

    // Normalize and check if already processed
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let normalized = canonical.to_string_lossy().to_lowercase();

    if processed_paths.contains(&normalized) {
        return Ok(Vec::new());
    }

    // Check if file exists
    if !path.exists() {
        return Ok(Vec::new());
    }

    // Read file
    let raw_content = fs::read_to_string(path).map_err(|e| AgentError::Io(e))?;

    if raw_content.trim().is_empty() {
        return Ok(Vec::new());
    }

    processed_paths.insert(normalized);

    let parent_str = parent.map(|s| s.to_string());
    let base_dir = path.parent().unwrap_or(Path::new("."));

    // Parse content
    let parsed = parse_frontmatter(&raw_content);

    // Extract include paths
    let include_paths = extract_include_paths(&parsed.content, base_dir);

    let content_differs = parsed.content != raw_content;

    // Create main file entry
    let mut result = vec![AiMdFile {
        path: path.to_string_lossy().to_string(),
        md_type,
        content: parsed.content.clone(),
        raw_content: if content_differs {
            Some(raw_content)
        } else {
            None
        },
        globs: parsed.frontmatter.paths,
        parent: parent_str.clone(),
        content_differs_from_disk: content_differs,
    }];

    // Process included files
    for include_path in include_paths {
        let included = process_ai_md_file(
            &include_path,
            md_type,
            processed_paths,
            depth + 1,
            Some(&path.to_string_lossy()),
        )?;
        result.extend(included);
    }

    Ok(result)
}

/// Get all AI.md files for a given working directory
pub fn get_ai_md_files(cwd: &Path) -> Result<Vec<AiMdFile>, AgentError> {
    let mut result = Vec::new();
    let mut processed_paths = HashSet::new();

    // 1. Managed memory (/etc/ai-code/AI.md and /etc/ai-code/.ai/rules/*.md)
    if let Some(managed_dir) = PathBuf::from("/etc/ai-code")
        .exists()
        .then_some(PathBuf::from("/etc/ai-code"))
    {
        let managed_md = managed_dir.join(AI_MD_FILENAME);
        if managed_md.exists() {
            let files = process_ai_md_file(
                &managed_md,
                AiMdType::Managed,
                &mut processed_paths,
                0,
                None,
            )?;
            result.extend(files);
        }

        // Managed rules
        if let Some(rules_dir) = get_managed_rules_dir() {
            result.extend(load_rules_from_dir(
                &rules_dir,
                AiMdType::Managed,
                &mut processed_paths,
                0,
            )?);
        }
    }

    // 2. User memory (~/.ai/AI.md and ~/.ai/rules/*.md)
    if let Some(config_dir) = get_config_dir() {
        let user_md = config_dir.join(AI_MD_FILENAME);
        if user_md.exists() {
            let files =
                process_ai_md_file(&user_md, AiMdType::User, &mut processed_paths, 0, None)?;
            result.extend(files);
        }

        // User rules
        if let Some(rules_dir) = get_user_rules_dir() {
            result.extend(load_rules_from_dir(
                &rules_dir,
                AiMdType::User,
                &mut processed_paths,
                0,
            )?);
        }
    }

    // 3. Project memory (traverse from CWD up to root)
    let mut current_dir = cwd.to_path_buf();
    let root = PathBuf::from("/");

    while current_dir != root {
        // AI.md in current directory (localized name)
        let project_md = current_dir.join(AI_MD_FILENAME);
        if project_md.exists() {
            let files = process_ai_md_file(
                &project_md,
                AiMdType::Project,
                &mut processed_paths,
                0,
                None,
            )?;
            result.extend(files);
        }

        // .ai/rules/*.md
        let rules_dir = current_dir.join(PROJECT_RULES_DIR);
        if rules_dir.exists() {
            result.extend(load_rules_from_dir(
                &rules_dir,
                AiMdType::Project,
                &mut processed_paths,
                0,
            )?);
        }

        // Move to parent
        if !current_dir.pop() {
            break;
        }
    }

    // 4. Local memory (AI.local.md - traverse from CWD up to root)
    let mut local_dir = cwd.to_path_buf();
    while local_dir != root {
        // AI.local.md
        let local_md = local_dir.join(AI_MD_LOCAL_FILENAME);
        if local_md.exists() {
            let files =
                process_ai_md_file(&local_md, AiMdType::Local, &mut processed_paths, 0, None)?;
            result.extend(files);
        }

        if !local_dir.pop() {
            break;
        }
    }

    Ok(result)
}

/// Load .ai/rules/*.md files from a directory
fn load_rules_from_dir(
    dir: &Path,
    md_type: AiMdType,
    processed_paths: &mut HashSet<String>,
    depth: usize,
) -> Result<Vec<AiMdFile>, AgentError> {
    let mut result = Vec::new();

    if !dir.exists() || !dir.is_dir() {
        return Ok(result);
    }

    let entries = fs::read_dir(dir).map_err(|e| AgentError::Io(e))?;

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            // Recursively process subdirectories
            result.extend(load_rules_from_dir(&path, md_type, processed_paths, depth)?);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            // Process .md files
            let files = process_ai_md_file(&path, md_type, processed_paths, depth, None)?;
            result.extend(files);
        }
    }

    Ok(result)
}

/// Format AI.md files for inclusion in system prompt
pub fn load_ai_md(cwd: &Path) -> Result<Option<String>, AgentError> {
    let files = get_ai_md_files(cwd)?;

    if files.is_empty() {
        return Ok(None);
    }

    let contents: Vec<AiMdContent> = files
        .into_iter()
        .filter(|f| !f.content.trim().is_empty())
        .map(|f| AiMdContent::new(f.path, f.content, f.md_type))
        .collect();

    if contents.is_empty() {
        return Ok(None);
    }

    let mut result = AI_MD_INSTRUCTION_PROMPT.to_string();
    result.push_str("\n\n");

    for c in contents {
        result.push_str(&format!(
            "Contents of {} {}:\n\n{}\n\n",
            c.path, c.type_description, c.content
        ));
    }

    Ok(Some(result))
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
name: test
paths: ["*.rs", "*.toml"]
---
# Hello World

This is content.
"#;
        let parsed = parse_frontmatter(content);
        assert!(parsed.frontmatter.name.is_some());
        assert_eq!(parsed.frontmatter.name.unwrap(), "test");
        assert!(parsed.frontmatter.paths.is_some());
    }

    #[test]
    fn test_process_file() {
        let temp_dir = TempDir::new().unwrap();
        let md_path = temp_dir.path().join("AI.md");
        fs::write(&md_path, "# Test\n\nContent here.").unwrap();

        let mut processed = HashSet::new();
        let result =
            process_ai_md_file(&md_path, AiMdType::Project, &mut processed, 0, None).unwrap();

        assert!(!result.is_empty());
        assert_eq!(result[0].md_type, AiMdType::Project);
    }

    #[test]
    fn test_get_ai_md_files() {
        let temp_dir = TempDir::new().unwrap();
        let md_path = temp_dir.path().join("AI.md");
        fs::write(&md_path, "# Project AI.md\n\nSome instructions.").unwrap();

        let files = get_ai_md_files(temp_dir.path()).unwrap();
        assert!(!files.is_empty());
    }
}
