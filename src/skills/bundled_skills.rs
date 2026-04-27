//! Bundled skills infrastructure - ported from openclaudecode/src/skills/bundledSkills.ts
//!
//! Provides the core types and functions for registering bundled skills
//! that ship with the SDK.

use crate::AgentError;
use crate::utils::hooks::register_skill_hooks::HooksSettings;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// Content block for skill prompts (matches TypeScript ContentBlockParam)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ImageSource },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub data: String,
    pub media_type: String,
}

/// Context passed to skill prompt functions
#[derive(Debug, Clone)]
pub struct SkillContext {
    pub cwd: String,
    // Add more context fields as needed
}

/// Skill prompt function type - returns content blocks for the skill prompt.
/// Uses `Arc` for cloneability across callers.
pub type GetPromptForCommandFn =
    std::sync::Arc<dyn Fn(&str, &SkillContext) -> Result<Vec<ContentBlock>, AgentError> + Send + Sync>;

/// Internal registry for bundled skills
static BUNDLED_SKILLS: OnceLock<Mutex<Vec<BundledSkill>>> = OnceLock::new();

#[derive(Clone)]
pub struct BundledSkill {
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub has_user_specified_description: bool,
    pub allowed_tools: Vec<String>,
    pub argument_hint: Option<String>,
    pub when_to_use: Option<String>,
    pub model: Option<String>,
    pub disable_model_invocation: bool,
    pub user_invocable: bool,
    pub content_length: usize,
    pub source: String,
    pub loaded_from: String,
    pub skill_root: Option<String>,
    pub context: Option<String>,
    pub agent: Option<String>,
    pub is_enabled: Option<fn() -> bool>,
    pub hooks: Option<HooksSettings>,
    pub is_hidden: bool,
    pub progress_message: String,
    pub get_prompt_for_command: GetPromptForCommandFn,
}

/// Definition for a bundled skill that ships with the SDK.
/// These are registered programmatically at startup.
pub struct BundledSkillDefinition {
    pub name: String,
    pub description: String,
    pub aliases: Option<Vec<String>>,
    pub when_to_use: Option<String>,
    pub argument_hint: Option<String>,
    pub allowed_tools: Option<Vec<String>>,
    pub model: Option<String>,
    pub disable_model_invocation: Option<bool>,
    pub user_invocable: Option<bool>,
    pub is_enabled: Option<fn() -> bool>,
    pub hooks: Option<HooksSettings>,
    pub context: Option<String>,
    pub agent: Option<String>,
    /// Additional reference files to extract to disk on first invocation.
    /// Keys are relative paths (forward slashes, no `..`), values are content.
    pub files: Option<std::collections::HashMap<String, String>>,
    /// Required: function to generate the prompt for this skill
    pub get_prompt_for_command: GetPromptForCommandFn,
}

/// Register a bundled skill that will be available to the model.
/// Call this at module initialization or in an init function.
///
/// Bundled skills are compiled into the SDK and available to all users.
pub fn register_bundled_skill(definition: BundledSkillDefinition) -> Result<(), AgentError> {
    let mutex = BUNDLED_SKILLS.get_or_init(|| Mutex::new(Vec::new()));
    let mut skills = mutex
        .lock()
        .map_err(|e| AgentError::Internal(format!("Lock error: {}", e)))?;

    let get_prompt_for_command = definition.get_prompt_for_command;
    let files = definition.files.clone();

    // Determine skill_root: if the skill has bundled files, set up the extract dir.
    let skill_root = if files.is_some() && files.as_ref().map_or(0, |f| f.len()) > 0 {
        let dir = get_bundled_skill_extract_dir(&definition.name);
        // Extract files to disk now (synchronous, unlike TS which does it lazily)
        let _ = extract_bundled_skill_files(&definition.name, files.as_ref().unwrap());
        Some(dir)
    } else {
        None
    };

    let skill = BundledSkill {
        name: definition.name.clone(),
        description: definition.description.clone(),
        aliases: definition.aliases.unwrap_or_default(),
        has_user_specified_description: true,
        allowed_tools: definition.allowed_tools.unwrap_or_default(),
        argument_hint: definition.argument_hint,
        when_to_use: definition.when_to_use,
        model: definition.model,
        disable_model_invocation: definition.disable_model_invocation.unwrap_or(false),
        user_invocable: definition.user_invocable.unwrap_or(true),
        content_length: 0,
        source: "bundled".to_string(),
        loaded_from: "bundled".to_string(),
        hooks: definition.hooks,
        skill_root,
        context: definition.context,
        agent: definition.agent,
        is_enabled: definition.is_enabled,
        is_hidden: !(definition.user_invocable.unwrap_or(true)),
        progress_message: "running".to_string(),
        get_prompt_for_command,
    };

    skills.push(skill);
    Ok(())
}

