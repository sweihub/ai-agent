// Source: ~/claudecode/openclaudecode/src/tools/AgentTool/
pub mod agent_color_manager;
pub mod agent_display;
pub mod agent_memory;
pub mod agent_memory_snapshot;
pub mod agent_tool_utils;
pub mod built_in_agents;
pub mod built_in_dir;
pub mod constants;
pub mod fork_subagent;
pub mod load_agents_dir;
pub mod prompt;
pub mod resume_agent;
pub mod run_agent;

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

// Re-export commonly used types
pub use load_agents_dir::AgentDefinition;
pub use agent_color_manager::AgentColorName;

/// Global agent color map
static AGENT_COLOR_MAP: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn get_agent_color_map() -> &'static Mutex<HashMap<String, String>> {
    &AGENT_COLOR_MAP
}
