// Source: ~/claudecode/openclaudecode/src/tools/AgentTool/agentMemory.ts
#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// Persistent agent memory scope: 'user', 'project', or 'local'
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMemoryScope {
    User,
    Project,
    Local,
}

impl AgentMemoryScope {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "user" => Some(AgentMemoryScope::User),
            "project" => Some(AgentMemoryScope::Project),
            "local" => Some(AgentMemoryScope::Local),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AgentMemoryScope::User => "user",
            AgentMemoryScope::Project => "project",
            AgentMemoryScope::Local => "local",
        }
    }
}

/// Sanitize an agent type name for use as a directory name.
/// Replaces colons with dashes (for plugin-namespaced agent types).
fn sanitize_agent_type_for_path(agent_type: &str) -> String {
    agent_type.replace(':', "-")
}

/// Returns the base directory for memory storage.
/// Uses CLAUDE_CODE_MEMORY_BASE_DIR if set, otherwise defaults to ~/.claude.
fn get_memory_base_dir() -> PathBuf {
    std::env::var("CLAUDE_CODE_MEMORY_BASE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".claude")
        })
}

/// Returns the current working directory.
fn get_cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Returns the local agent memory directory.
fn get_local_agent_memory_dir(dir_name: &str) -> PathBuf {
    if let Ok(remote_dir) = std::env::var("CLAUDE_CODE_REMOTE_MEMORY_DIR") {
        let project_root = get_project_root();
        PathBuf::from(&remote_dir)
            .join("projects")
            .join(sanitize_path(&project_root))
            .join("agent-memory-local")
            .join(dir_name)
    } else {
        get_cwd()
            .join(".claude")
            .join("agent-memory-local")
            .join(dir_name)
    }
}

/// Sanitize a path for use in a directory name.
fn sanitize_path(path: &str) -> String {
    path.replace(|c: char| !c.is_alphanumeric() && c != '/' && c != '-' && c != '_', "_")
}

/// Get the project root (git root or current directory).
fn get_project_root() -> String {
    // Simplified: use current directory as project root
    get_cwd().to_string_lossy().to_string()
}

/// Returns the agent memory directory for a given agent type and scope.
pub fn get_agent_memory_dir(agent_type: &str, scope: AgentMemoryScope) -> PathBuf {
    let dir_name = sanitize_agent_type_for_path(agent_type);
    match scope {
        AgentMemoryScope::Project => {
            get_cwd().join(".claude").join("agent-memory").join(dir_name)
        }
        AgentMemoryScope::Local => get_local_agent_memory_dir(&dir_name),
        AgentMemoryScope::User => {
            get_memory_base_dir().join("agent-memory").join(dir_name)
        }
    }
}

/// Check if file is within an agent memory directory (any scope).
pub fn is_agent_memory_path(absolute_path: &str) -> bool {
    let normalized = Path::new(absolute_path).canonicalize().unwrap_or_else(|_| absolute_path.into());
    let normalized_str = normalized.to_string_lossy();
    let memory_base = get_memory_base_dir();

    // User scope
    if normalized_str.starts_with(&memory_base.join("agent-memory").to_string_lossy().to_string()) {
        return true;
    }

    // Project scope
    let project_mem = get_cwd().join(".claude").join("agent-memory");
    if normalized_str.starts_with(&project_mem.to_string_lossy().to_string()) {
        return true;
    }

    // Local scope
    if let Ok(remote_dir) = std::env::var("CLAUDE_CODE_REMOTE_MEMORY_DIR") {
        if normalized_str.contains("agent-memory-local")
            && normalized_str.starts_with(&format!("{}/projects", remote_dir))
        {
            return true;
        }
    } else {
        let local_mem = get_cwd().join(".claude").join("agent-memory-local");
        if normalized_str.starts_with(&local_mem.to_string_lossy().to_string()) {
            return true;
        }
    }

    false
}

