//! Bundled skills infrastructure - ported from openclaudecode/src/skills/bundledSkills.ts
//!
//! Provides the core types and functions for registering bundled skills
//! that ship with the SDK.

use crate::AgentError;
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

/// Skill prompt function type - returns content blocks for the skill prompt
pub type GetPromptForCommandFn =
    fn(args: &str, context: &SkillContext) -> Result<Vec<ContentBlock>, AgentError>;

/// Internal registry for bundled skills
static BUNDLED_SKILLS: OnceLock<Mutex<Vec<BundledSkill>>> = OnceLock::new();

#[derive(Debug, Clone)]
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

    // Note: Files extraction not implemented in initial port
    // Would need to implement extract_bundled_skill_files()

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
        skill_root: None,
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
