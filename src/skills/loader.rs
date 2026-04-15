//! Skill loader - loads skills from SKILL.md files
//!
//! Loads external skills from directories containing SKILL.md files.
//! Supports conditional skills with paths frontmatter for dynamic activation.

use crate::AgentError;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Effort level for skills
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EffortValue {
    Minimum,
    Low,
    Medium,
    High,
    Maximum,
}

impl EffortValue {
    pub fn as_str(&self) -> &str {
        match self {
            EffortValue::Minimum => "minimum",
            EffortValue::Low => "low",
            EffortValue::Medium => "medium",
            EffortValue::High => "high",
            EffortValue::Maximum => "maximum",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "minimum" => Some(EffortValue::Minimum),
            "low" => Some(EffortValue::Low),
            "medium" => Some(EffortValue::Medium),
            "high" => Some(EffortValue::High),
            "maximum" => Some(EffortValue::Maximum),
            _ => None,
        }
    }
}

/// Execution context for skills
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillContext {
    Inline,
    Fork,
}

impl SkillContext {
    pub fn as_str(&self) -> &str {
        match self {
            SkillContext::Inline => "inline",
            SkillContext::Fork => "fork",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "inline" => Some(SkillContext::Inline),
            "fork" => Some(SkillContext::Fork),
            _ => None,
        }
    }
}

/// Hooks settings for skills
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HooksSettings {
    #[serde(rename = "PreToolUse", skip_serializing_if = "Option::is_none")]
    pub pre_tool_use: Option<Vec<HookDefinition>>,
    #[serde(rename = "PostToolUse", skip_serializing_if = "Option::is_none")]
    pub post_tool_use: Option<Vec<HookDefinition>>,
    #[serde(rename = "Notification", skip_serializing_if = "Option::is_none")]
    pub notification: Option<Vec<HookDefinition>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HookDefinition {
    pub command: Option<String>,
    pub timeout: Option<u64>,
    pub matcher: Option<String>,
}

/// Skill metadata parsed from SKILL.md frontmatter
#[derive(Debug, Clone)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub allowed_tools: Option<Vec<String>>,
    pub argument_hint: Option<String>,
    pub arg_names: Option<Vec<String>>,
    pub when_to_use: Option<String>,
    pub user_invocable: Option<bool>,
    /// Conditional paths - skill is activated when these paths are touched
    pub paths: Option<Vec<String>>,
    /// Hooks for this skill
    pub hooks: Option<HooksSettings>,
    /// Effort level
    pub effort: Option<EffortValue>,
    /// Model to use for this skill
    pub model: Option<String>,
    /// Execution context (inline or fork)
    pub context: Option<SkillContext>,
    /// Agent to use for this skill
    pub agent: Option<String>,
}

/// Loaded skill with its metadata and content
#[derive(Debug, Clone)]
pub struct LoadedSkill {
    pub metadata: SkillMetadata,
    pub content: String,
    pub base_dir: String,
}

/// Parse simple frontmatter (key: value format)
fn parse_frontmatter(content: &str) -> (HashMap<String, String>, String) {
    let mut fields = HashMap::new();
    let trimmed = content.trim();

    if !trimmed.starts_with("---") {
        return (fields, content.to_string());
    }

    if let Some(end_pos) = trimmed[3..].find("---") {
        let frontmatter = &trimmed[3..end_pos + 3];
        for line in frontmatter.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                fields.insert(key, value);
            }
        }
        let body = trimmed[end_pos + 6..].trim_start().to_string();
        return (fields, body);
    }

    (fields, content.to_string())
}