/// Returns the agent memory file path for a given agent type and scope.
pub fn get_agent_memory_entrypoint(agent_type: &str, scope: AgentMemoryScope) -> PathBuf {
    get_agent_memory_dir(agent_type, scope).join("MEMORY.md")
}

/// Get a human-readable display string for the memory scope.
pub fn get_memory_scope_display(scope: Option<AgentMemoryScope>) -> &'static str {
    match scope {
        Some(AgentMemoryScope::User) => "User (~/.claude/agent-memory/)",
        Some(AgentMemoryScope::Project) => "Project (.claude/agent-memory/)",
        Some(AgentMemoryScope::Local) => "Local (.claude/agent-memory-local/)",
        None => "None",
    }
}

/// Load persistent memory for an agent with memory enabled.
/// Creates the memory directory if needed and returns a prompt with memory contents.
pub fn load_agent_memory_prompt(agent_type: &str, scope: AgentMemoryScope) -> String {
    let scope_note = match scope {
        AgentMemoryScope::User => {
            "- Since this memory is user-scope, keep learnings general since they apply across all projects"
        }
        AgentMemoryScope::Project => {
            "- Since this memory is project-scope and shared with your team via version control, tailor your memories to this project"
        }
        AgentMemoryScope::Local => {
            "- Since this memory is local-scope (not checked into version control), tailor your memories to this project and machine"
        }
    };

    let memory_dir = get_agent_memory_dir(agent_type, scope);

    // Fire-and-forget: ensure directory exists
    let _ = std::fs::create_dir_all(&memory_dir);

    let extra_guidelines = std::env::var("CLAUDE_COWORK_MEMORY_EXTRA_GUIDELINES").ok();
    let extra_guidelines = extra_guidelines
        .as_deref()
        .filter(|s| !s.trim().is_empty());

    build_memory_prompt(
        "Persistent Agent Memory",
        &memory_dir,
        if let Some(guidelines) = extra_guidelines {
            vec![scope_note, guidelines]
        } else {
            vec![scope_note]
        },
    )
}

/// Build a memory prompt string.
fn build_memory_prompt(display_name: &str, memory_dir: &Path, extra_guidelines: Vec<&str>) -> String {
    let memory_contents = read_memory_files(memory_dir);
    let guidelines = extra_guidelines.join("\n");

    format!(
        "# {display_name}\n\n\
         Memory directory: {memory_dir}\n\n\
         {guidelines}\n\n\
         {memory_contents}",
        memory_dir = memory_dir.display()
    )
}

/// Read all .md files from the memory directory.
fn read_memory_files(memory_dir: &Path) -> String {
    let mut contents = String::new();

    if let Ok(entries) = std::fs::read_dir(memory_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    contents.push_str(&format!("\n--- {} ---\n{}\n", path.display(), content));
                }
            }
        }
    }

    contents
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_agent_type() {
        assert_eq!(sanitize_agent_type_for_path("my-agent"), "my-agent");
        assert_eq!(sanitize_agent_type_for_path("my-plugin:my-agent"), "my-plugin-my-agent");
    }

    #[test]
    fn test_memory_scope_from_str() {
        assert_eq!(AgentMemoryScope::from_str("user"), Some(AgentMemoryScope::User));
        assert_eq!(AgentMemoryScope::from_str("project"), Some(AgentMemoryScope::Project));
        assert_eq!(AgentMemoryScope::from_str("local"), Some(AgentMemoryScope::Local));
        assert_eq!(AgentMemoryScope::from_str("invalid"), None);
    }

    #[test]
    fn test_memory_scope_display() {
        assert_eq!(get_memory_scope_display(None), "None");
        assert_eq!(get_memory_scope_display(Some(AgentMemoryScope::User)), "User (~/.claude/agent-memory/)");
    }

    #[test]
    fn test_get_agent_memory_entrypoint() {
        let path = get_agent_memory_entrypoint("test-agent", AgentMemoryScope::Project);
        assert!(path.to_string_lossy().contains("agent-memory"));
        assert!(path.to_string_lossy().contains("MEMORY.md"));
    }
}
