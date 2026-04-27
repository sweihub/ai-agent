//! Skill loader - loads skills from SKILL.md files
//!
//! Loads external skills from directories containing SKILL.md files.
//! Supports conditional skills with paths frontmatter for dynamic activation.

use crate::AgentError;
use crate::utils::git::gitignore::is_path_gitignored;
use crate::utils::memoize::memoize_with_lru;
use once_cell::sync::Lazy;
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

/// Hooks settings for skills — uses HashMap format matching register_skill_hooks.
/// Re-exported from the hooks module for use in skill frontmatter parsing.
pub use crate::utils::hooks::register_skill_hooks::HooksSettings;
pub use crate::utils::hooks::register_skill_hooks::HookMatcher;

/// Skill metadata parsed from SKILL.md frontmatter
#[derive(Debug, Clone)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    /// Display name parsed from frontmatter `name` field (TS: displayName)
    pub display_name: Option<String>,
    /// Version parsed from frontmatter `version` field
    pub version: Option<String>,
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
    /// Shell for embedded command execution (bash or powershell)
    pub shell: Option<String>,
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

/// Substitute `${CLAUDE_SKILL_DIR}` and `${CLAUDE_SESSION_ID}` environment
/// variables in skill content strings.
///
/// Matches TypeScript behavior at loadSkillsDir.ts lines 362-368 where
/// `getPromptForCommand` replaces `${CLAUDE_SKILL_DIR}` with the skill's
/// base directory and `${CLAUDE_SESSION_ID}` with the current session ID.
pub fn substitute_env_vars_in_skill(content: &str, base_dir: &str) -> String {
    let session_id = crate::bootstrap::state::get_session_id();
    // Normalize backslashes to forward slashes on Windows so shell commands
    // don't treat them as escapes.
    #[cfg(windows)]
    let normalised_base_dir = base_dir.replace('\\', "/");
    #[cfg(not(windows))]
    let normalised_base_dir = base_dir.to_string();

    content
        .replace("${CLAUDE_SKILL_DIR}", &normalised_base_dir)
        .replace("${CLAUDE_SESSION_ID}", &session_id)
}

/// Estimate token count for a skill based on frontmatter only
/// (name, description, when_to_use) since full content is only loaded on invocation.
///
/// Matches TypeScript `estimateSkillFrontmatterTokens` at loadSkillsDir.ts lines 97-105.
pub fn estimate_skill_frontmatter_tokens(metadata: &SkillMetadata) -> usize {
    let parts: Vec<&str> = vec![
        Some(metadata.name.as_str()),
        Some(metadata.description.as_str()),
        metadata.when_to_use.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect();
    let frontmatter_text = parts.join(" ");
    crate::services::token_estimation::rough_token_count_estimation(&frontmatter_text, 4.0)
}

/// Load a skill from a directory containing SKILL.md
pub fn parse_hooks_from_frontmatter(content: &str) -> Option<HooksSettings> {
    let trimmed = content.trim();

    // Extract frontmatter block between --- delimiters
    if !trimmed.starts_with("---") {
        return None;
    }
    let frontmatter_end = trimmed[3..].find("---")?;
    let frontmatter = &trimmed[3..frontmatter_end + 3];

    // Parse the entire frontmatter as YAML to get complex structures
    let yaml_value: serde_yaml::Value = match serde_yaml::from_str(frontmatter) {
        Ok(v) => v,
        Err(e) => {
            log::debug!("Failed to parse SKILL.md frontmatter as YAML: {}", e);
            return None;
        }
    };

    // Extract the 'hooks' field as a serde_yaml::Value
    let hooks_value = yaml_value.get("hooks")?;

    // Convert serde_yaml::Value to serde_json::Value for deserialization
    // into HooksSettings (which uses serde_json::Value in HookMatcher.hooks)
    let hooks_json = yaml_to_json(hooks_value.clone())?;

    // Deserialize into HooksSettings
    // The HooksSettings uses #[serde(flatten)] with HashMap<String, Vec<HookMatcher>>
    let hooks: HooksSettings = match serde_json::from_value(hooks_json) {
        Ok(h) => h,
        Err(e) => {
            log::debug!("Failed to deserialize hooks from YAML: {}", e);
            return None;
        }
    };

    if hooks.events.is_empty() {
        return None;
    }

    Some(hooks)
}

/// Convert a serde_yaml::Value to serde_json::Value
fn yaml_to_json(value: serde_yaml::Value) -> Option<serde_json::Value> {
    match value {
        serde_yaml::Value::Null => Some(serde_json::Value::Null),
        serde_yaml::Value::Bool(b) => Some(serde_json::Value::Bool(b)),
        serde_yaml::Value::Number(n) => {
            if let Some(v) = n.as_i64() {
                Some(serde_json::Value::Number(v.into()))
            } else if let Some(v) = n.as_u64() {
                Some(serde_json::Value::Number(v.into()))
            } else if let Some(v) = n.as_f64() {
                serde_json::Number::from_f64(v).map(serde_json::Value::Number)
            } else {
                None
            }
        }
        serde_yaml::Value::String(s) => Some(serde_json::Value::String(s)),
        serde_yaml::Value::Sequence(seq) => {
            let arr = seq.into_iter().filter_map(|v| yaml_to_json(v)).collect();
            Some(serde_json::Value::Array(arr))
        }
        serde_yaml::Value::Mapping(map) => {
            let obj = map
                .into_iter()
                .filter_map(|(k, v)| {
                    let key = match &k {
                        serde_yaml::Value::String(s) => s.clone(),
                        serde_yaml::Value::Number(n) => n.to_string(),
                        serde_yaml::Value::Bool(b) => b.to_string(),
                        _ => return None,
                    };
                    yaml_to_json(v).map(|val| (key, val))
                })
                .collect();
            Some(serde_json::Value::Object(obj))
        }
        serde_yaml::Value::Tagged(ref tagged) => {
            // Handle tagged YAML values by extracting the value
            yaml_to_json(tagged.value.clone())
        }
    }
}
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

    let display_name = fields.get("name").cloned();
    let version = fields.get("version").cloned();

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
    let shell = fields.get("shell").cloned();

    // Parse hooks from YAML frontmatter block
    let hooks = if fields.contains_key("hooks") {
        parse_hooks_from_frontmatter(&content)
    } else {
        None
    };

    let metadata = SkillMetadata {
        name,
        description,
        display_name,
        version,
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
        shell,
    };

    Ok(LoadedSkill {
        metadata,
        content: body,
        base_dir: dir_path.to_string_lossy().to_string(),
    })
}

