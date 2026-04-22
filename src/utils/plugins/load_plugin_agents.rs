// Source: ~/claudecode/openclaudecode/src/utils/plugins/loadPluginAgents.ts
#![allow(dead_code)]

use std::collections::HashSet;
use std::path::Path;
use std::sync::Mutex;

use once_cell::sync::Lazy;

use super::loader::load_all_plugins_cache_only;
use super::plugin_options_storage::{
    load_plugin_options, substitute_plugin_variables, substitute_user_config_in_content,
};
use super::walk_plugin_markdown::{WalkPluginMarkdownOpts, walk_plugin_markdown};
use crate::plugin::types::PluginManifest;

/// Stub for frontmatter parsing - requires frontmatter_parser module.
fn parse_frontmatter_stub<'a>(
    content: &'a str,
    _path: &str,
) -> (serde_json::Map<String, serde_json::Value>, &'a str) {
    // Simple stub: returns empty frontmatter and full content as markdown
    (serde_json::Map::new(), content)
}

static PLUGIN_AGENT_CACHE: Lazy<Mutex<Option<Vec<AgentDefinition>>>> =
    Lazy::new(|| Mutex::new(None));

/// Agent definition loaded from a plugin.
#[derive(Clone, Debug)]
pub struct AgentDefinition {
    pub agent_type: String,
    pub when_to_use: String,
    pub tools: Option<Vec<String>>,
    pub skills: Option<Vec<String>>,
    pub color: Option<String>,
    pub model: Option<String>,
    pub background: Option<bool>,
    pub system_prompt: String,
    pub source: String,
    pub filename: String,
    pub plugin: String,
    pub memory: Option<String>,
    pub isolation: Option<String>,
    pub effort: Option<u32>,
    pub max_turns: Option<u32>,
    pub disallowed_tools: Option<Vec<String>>,
}

/// Load plugin agents from all enabled plugins.
pub async fn load_plugin_agents()
-> Result<Vec<AgentDefinition>, Box<dyn std::error::Error + Send + Sync>> {
    // Return cached result if available
    {
        let cache = PLUGIN_AGENT_CACHE.lock().unwrap();
        if let Some(ref agents) = *cache {
            return Ok(agents.clone());
        }
    }

    let plugin_result = load_all_plugins_cache_only().await?;

    let mut all_agents = Vec::new();

    for plugin in &plugin_result.enabled {
        let mut loaded_paths = HashSet::new();

        // Load agents from default agents directory
        if let Some(ref agents_path) = plugin.agents_path {
            if let Ok(agents) = load_agents_from_directory(
                Path::new(agents_path),
                &plugin.name,
                &plugin.source,
                &plugin.path,
                &plugin.manifest,
                &mut loaded_paths,
            )
            .await
            {
                log::debug!(
                    "Loaded {} agents from plugin {} default directory",
                    agents.len(),
                    plugin.name
                );
                all_agents.extend(agents);
            }
        }

        // Load agents from additional paths specified in manifest
        if let Some(ref agents_paths) = plugin.agents_paths {
            for agent_path in agents_paths {
                if let Ok(agents) = load_agents_from_path(
                    agent_path,
                    &plugin.name,
                    &plugin.source,
                    &plugin.path,
                    &plugin.manifest,
                    &mut loaded_paths,
                )
                .await
                {
                    all_agents.extend(agents);
                }
            }
        }
    }

    log::debug!("Total plugin agents loaded: {}", all_agents.len());

    // Cache the result
    {
        let mut cache = PLUGIN_AGENT_CACHE.lock().unwrap();
        *cache = Some(all_agents.clone());
    }

    Ok(all_agents)
}

async fn load_agents_from_directory(
    agents_path: &Path,
    plugin_name: &str,
    source_name: &str,
    plugin_path: &str,
    plugin_manifest: &PluginManifest,
    loaded_paths: &mut HashSet<String>,
) -> Result<Vec<AgentDefinition>, Box<dyn std::error::Error + Send + Sync>> {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let agents: Arc<Mutex<Vec<AgentDefinition>>> = Arc::new(Mutex::new(Vec::new()));

    walk_plugin_markdown(
        agents_path,
        |full_path, namespace| {
            let plugin_name = plugin_name.to_string();
            let source_name = source_name.to_string();
            let plugin_path = plugin_path.to_string();
            let manifest = plugin_manifest.clone();
            let agents = Arc::clone(&agents);

            Box::pin(async move {
                match load_agent_from_file(
                    &full_path,
                    &plugin_name,
                    &namespace,
                    &source_name,
                    &plugin_path,
                    &manifest,
                    &mut HashSet::new(),
                )
                .await
                {
                    Ok(Some(agent)) => agents.lock().await.push(agent),
                    Ok(None) => {}
                    Err(e) => log::debug!("Failed to load agent from {:?}: {}", full_path, e),
                }
            })
        },
        WalkPluginMarkdownOpts {
            stop_at_skill_dir: Some(false),
            log_label: Some("agents".to_string()),
        },
    )
    .await
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    Ok(Arc::try_unwrap(agents).unwrap().into_inner())
}