/// Memoized root directory for bundled skill file extractions.
/// Mirrors TypeScript `getBundledSkillsRoot()` with a per-process nonce.
fn get_bundled_skills_root() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE.get_or_init(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let base = PathBuf::from(&home).join(".ai").join("tmp");
        let nonce: u64 = rand::random();
        let root = base.join(format!("bundled-skills/{}/{}", env!("CARGO_PKG_VERSION"), nonce));
        let _ = std::fs::create_dir_all(&root);
        root.to_string_lossy().to_string()
    })
    .clone()
}

/// Deterministic extraction directory for a bundled skill's reference files.
/// Mirrors TypeScript `getBundledSkillExtractDir()`.
pub fn get_bundled_skill_extract_dir(skill_name: &str) -> String {
    format!("{}/{}", get_bundled_skills_root(), skill_name)
}

/// Extract a bundled skill's reference files to disk so the model can
/// Read/Grep them on demand. Called during skill registration.
///
/// Returns the directory written to, or None if write failed.
pub fn extract_bundled_skill_files(
    skill_name: &str,
    files: &HashMap<String, String>,
) -> Option<String> {
    let dir = get_bundled_skill_extract_dir(skill_name);
    match write_skill_files(&dir, files) {
        Ok(()) => Some(dir),
        Err(e) => {
            eprintln!(
                "Failed to extract bundled skill '{}' to {}: {}",
                skill_name, dir, e
            );
            None
        }
    }
}

/// Write all skill reference files to disk, grouped by parent directory.
fn write_skill_files(
    dir: &str,
    files: &HashMap<String, String>,
) -> Result<(), std::io::Error> {
    // Group by parent dir so we mkdir each subtree once, then write.
    let mut by_parent: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for (rel_path, content) in files {
        let target = resolve_skill_file_path(dir, rel_path)?;
        let parent = target.parent().unwrap().to_path_buf();
        let parent_str = parent.to_string_lossy().to_string();
        let entry = (target.to_string_lossy().to_string(), content.clone());
        by_parent.entry(parent_str).or_default().push(entry);
    }
    for (parent, entries) in by_parent {
        std::fs::create_dir_all(&parent)?;
        for (path, content) in entries {
            safe_write_file(&path, &content)?;
        }
    }
    Ok(())
}

/// Normalize and validate a skill-relative path; returns an error on traversal attempts.
fn resolve_skill_file_path(base_dir: &str, rel_path: &str) -> Result<PathBuf, std::io::Error> {
    let p = Path::new(rel_path);
    // Check for ".." components or absolute paths
    for component in p.components() {
        use std::path::Component;
        if let Component::ParentDir = component {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("bundled skill file path escapes skill dir: {}", rel_path),
            ));
        }
    }
    if p.is_absolute() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("bundled skill file path is absolute: {}", rel_path),
        ));
    }
    Ok(PathBuf::from(base_dir).join(p))
}

/// Write content to a file safely, refusing to overwrite existing files.
fn safe_write_file(path: &str, content: &str) -> Result<(), std::io::Error> {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true) // fail if file already exists
        .mode(0o600)
        .open(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Get all registered bundled skills.
/// Returns a reference to the internal skills list.
pub fn get_bundled_skills() -> Vec<BundledSkill> {
    match BUNDLED_SKILLS.get() {
        Some(mutex) => match mutex.lock() {
            Ok(skills) => skills.clone(),
            Err(_) => Vec::new(),
        },
        None => Vec::new(),
    }
}

/// Clear bundled skills registry (for testing).
pub fn clear_bundled_skills() {
    if let Some(mutex) = BUNDLED_SKILLS.get() {
        if let Ok(mut skills) = mutex.lock() {
            skills.clear();
        }
    }
}

/// Invoke a bundled skill by name.
/// Returns the content blocks generated by the skill.
pub fn invoke_skill(name: &str, args: &str) -> Result<Vec<ContentBlock>, AgentError> {
    let skills = get_bundled_skills();

    // Find skill by name or alias
    let skill = skills
        .iter()
        .find(|s| s.name == name || s.aliases.iter().any(|a| a == name))
        .ok_or_else(|| AgentError::Skill(format!("Skill '{}' not found", name)))?;

    // Create skill context
    let context = SkillContext {
        cwd: std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string()),
    };

    // Call the skill's prompt function
    (skill.get_prompt_for_command)(args, &context)
}
