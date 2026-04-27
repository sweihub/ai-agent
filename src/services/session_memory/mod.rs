pub mod prompts;
pub mod session_memory;
pub mod session_memory_utils;

pub use prompts::*;
pub use session_memory::*;
pub use session_memory_utils::*;

/// Backward-compatible re-exports for callers that still import from
/// `crate::session_memory` (away_summary, session_memory_compact, tests).
pub use crate::utils::permissions::filesystem::{
    get_session_memory_dir, get_session_memory_path,
};