async fn load_agents_from_path(
    agent_path: &str,
    plugin_name: &str,
    source_name: &str,
    plugin_path: &str,
    plugin_manifest: &PluginManifest,
    loaded_paths: &mut HashSet<String>,
) -> Result<Vec<AgentDefinition>, Box<dyn std::error::Error + Send + Sync>> {
    load_agents_from_directory(
        Path::new(agent_path),
        plugin_name,
        source_name,
        plugin_path,
        plugin_manifest,
        loaded_paths,
    )
    .await
}

async fn load_agent_from_file(
    file_path: &str,
    plugin_name: &str,
    namespace: &[String],
    source_name: &str,
    plugin_path: &str,
    plugin_manifest: &PluginManifest,
    loaded_paths: &mut HashSet<String>,
) -> Result<Option<AgentDefinition>, Box<dyn std::error::Error + Send + Sync>> {
    if loaded_paths.contains(file_path) {
        return Ok(None);
    }
    loaded_paths.insert(file_path.to_string());

    let content = tokio::fs::read_to_string(file_path)
        .await
        .map_err(|e| format!("Failed to read {}: {}", file_path, e))?;
    let (frontmatter, markdown_content) = parse_frontmatter_stub(&content, file_path);

    let base_agent_name = match frontmatter.get("name").and_then(|v| v.as_str()) {
        Some(name) => name.to_string(),
        None => std::path::Path::new(file_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default(),
    };

    // Apply namespace prefixing
    let mut name_parts = vec![plugin_name.to_string()];
    name_parts.extend(namespace.iter().cloned());
    name_parts.push(base_agent_name.clone());
    let agent_type = name_parts.join(":");

    let when_to_use = frontmatter
        .get("description")
        .or_else(|| frontmatter.get("when-to-use"))
        .and_then(|v| v.as_str())
        .unwrap_or(&format!("Agent from {} plugin", plugin_name))
        .to_string();

    let tools = parse_agent_tools_from_frontmatter(frontmatter.get("tools"));
    let skills = parse_slash_command_tools_from_frontmatter(frontmatter.get("skills"));
    let color = frontmatter
        .get("color")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let model = frontmatter.get("model").and_then(|v| v.as_str()).map(|s| {
        let trimmed = s.trim().to_lowercase();
        if trimmed == "inherit" {
            "inherit".to_string()
        } else {
            s.trim().to_string()
        }
    });

    let background = frontmatter
        .get("background")
        .and_then(|v| v.as_str())
        .map(|s| s == "true")
        .or_else(|| frontmatter.get("background").and_then(|v| v.as_bool()));

    let mut system_prompt =
        substitute_plugin_variables(markdown_content.trim(), plugin_path, source_name);
    if plugin_manifest.user_config.is_some() {
        let options = load_plugin_options(source_name);
        system_prompt = substitute_user_config_in_content(
            &system_prompt,
            &options,
            plugin_manifest.user_config.as_ref().unwrap(),
        );
    }

    Ok(Some(AgentDefinition {
        agent_type,
        when_to_use,
        tools,
        skills,
        color,
        model,
        background,
        system_prompt,
        source: "plugin".to_string(),
        filename: base_agent_name,
        plugin: source_name.to_string(),
        memory: None,
        isolation: None,
        effort: None,
        max_turns: None,
        disallowed_tools: None,
    }))
}

fn parse_agent_tools_from_frontmatter(value: Option<&serde_json::Value>) -> Option<Vec<String>> {
    match value {
        Some(serde_json::Value::String(s)) => Some(
            s.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        ),
        Some(serde_json::Value::Array(arr)) => Some(
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
        ),
        _ => None,
    }
}

fn parse_slash_command_tools_from_frontmatter(
    value: Option<&serde_json::Value>,
) -> Option<Vec<String>> {
    parse_agent_tools_from_frontmatter(value)
}

/// Clear the plugin agent cache.
pub fn clear_plugin_agent_cache() {
    let mut cache = PLUGIN_AGENT_CACHE.lock().unwrap();
    *cache = None;
}