/// Load all skills from a skills directory (skill-name/SKILL.md format)
pub fn load_skills_from_dir(base_path: &Path, cwd: &Path) -> Result<Vec<LoadedSkill>, AgentError> {
    if !base_path.exists() {
        return Ok(Vec::new());
    }

    let mut skills = Vec::new();

    let entries = fs::read_dir(base_path).map_err(|e| AgentError::Io(e))?;

    for entry in entries {
        let entry = entry.map_err(|e| AgentError::Io(e))?;
        let path = entry.path();

        if path.is_dir() {
            // Skip skill directories that are gitignored
            if is_path_gitignored(&path, cwd) {
                log::debug!(
                    "[skills] Skipped gitignored skill dir: {}",
                    path.display()
                );
                continue;
            }

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

/// Source of a loaded skill.
#[derive(Debug, Clone, PartialEq)]
pub enum SkillSource {
    Bundled,
    User,
    Project,
    Plugin,
}

impl std::fmt::Display for SkillSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillSource::Bundled => write!(f, "bundled"),
            SkillSource::User => write!(f, "user"),
            SkillSource::Project => write!(f, "project"),
            SkillSource::Plugin => write!(f, "plugin"),
        }
    }
}

/// Unified skill entry from any source.
#[derive(Debug, Clone)]
pub struct UnifiedSkill {
    pub name: String,
    pub description: String,
    pub source: SkillSource,
    pub content: String,
    pub paths: Option<Vec<String>>,
    pub user_invocable: Option<bool>,
    pub hooks: Option<HooksSettings>,
}

/// Resolve the user skills directory (~/.ai/skills).
/// Returns None if the home directory cannot be determined.
pub fn get_user_skills_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".ai").join("skills"))
}

/// Resolve the project skills directory (<cwd>/.ai/skills).
pub fn get_project_skills_dir(cwd: &str) -> PathBuf {
    Path::new(cwd).join(".ai").join("skills")
}

