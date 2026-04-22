//! AI.md instruction file loading system
//!
//! This module provides the AI.md reading functionality.
//! It loads project instructions at startup.
//!
//! Files are loaded in the following order (reverse priority - later = higher):
//! 1. Managed memory - Global instructions for all users
//! 2. User memory - Private global instructions for all projects
//! 3. Project memory - Instructions checked into the codebase
//! 4. Local memory - Private project-specific instructions

pub mod loader;
pub mod types;

pub use loader::{
    AI_MD_FILENAME, AI_MD_LOCAL_FILENAME, CLAUDE_LOCAL_MD_FILENAME, CLAUDE_MD_FILENAME,
    PROJECT_RULES_DIR, get_ai_md_files, load_ai_md, process_ai_md_file,
};
pub use types::{AiMdContent, AiMdFile, AiMdType};

/// Instruction prompt shown when loading AI.md files
pub const AI_MD_INSTRUCTION_PROMPT: &str = "Codebase and user instructions are shown below. Be sure to adhere to these instructions. IMPORTANT: These instructions OVERRIDE any default behavior and you MUST follow them exactly as written.";

/// Maximum character count for AI.md content
pub const MAX_AI_MD_CHARACTER_COUNT: usize = 40000;
