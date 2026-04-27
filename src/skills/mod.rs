//! Skills module - ported from ~/claudecode/openclaudecode/src/skills
//!
//! This module provides the bundled skills infrastructure for the Rust SDK.

pub mod bundled;
pub mod bundled_skills;
pub mod loader;
pub mod mcp_skill_builders;

pub use bundled::init_bundled_skills;
pub use bundled_skills::*;
pub use loader::{
    LoadedSkill, SkillMetadata, SkillSource, UnifiedSkill, SkillsDirKey, load_all_skills,
    load_all_skills_cached, load_skill_from_dir, load_skills_from_dir,
    load_skills_from_dir_cached, get_user_skills_dir, get_project_skills_dir,
    substitute_env_vars_in_skill, estimate_skill_frontmatter_tokens,
};