/// Load skills from all sources: bundled, user (~/.ai/skills), project (<cwd>/.ai/skills).
///
/// Skills are deduplicated by name. Later sources override earlier ones:
/// Project > User > Bundled.
///
/// Returns a Vec of UnifiedSkill sorted by source priority (project first).
pub fn load_all_skills(cwd: &str) -> Result<Vec<UnifiedSkill>, AgentError> {
    let mut skill_map: HashMap<String, UnifiedSkill> = HashMap::new();

    // 1. Load bundled skills
    let bundled_skills = crate::skills::bundled_skills::get_bundled_skills();
    for bs in bundled_skills {
        skill_map.insert(
            bs.name.clone(),
            UnifiedSkill {
                name: bs.name,
                description: bs.description,
                source: SkillSource::Bundled,
                content: String::new(),
                paths: None,
                user_invocable: Some(bs.user_invocable),
                hooks: None,
            },
        );
    }

    // 2. Load user skills (~/.ai/skills)
    if let Some(user_dir) = get_user_skills_dir() {
        if let Ok(user_skills) = load_skills_from_dir(&user_dir, Path::new(cwd)) {
            for us in user_skills {
                skill_map.insert(
                    us.metadata.name.clone(),
                    UnifiedSkill {
                        name: us.metadata.name,
                        description: us.metadata.description,
                        source: SkillSource::User,
                        content: us.content,
                        paths: us.metadata.paths,
                        user_invocable: us.metadata.user_invocable,
                        hooks: us.metadata.hooks,
                    },
                );
            }
        }
    }

    // 3. Load project skills (<cwd>/.ai/skills)
    let project_dir = get_project_skills_dir(cwd);
    if let Ok(project_skills) = load_skills_from_dir(&project_dir, Path::new(cwd)) {
        for ps in project_skills {
            skill_map.insert(
                ps.metadata.name.clone(),
                UnifiedSkill {
                    name: ps.metadata.name,
                    description: ps.metadata.description,
                    source: SkillSource::Project,
                    content: ps.content,
                    paths: ps.metadata.paths,
                    user_invocable: ps.metadata.user_invocable,
                    hooks: ps.metadata.hooks,
                },
            );
        }
    }

    let mut all_skills: Vec<UnifiedSkill> = skill_map.into_values().collect();

    // Sort: project first, then user, then bundled (alphabetical within)
    all_skills.sort_by(|a, b| {
        let source_order = |s: &SkillSource| -> u8 {
            match s {
                SkillSource::Project => 0,
                SkillSource::User => 1,
                SkillSource::Bundled => 2,
                SkillSource::Plugin => 3,
            }
        };
        source_order(&a.source)
            .cmp(&source_order(&b.source))
            .then_with(|| a.name.cmp(&b.name))
    });

    Ok(all_skills)
}

// ---------------------------------------------------------------------------
// Memoized (LRU-cached) skill loading
// ---------------------------------------------------------------------------
// Matches TS `memoize(getSkillDirCommands)` from loadSkillsDir.ts.
//
// `AgentError` does not derive Clone, so the cached variants return
// `Result<..., String>` instead.  The original unmemoized functions remain
// public and return `Result<..., AgentError>`.

/// Key for `load_skills_from_dir` memoization.
/// Bundles the two string arguments into a single cache key.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
#[allow(dead_code)]
pub struct SkillsDirKey {
    pub base_path: String,
    pub cwd: String,
}

/// Memoized version of `load_all_skills`, keyed by cwd.
/// Matches TS `memoize(getSkillDirCommands)` from loadSkillsDir.ts
#[allow(dead_code)]
static LOAD_ALL_SKILLS_MEMO: Lazy<
    crate::utils::memoize::LruMemoized<String, String, Result<Vec<UnifiedSkill>, String>>,
> = Lazy::new(|| {
    memoize_with_lru(
        |cwd: String| load_all_skills(&cwd).map_err(|e| e.to_string()),
        |cwd: &String| cwd.clone(),
        50, // Max 50 cached cwd entries
    )
});

/// Memoized version of `load_all_skills(cwd)`.
/// Caches results per working directory to avoid re-scanning the filesystem
/// on every turn.
#[allow(dead_code)]
pub fn load_all_skills_cached(cwd: &str) -> Result<Vec<UnifiedSkill>, String> {
    LOAD_ALL_SKILLS_MEMO.call(cwd.to_string())
}

/// Memoized version of `load_skills_from_dir`.
/// Keyed by (base_path, cwd) tuple to avoid redundant filesystem scans.
#[allow(dead_code)]
static LOAD_SKILLS_FROM_DIR_MEMO: Lazy<
    crate::utils::memoize::LruMemoized<
        SkillsDirKey,
        SkillsDirKey,
        Result<Vec<LoadedSkill>, String>,
    >,
> = Lazy::new(|| {
    memoize_with_lru(
        |key: SkillsDirKey| {
            load_skills_from_dir(Path::new(&key.base_path), Path::new(&key.cwd))
                .map_err(|e| e.to_string())
        },
        |key: &SkillsDirKey| key.clone(),
        50, // Max 50 cached entries
    )
});