/// Load a skill from a directory containing SKILL.md
pub fn load_skill_from_dir(dir_path: &Path) -> Result<LoadedSkill, AgentError> {
    let skill_file = dir_path.join("SKILL.md");
    if !skill_file.exists() {
        return Err(AgentError::Skill(format!(
            "SKILL.md not found in {}",
            dir_path.display()
        )));
    }

    let content = fs::read_to_string(&skill_file).map_err(|e| AgentError::Io(e))?;

    let (fields, body) = parse_frontmatter(&content);

    let name = dir_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let description = fields.get("description").cloned().unwrap_or_default();

    let allowed_tools = fields
        .get("allowed-tools")
        .map(|s| s.split(',').map(|x| x.trim().to_string()).collect());

    let argument_hint = fields.get("argument-hint").cloned();
    let when_to_use = fields.get("when_to_use").cloned();
    let user_invocable = fields.get("user-invocable").and_then(|v| match v.as_str() {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    });

    let arg_names = fields
        .get("arg-names")
        .map(|s| s.split(',').map(|x| x.trim().to_string()).collect());

    let paths = fields
        .get("paths")
        .map(|s| s.split(',').map(|x| x.trim().to_string()).collect());

    let effort = fields.get("effort").and_then(|s| EffortValue::from_str(s));

    let context = fields
        .get("context")
        .and_then(|s| SkillContext::from_str(s));

    let model = fields.get("model").cloned();
    let agent = fields.get("agent").cloned();

    // Parse hooks (simplified - just check if hooks field exists)
    let hooks = if fields.contains_key("hooks") {
        Some(HooksSettings::default())
    } else {
        None
    };

    let metadata = SkillMetadata {
        name,
        description,
        allowed_tools,
        argument_hint,
        arg_names,
        when_to_use,
        user_invocable,
        paths,
        hooks,
        effort,
        model,
        context,
        agent,
    };

    Ok(LoadedSkill {
        metadata,
        content: body,
        base_dir: dir_path.to_string_lossy().to_string(),
    })
}

/// Load all skills from a skills directory (skill-name/SKILL.md format)
pub fn load_skills_from_dir(base_path: &Path) -> Result<Vec<LoadedSkill>, AgentError> {
    if !base_path.exists() {
        return Ok(Vec::new());
    }

    let mut skills = Vec::new();

    let entries = fs::read_dir(base_path).map_err(|e| AgentError::Io(e))?;

    for entry in entries {
        let entry = entry.map_err(|e| AgentError::Io(e))?;
        let path = entry.path();

        if path.is_dir() {
            if let Ok(skill) = load_skill_from_dir(&path) {
                skills.push(skill);
            }
        }
    }

    Ok(skills)
}

/// Check if a path matches any of the given glob patterns
/// Supports patterns like "*.rs", "src/**/*.ts", "**/test*.py"
fn path_matches_patterns(path: &str, patterns: &[String]) -> bool {
    for pattern in patterns {
        if glob_match(pattern, path) {
            return true;
        }
    }
    false
}

/// Simple glob matching function
/// Supports: * (any characters), ** (any path segments), ? (single character)
fn glob_match(pattern: &str, path: &str) -> bool {
    // Convert glob to regex and match
    let regex_pattern = glob_to_regex(pattern);
    if let Ok(re) = regex::Regex::new(&regex_pattern) {
        re.is_match(path)
    } else {
        false
    }
}

/// Convert glob pattern to regex pattern
fn glob_to_regex(pattern: &str) -> String {
    let mut regex = String::from("^");
    let mut chars = pattern.chars().peekable();
    let mut prev_was_doublestar = false;

    while let Some(c) = chars.next() {
        match c {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                    prev_was_doublestar = true;
                    // ** matches zero or more path segments (any characters including /)
                    regex.push_str("(.*/)?");
                } else {
                    prev_was_doublestar = false;
                    // * matches any characters except /
                    regex.push_str("[^/]*");
                }
            }
            '/' if prev_was_doublestar => {
                // After **, the slash is already included in the (.*/)? pattern,
                // so we skip it here
                prev_was_doublestar = false;
            }
            '?' => regex.push('.'),
            '[' => {
                // Character class - pass through until ]
                regex.push(c);
                while let Some(&next) = chars.peek() {
                    regex.push(next);
                    chars.next();
                    if next == ']' {
                        break;
                    }
                }
            }
            '.' | '+' | '^' | '$' | '(' | ')' | '|' | '\\' => {
                regex.push('\\');
                regex.push(c);
            }
            _ => regex.push(c),
        }
    }

    regex.push('$');
    regex
}

/// Discover skill directories that match the given file paths
/// This implements discoverSkillDirsForPaths from TypeScript
pub fn discover_skill_dirs_for_paths(
    skills_dir: &Path,
    touched_paths: &[String],
) -> Result<Vec<PathBuf>, AgentError> {
    if !skills_dir.exists() {
        return Ok(Vec::new());
    }

    let mut matching_dirs = Vec::new();

    let entries = fs::read_dir(skills_dir).map_err(|e| AgentError::Io(e))?;

    for entry in entries {
        let entry = entry.map_err(|e| AgentError::Io(e))?;
        let path = entry.path();

        if path.is_dir() {
            // Load the skill to check its paths
            if let Ok(skill) = load_skill_from_dir(&path) {
                if let Some(skill_paths) = &skill.metadata.paths {
                    // Check if any touched path matches the skill's paths
                    for touched in touched_paths {
                        if path_matches_patterns(touched, skill_paths) {
                            matching_dirs.push(path.clone());
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(matching_dirs)
}

/// Activate conditional skills for given file paths
/// Returns skills that should be active based on the touched files
/// This implements activateConditionalSkillsForPaths from TypeScript
pub fn activate_conditional_skills_for_paths(
    skills_dir: &Path,
    touched_paths: &[String],
) -> Result<Vec<LoadedSkill>, AgentError> {
    if !skills_dir.exists() || touched_paths.is_empty() {
        return Ok(Vec::new());
    }

    let mut active_skills = Vec::new();

    let entries = fs::read_dir(skills_dir).map_err(|e| AgentError::Io(e))?;

    for entry in entries {
        let entry = entry.map_err(|e| AgentError::Io(e))?;
        let path = entry.path();

        if path.is_dir() {
            if let Ok(skill) = load_skill_from_dir(&path) {
                if let Some(skill_paths) = &skill.metadata.paths {
                    // Check if any touched path matches the skill's paths
                    for touched in touched_paths {
                        if path_matches_patterns(touched, skill_paths) {
                            active_skills.push(skill);
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(active_skills)
}

/// Get all conditional skills (skills with paths frontmatter)
pub fn get_conditional_skills(skills_dir: &Path) -> Result<Vec<LoadedSkill>, AgentError> {
    if !skills_dir.exists() {
        return Ok(Vec::new());
    }

    let mut conditional_skills = Vec::new();

    let entries = fs::read_dir(skills_dir).map_err(|e| AgentError::Io(e))?;

    for entry in entries {
        let entry = entry.map_err(|e| AgentError::Io(e))?;
        let path = entry.path();

        if path.is_dir() {
            if let Ok(skill) = load_skill_from_dir(&path) {
                if skill.metadata.paths.is_some() {
                    conditional_skills.push(skill);
                }
            }
        }
    }

    Ok(conditional_skills)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match_simple() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(glob_match("*.rs", "lib.rs"));
        assert!(!glob_match("*.rs", "main.py"));
    }

    #[test]
    fn test_glob_match_double_star() {
        assert!(glob_match("src/**/*.ts", "src/foo.ts"));
        assert!(glob_match("src/**/*.ts", "src/bar/baz.ts"));
        assert!(!glob_match("src/**/*.ts", "tests/foo.ts"));
    }

    #[test]
    fn test_glob_match_question() {
        assert!(glob_match("file?.txt", "file1.txt"));
        assert!(glob_match("file?.txt", "filea.txt"));
        assert!(!glob_match("file?.txt", "file12.txt"));
    }

    #[test]
    fn test_effort_value() {
        assert_eq!(EffortValue::as_str(&EffortValue::High), "high");
        assert_eq!(EffortValue::from_str("medium"), Some(EffortValue::Medium));
        assert_eq!(EffortValue::from_str("invalid"), None);
    }

    #[test]
    fn test_skill_context() {
        assert_eq!(SkillContext::as_str(&SkillContext::Fork), "fork");
        assert_eq!(SkillContext::from_str("inline"), Some(SkillContext::Inline));
        assert_eq!(SkillContext::from_str("invalid"), None);
    }
}