/// Memoized version of `load_skills_from_dir(base_path, cwd)`.
/// Caches results per directory path to avoid re-scanning the filesystem.
#[allow(dead_code)]
pub fn load_skills_from_dir_cached(
    base_path: &str,
    cwd: &str,
) -> Result<Vec<LoadedSkill>, String> {
    let key = SkillsDirKey {
        base_path: base_path.to_string(),
        cwd: cwd.to_string(),
    };
    LOAD_SKILLS_FROM_DIR_MEMO.call(key)
}

// ============================================================================
// MCP skill builders registration
//
// Registers the two loadSkillsDir functions that MCP skill discovery needs.
// This write-once registry breaks the circular dependency between MCP skill
// discovery and the skill loader.
// ============================================================================

fn create_skill_command_for_mcp(
    params: &crate::skills::mcp_skill_builders::LoadedSkillCommandParams,
) -> crate::skills::bundled_skills::BundledSkillDefinition {
    use crate::skills::bundled_skills::{BundledSkillDefinition, ContentBlock, SkillContext};
    use crate::AgentError;

    let markdown_content = params.markdown_content.clone();
    let base_dir = params.base_dir.clone();
    let argument_names = params.argument_names.clone();

    crate::skills::bundled_skills::BundledSkillDefinition {
        name: params.skill_name.clone(),
        description: params.description.clone(),
        aliases: params
            .display_name
            .as_ref()
            .map(|d| vec![d.clone()]),
        when_to_use: params.when_to_use.clone(),
        argument_hint: params.argument_hint.clone(),
        allowed_tools: params.allowed_tools.clone(),
        model: params.model.clone(),
        disable_model_invocation: Some(params.disable_model_invocation),
        user_invocable: Some(params.user_invocable),
        is_enabled: None,
        hooks: None,
        context: None,
        agent: None,
        files: None,
        get_prompt_for_command: std::sync::Arc::new(move |args: &str, _ctx: &SkillContext| {
            let mut content = markdown_content.clone();

            // Substitute CLAUDE_SKILL_DIR
            if !base_dir.is_empty() {
                let skill_dir = base_dir.replace('\\', "/");
                content = content.replace("${CLAUDE_SKILL_DIR}", &skill_dir);
            }

            // Substitute CLAUDE_SESSION_ID
            content = content.replace(
                "${CLAUDE_SESSION_ID}",
                &std::env::var("AI_SESSION_ID").unwrap_or_default(),
            );

            // Substitute arguments
            if let Some(ref arg_names) = argument_names {
                for (i, name) in arg_names.iter().enumerate() {
                    let placeholder = format!("${}", name);
                    let args_vec: Vec<&str> = args.split_whitespace().collect();
                    if let Some(val) = args_vec.get(i) {
                        content = content.replace(&placeholder, val);
                    }
                }
            }

            // Prepend base dir info
            let final_content = if !base_dir.is_empty() {
                format!("Base directory for this skill: {}\n\n{}", base_dir, content)
            } else {
                content
            };

            Ok(vec![ContentBlock::Text {
                text: final_content,
            }])
        }),
    }
}

fn parse_skill_frontmatter_fields_for_mcp(
    content: &str,
) -> crate::skills::mcp_skill_builders::SkillFrontmatterFields {
    crate::skills::mcp_skill_builders::default_parse_skill_frontmatter_fields(content)
}

/// Register MCP skill builders at module init.
/// This static is initialized when first accessed, which occurs at startup
/// when the loader module is imported.
static MCP_SKILL_BUILDERS_INIT: once_cell::sync::OnceCell<()> = once_cell::sync::OnceCell::new();

fn init_mcp_skill_builders() {
    let _ = MCP_SKILL_BUILDERS_INIT.get_or_init(|| {
        use crate::skills::mcp_skill_builders::{register_mcp_skill_builders, LoadedSkillCommandParams, SkillFrontmatterFields};

        let create_fn: Box<dyn Fn(&LoadedSkillCommandParams) -> crate::skills::bundled_skills::BundledSkillDefinition + Send + Sync> =
            Box::new(create_skill_command_for_mcp);
        let parse_fn: Box<dyn Fn(&str) -> SkillFrontmatterFields + Send + Sync> =
            Box::new(parse_skill_frontmatter_fields_for_mcp);

        register_mcp_skill_builders(create_fn, parse_fn);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::Hasher;
    use std::io::Write;

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

    #[test]
    fn test_get_user_skills_dir() {
        let dir = get_user_skills_dir();
        // May be None if home dir not available in test env
        if let Some(d) = dir {
            assert!(d.to_string_lossy().ends_with(".ai/skills"));
        }
    }

    #[test]
    fn test_get_project_skills_dir() {
        let dir = get_project_skills_dir("/my/project");
        assert_eq!(dir, PathBuf::from("/my/project/.ai/skills"));
    }

    #[test]
    fn test_load_all_skills_no_skills() {
        // With empty cwd and no skills registered, should return empty
        let result = load_all_skills("/tmp/nonexistent_dir_12345");
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_all_skills_from_temp_dir() {
        use std::io::Write;
        let temp = tempfile::tempdir().unwrap();
        let cwd = temp.path().to_string_lossy().to_string();

        // Create a project skill
        let skill_dir = temp.path().join(".ai").join("skills").join("test-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        let mut skill_file = std::fs::File::create(skill_dir.join("SKILL.md")).unwrap();
        writeln!(skill_file, "---").unwrap();
        writeln!(skill_file, "description: Test skill from project").unwrap();
        writeln!(skill_file, "---").unwrap();
        writeln!(skill_file, "Test skill content").unwrap();

        let result = load_all_skills(&cwd).unwrap();
        let test_skill = result.iter().find(|s| s.name == "test-skill");
        assert!(test_skill.is_some());
        assert_eq!(test_skill.unwrap().source, SkillSource::Project);
    }

    #[test]
    fn test_skill_source_display() {
        assert_eq!(format!("{}", SkillSource::Bundled), "bundled");
        assert_eq!(format!("{}", SkillSource::User), "user");
        assert_eq!(format!("{}", SkillSource::Project), "project");
        assert_eq!(format!("{}", SkillSource::Plugin), "plugin");
    }

    #[test]
    fn test_unified_skill_creation() {
        let skill = UnifiedSkill {
            name: "test".to_string(),
            description: "A test skill".to_string(),
            source: SkillSource::Project,
            content: "content".to_string(),
            paths: Some(vec!["*.rs".to_string()]),
            user_invocable: Some(true),
            hooks: None,
        };
        assert_eq!(skill.name, "test");
        assert!(skill.user_invocable.unwrap());
    }

    #[test]
    fn test_parse_hooks_from_frontmatter_valid() {
        let content = r#"---
name: test-skill
description: A test skill with hooks
hooks:
  Stop:
    - matcher: ""
      hooks:
        - type: command
          command: "echo skill-stop"
  PreToolUse:
    - matcher: "Bash"
      hooks:
        - type: command
          command: "echo pre-bash"
          timeout: 10
---
Skill content here
"#;
        let hooks = parse_hooks_from_frontmatter(content);
        assert!(hooks.is_some());
        let hooks = hooks.unwrap();

        // Should have Stop and PreToolUse events
        assert!(hooks.events.contains_key("Stop"));
        assert!(hooks.events.contains_key("PreToolUse"));
        assert!(!hooks.events.is_empty());
    }

    #[test]
    fn test_parse_hooks_from_frontmatter_no_hooks() {
        let content = r#"---
name: test-skill
description: A test skill without hooks
---
Skill content here
"#;
        let hooks = parse_hooks_from_frontmatter(content);
        assert!(hooks.is_none());
    }

    #[test]
    fn test_parse_hooks_from_frontmatter_no_frontmatter() {
        let content = "Just plain text content";
        let hooks = parse_hooks_from_frontmatter(content);
        assert!(hooks.is_none());
    }

    #[test]
    fn test_parse_hooks_from_frontmatter_empty_hooks() {
        let content = r#"---
name: test-skill
hooks: {}
---
Content
"#;
        let hooks = parse_hooks_from_frontmatter(content);
        // Empty hooks map should return None
        assert!(hooks.is_none());
    }

    #[test]
    fn test_yaml_to_json_basic_types() {
        let yaml_str = r#"
null_val: null
bool_val: true
int_val: 42
str_val: hello
list_val:
  - a
  - b
map_val:
  key: value
"#;
        let yaml_val: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let json = yaml_to_json(yaml_val).unwrap();

        assert_eq!(json["null_val"], serde_json::Value::Null);
        assert_eq!(json["bool_val"], true);
        assert_eq!(json["int_val"], 42);
        assert_eq!(json["str_val"], "hello");
        assert!(json["list_val"].is_array());
        assert_eq!(json["list_val"][0], "a");
        assert_eq!(json["map_val"]["key"], "value");
    }

    #[test]
    fn test_load_skill_with_hooks() {
        use std::io::Write;
        let temp = tempfile::tempdir().unwrap();
        let skill_dir = temp.path().join("hook-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();

        let mut skill_file = std::fs::File::create(skill_dir.join("SKILL.md")).unwrap();
        writeln!(skill_file, "---").unwrap();
        writeln!(skill_file, "description: Skill with hooks").unwrap();
        writeln!(skill_file, "hooks:").unwrap();
        writeln!(skill_file, "  Stop:").unwrap();
        writeln!(skill_file, "    - matcher: \"\"").unwrap();
        writeln!(skill_file, "      hooks:").unwrap();
        writeln!(skill_file, "        - type: command").unwrap();
        writeln!(skill_file, "          command: echo done").unwrap();
        writeln!(skill_file, "---").unwrap();
        writeln!(skill_file, "Skill body").unwrap();

        let skill = load_skill_from_dir(&skill_dir).unwrap();
        assert_eq!(skill.metadata.name, "hook-skill");
        assert!(skill.metadata.hooks.is_some());
        let hooks = skill.metadata.hooks.unwrap();
        assert!(hooks.events.contains_key("Stop"));
    }

    #[test]
    fn test_load_skill_without_hooks() {
        use std::io::Write;
        let temp = tempfile::tempdir().unwrap();
        let skill_dir = temp.path().join("no-hook-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();

        let mut skill_file = std::fs::File::create(skill_dir.join("SKILL.md")).unwrap();
        writeln!(skill_file, "---").unwrap();
        writeln!(skill_file, "description: Skill without hooks").unwrap();
        writeln!(skill_file, "---").unwrap();
        writeln!(skill_file, "Skill body").unwrap();

        let skill = load_skill_from_dir(&skill_dir).unwrap();
        assert!(skill.metadata.hooks.is_none());
    }

    #[test]
    fn test_load_skills_from_dir_skips_gitignored() {
        use std::io::Write;

        let temp = tempfile::tempdir().unwrap();
        let repo_root = temp.path();

        // Initialize a git repo
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(repo_root)
            .output()
            .expect("git init failed");

        // Create .gitignore that ignores "ignored-skill"
        let gitignore_path = repo_root.join(".gitignore");
        let mut gitignore_file = std::fs::File::create(&gitignore_path).unwrap();
        writeln!(gitignore_file, "ignored-skill/").unwrap();
        drop(gitignore_file);

        // Create skills directory
        let skills_dir = repo_root.join(".ai").join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        // Create a normal skill (should be loaded)
        let normal_skill_dir = skills_dir.join("normal-skill");
        std::fs::create_dir_all(&normal_skill_dir).unwrap();
        let mut normal_skill_file =
            std::fs::File::create(normal_skill_dir.join("SKILL.md")).unwrap();
        writeln!(normal_skill_file, "---").unwrap();
        writeln!(normal_skill_file, "description: Normal skill").unwrap();
        writeln!(normal_skill_file, "---").unwrap();
        writeln!(normal_skill_file, "Normal skill content").unwrap();
        drop(normal_skill_file);

        // Create a gitignored skill (should be skipped)
        let ignored_skill_dir = skills_dir.join("ignored-skill");
        std::fs::create_dir_all(&ignored_skill_dir).unwrap();
        let mut ignored_skill_file =
            std::fs::File::create(ignored_skill_dir.join("SKILL.md")).unwrap();
        writeln!(ignored_skill_file, "---").unwrap();
        writeln!(ignored_skill_file, "description: Ignored skill").unwrap();
        writeln!(ignored_skill_file, "---").unwrap();
        writeln!(ignored_skill_file, "Ignored skill content").unwrap();
        drop(ignored_skill_file);

        // Load skills - pass repo_root as cwd for git check-ignore context
        let skills =
            load_skills_from_dir(&skills_dir, repo_root).expect("failed to load skills");

        // Should have exactly 1 skill (the normal one), not the ignored one
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].metadata.name, "normal-skill");
    }

    // -----------------------------------------------------------------------
    // Memoization tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_load_all_skills_memoization() {
        use std::io::Write;

        // Clear shared cache to isolate this test from others
        LOAD_ALL_SKILLS_MEMO.clear();

        let temp = tempfile::tempdir().unwrap();
        let cwd = temp.path().to_string_lossy().to_string();

        // Create a project skill
        let skill_dir = temp.path().join(".ai").join("skills").join("memo-test");
        std::fs::create_dir_all(&skill_dir).unwrap();
        let mut skill_file = std::fs::File::create(skill_dir.join("SKILL.md")).unwrap();
        writeln!(skill_file, "---").unwrap();
        writeln!(skill_file, "description: Memo test skill").unwrap();
        writeln!(skill_file, "---").unwrap();
        writeln!(skill_file, "Body").unwrap();
        drop(skill_file);

        // First call - populates the cache
        let skills1 = load_all_skills_cached(&cwd).unwrap();
        let has_skill1 = skills1.iter().any(|s| s.name == "memo-test");
        assert!(has_skill1);

        // Second call with the same cwd - should hit the cache
        let skills2 = load_all_skills_cached(&cwd).unwrap();
        let has_skill2 = skills2.iter().any(|s| s.name == "memo-test");
        assert!(has_skill2);

        // Results should be identical
        assert_eq!(skills1.len(), skills2.len());
    }

    #[test]
    fn test_load_skills_from_dir_memoization() {
        use std::io::Write;

        // Clear shared cache to isolate this test from others
        LOAD_SKILLS_FROM_DIR_MEMO.clear();

        let temp = tempfile::tempdir().unwrap();
        let base_dir = temp.path().join(".ai").join("skills");
        std::fs::create_dir_all(&base_dir).unwrap();

        // Create a skill inside the directory
        let skill_dir = base_dir.join("cached-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        let mut sf = std::fs::File::create(skill_dir.join("SKILL.md")).unwrap();
        writeln!(sf, "---").unwrap();
        writeln!(sf, "description: Cached skill").unwrap();
        writeln!(sf, "---").unwrap();
        writeln!(sf, "Body").unwrap();
        drop(sf);

        let base_str = base_dir.to_string_lossy().to_string();
        let cwd_str = temp.path().to_string_lossy().to_string();

        // First call
        let skills1 = load_skills_from_dir_cached(&base_str, &cwd_str).unwrap();
        assert_eq!(skills1.len(), 1);
        assert_eq!(skills1[0].metadata.name, "cached-skill");

        // Second call with same args - should hit cache
        let skills2 = load_skills_from_dir_cached(&base_str, &cwd_str).unwrap();
        assert_eq!(skills2.len(), 1);
        assert_eq!(skills2[0].metadata.name, "cached-skill");
    }

    #[test]
    fn test_lru_memoization_eviction() {
        use std::io::Write;

        // Clear shared cache
        LOAD_ALL_SKILLS_MEMO.clear();

        // Create more than 50 temp directories to exercise LRU eviction.
        // Each directory gets a unique skill name so results are distinguishable.
        let temps: Vec<tempfile::TempDir> = (0..55)
            .map(|i| {
                let temp = tempfile::tempdir().unwrap();
                let skill_dir = temp.path().join(".ai").join("skills").join(format!("skill-{i}"));
                std::fs::create_dir_all(&skill_dir).unwrap();
                let mut sf = std::fs::File::create(skill_dir.join("SKILL.md")).unwrap();
                writeln!(sf, "---").unwrap();
                writeln!(sf, "description: Skill {i}").unwrap();
                writeln!(sf, "---").unwrap();
                writeln!(sf, "Body {i}").unwrap();
                drop(sf);
                temp
            })
            .collect();

        // Load all 55 directories through the cached function (max 50 cached).
        let cwd_vec: Vec<String> = temps
            .iter()
            .map(|t| t.path().to_string_lossy().to_string())
            .collect();

        for cwd in &cwd_vec {
            let _ = load_all_skills_cached(cwd);
        }

        // The cache can hold at most 50 entries; the first 5 should have been evicted.
        // Call one of the earliest entries - it will re-compute (cache miss, not an error).
        let first_cwd = &cwd_vec[0];
        let _ = load_all_skills_cached(first_cwd);

        // Call a middle entry that should still be cached (index 30 is within
        // the 50-entry window after eviction of entries 0..5).
        let middle_cwd = &cwd_vec[30];
        let skills = load_all_skills_cached(middle_cwd).unwrap();
        // Should find skill-30
        assert!(skills.iter().any(|s| s.name == "skill-30"));

        // The static cache size should be <= 50 (it may be slightly less because
        // the first entry was re-loaded, bumping out another).
        assert!(
            LOAD_ALL_SKILLS_MEMO.size() <= 50,
            "Cache size {} exceeds max 50",
            LOAD_ALL_SKILLS_MEMO.size()
        );
    }

    #[test]
    fn test_skills_dir_key_equality() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let k1 = SkillsDirKey {
            base_path: "/a".to_string(),
            cwd: "/b".to_string(),
        };
        let k2 = SkillsDirKey {
            base_path: "/a".to_string(),
            cwd: "/b".to_string(),
        };
        let k3 = SkillsDirKey {
            base_path: "/c".to_string(),
            cwd: "/d".to_string(),
        };
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
        // Hash equality
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        k1.hash(&mut h1);
        k2.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_memoization_different_keys_return_different_results() {
        use std::io::Write;

        LOAD_ALL_SKILLS_MEMO.clear();

        // Create two temp directories with different skills
        let temp_a = tempfile::tempdir().unwrap();
        let temp_b = tempfile::tempdir().unwrap();

        for (temp, name) in [(&temp_a, "skill-a"), (&temp_b, "skill-b")] {
            let skill_dir = temp.path().join(".ai").join("skills").join(name);
            std::fs::create_dir_all(&skill_dir).unwrap();
            let mut sf = std::fs::File::create(skill_dir.join("SKILL.md")).unwrap();
            writeln!(sf, "---").unwrap();
            writeln!(sf, "description: {name}").unwrap();
            writeln!(sf, "---").unwrap();
            writeln!(sf, "Body").unwrap();
            drop(sf);
        }

        let cwd_a = temp_a.path().to_string_lossy().to_string();
        let cwd_b = temp_b.path().to_string_lossy().to_string();

        let skills_a = load_all_skills_cached(&cwd_a).unwrap();
        let skills_b = load_all_skills_cached(&cwd_b).unwrap();

        assert!(skills_a.iter().any(|s| s.name == "skill-a"));
        assert!(!skills_a.iter().any(|s| s.name == "skill-b"));
        assert!(skills_b.iter().any(|s| s.name == "skill-b"));
        assert!(!skills_b.iter().any(|s| s.name == "skill-a"));
    }

    #[test]
    fn test_substitute_env_vars_in_skill() {
        // Test ${CLAUDE_SKILL_DIR} substitution
        let content = "Script in ${CLAUDE_SKILL_DIR}/bin/run.sh";
        let result = substitute_env_vars_in_skill(&content, "/home/user/.ai/skills/my-skill");
        assert_eq!(result, "Script in /home/user/.ai/skills/my-skill/bin/run.sh");

        // Test ${CLAUDE_SESSION_ID} substitution
        let content = "Session: ${CLAUDE_SESSION_ID}";
        let result = substitute_env_vars_in_skill(&content, "/some/dir");
        // Session ID is generated, so we just check that the placeholder was replaced
        assert!(!result.contains("${CLAUDE_SESSION_ID}"));
        assert!(result.starts_with("Session: "));

        // Test both substitutions together
        let content = "Dir: ${CLAUDE_SKILL_DIR}, Session: ${CLAUDE_SESSION_ID}";
        let result = substitute_env_vars_in_skill(&content, "/skills/test");
        assert!(!result.contains("${CLAUDE_SKILL_DIR}"));
        assert!(!result.contains("${CLAUDE_SESSION_ID}"));
        assert!(result.contains("Dir: /skills/test"));
    }

    #[test]
    fn test_estimate_skill_frontmatter_tokens() {
        let metadata = SkillMetadata {
            name: "my-skill".to_string(),
            description: "A skill that does something useful".to_string(),
            display_name: None,
            version: None,
            allowed_tools: None,
            argument_hint: None,
            arg_names: None,
            when_to_use: Some("When you need help".to_string()),
            user_invocable: None,
            paths: None,
            hooks: None,
            effort: None,
            model: None,
            context: None,
            agent: None,
            shell: None,
        };
        let tokens = estimate_skill_frontmatter_tokens(&metadata);
        // "my-skill A skill that does something useful When you need help"
        // should be a positive number of tokens
        assert!(tokens > 0);

        // Empty metadata should return 0 tokens
        let empty = SkillMetadata {
            name: "".to_string(),
            description: "".to_string(),
            display_name: None,
            version: None,
            allowed_tools: None,
            argument_hint: None,
            arg_names: None,
            when_to_use: None,
            user_invocable: None,
            paths: None,
            hooks: None,
            effort: None,
            model: None,
            context: None,
            agent: None,
            shell: None,
        };
        let empty_tokens = estimate_skill_frontmatter_tokens(&empty);
        assert_eq!(empty_tokens, 0);
    }

    #[test]
    fn test_load_skill_parses_version_and_display_name() {
        use std::io::Write;

        let temp = tempfile::tempdir().unwrap();
        let skill_dir = temp.path().join("versioned-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();

        let mut skill_file = std::fs::File::create(skill_dir.join("SKILL.md")).unwrap();
        writeln!(skill_file, "---").unwrap();
        writeln!(skill_file, "name: My Display Name").unwrap();
        writeln!(skill_file, "version: 2.1.0").unwrap();
        writeln!(skill_file, "description: A versioned skill").unwrap();
        writeln!(skill_file, "---").unwrap();
        writeln!(skill_file, "Skill body content").unwrap();
        drop(skill_file);

        let skill = load_skill_from_dir(&skill_dir).unwrap();
        assert_eq!(skill.metadata.name, "versioned-skill");
        assert_eq!(skill.metadata.display_name.as_deref(), Some("My Display Name"));
        assert_eq!(skill.metadata.version.as_deref(), Some("2.1.0"));
        assert_eq!(skill.metadata.description, "A versioned skill");
    }
}
