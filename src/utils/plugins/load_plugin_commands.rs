// Source: ~/claudecode/openclaudecode/src/utils/plugins/loadPluginCommands.ts
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

static PLUGIN_COMMAND_CACHE: Lazy<Mutex<Option<Vec<Command>>>> = Lazy::new(|| Mutex::new(None));

/// Command definition loaded from a plugin.
#[derive(Clone, Debug)]
pub struct Command {
    pub command_type: String,
    pub name: String,
    pub description: String,
    pub content: String,
    pub source: String,
    pub plugin: Option<String>,
    pub is_hidden: bool,
    pub allowed_tools: Vec<String>,
}

/// Load plugin commands from all enabled plugins.
pub async fn load_plugin_commands() -> Result<Vec<Command>, Box<dyn std::error::Error + Send + Sync>>
{
    {
        let cache = PLUGIN_COMMAND_CACHE.lock().unwrap();
        if let Some(ref commands) = *cache {
            return Ok(commands.clone());
        }
    }

    let plugin_result = load_all_plugins_cache_only().await?;
    let mut all_commands = Vec::new();

    for plugin in &plugin_result.enabled {
        let mut loaded_paths = HashSet::new();

        // Load commands from default commands directory
        if let Some(ref commands_path) = plugin.commands_path {
            match load_commands_from_directory(
                Path::new(commands_path),
                &plugin.name,
                &plugin.source,
                &plugin.manifest,
                &plugin.path,
                &mut loaded_paths,
                false,
            )
            .await
            {
                Ok(commands) => {
                    log::debug!(
                        "Loaded {} commands from plugin {} default directory",
                        commands.len(),
                        plugin.name
                    );
                    all_commands.extend(commands);
                }
                Err(e) => log::debug!(
                    "Failed to load commands from plugin {} default directory: {}",
                    plugin.name,
                    e
                ),
            }
        }

        // Load commands from additional paths
        if let Some(ref commands_paths) = plugin.commands_paths {
            for command_path in commands_paths {
                if let Ok(commands) = load_commands_from_path(
                    command_path,
                    &plugin.name,
                    &plugin.source,
                    &plugin.manifest,
                    &plugin.path,
                    &mut loaded_paths,
                )
                .await
                {
                    all_commands.extend(commands);
                }
            }
        }
    }

    log::debug!("Total plugin commands loaded: {}", all_commands.len());

    {
        let mut cache = PLUGIN_COMMAND_CACHE.lock().unwrap();
        *cache = Some(all_commands.clone());
    }

    Ok(all_commands)
}

async fn load_commands_from_directory(
    commands_path: &Path,
    plugin_name: &str,
    source_name: &str,
    plugin_manifest: &PluginManifest,
    plugin_path: &str,
    loaded_paths: &mut HashSet<String>,
    is_skill_mode: bool,
) -> Result<Vec<Command>, Box<dyn std::error::Error + Send + Sync>> {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let commands: Arc<Mutex<Vec<Command>>> = Arc::new(Mutex::new(Vec::new()));

    walk_plugin_markdown(
        commands_path,
        |full_path, _namespace| {
            let plugin_name = plugin_name.to_string();
            let source_name = source_name.to_string();
            let manifest = plugin_manifest.clone();
            let plugin_path = plugin_path.to_string();
            let base_dir = commands_path.to_path_buf();
            let commands = Arc::clone(&commands);

            Box::pin(async move {
                let path_str = full_path.clone();

                if let Ok(content) = tokio::fs::read_to_string(&full_path).await {
                    let command_name =
                        get_command_name_from_file(Path::new(&full_path), &base_dir, &plugin_name);

                    if let Ok(Some(command)) = create_plugin_command(
                        &command_name,
                        &content,
                        &path_str,
                        &source_name,
                        &manifest,
                        &plugin_path,
                        false,
                        is_skill_mode,
                    )
                    .await
                    {
                        commands.lock().await.push(command);
                    }
                }
            })
        },
        WalkPluginMarkdownOpts {
            stop_at_skill_dir: Some(false),
            log_label: Some("commands".to_string()),
        },
    )
    .await
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    Ok(Arc::try_unwrap(commands).unwrap().into_inner())
}

fn get_command_name_from_file(file_path: &Path, base_dir: &Path, plugin_name: &str) -> String {
    let is_skill = is_skill_file(file_path);

    let command_base_name = if is_skill {
        file_path
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    } else {
        file_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default()
    };

    let relative_path = file_path
        .parent()
        .and_then(|p| p.strip_prefix(base_dir).ok())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let namespace = if relative_path.is_empty() {
        String::new()
    } else {
        relative_path.replace('/', ":")
    };

    if namespace.is_empty() {
        format!("{}:{}", plugin_name, command_base_name)
    } else {
        format!("{}:{}:{}", plugin_name, namespace, command_base_name)
    }
}

fn is_skill_file(file_path: &Path) -> bool {
    file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .map(|n| n.to_lowercase() == "skill.md")
        .unwrap_or(false)
}

async fn create_plugin_command(
    command_name: &str,
    content: &str,
    file_path: &str,
    source_name: &str,
    plugin_manifest: &PluginManifest,
    plugin_path: &str,
    is_skill: bool,
    is_skill_mode: bool,
) -> Result<Option<Command>, Box<dyn std::error::Error + Send + Sync>> {
    let (frontmatter, markdown_content) = parse_frontmatter_stub(content, file_path);

    let description = frontmatter
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or(if is_skill {
            "Plugin skill"
        } else {
            "Plugin command"
        })
        .to_string();

    let mut final_content = if is_skill_mode {
        if let Some(parent) = std::path::Path::new(file_path).parent() {
            format!(
                "Base directory for this skill: {}\n\n{}",
                parent.display(),
                markdown_content
            )
        } else {
            markdown_content.to_string()
        }
    } else {
        markdown_content.to_string()
    };

    // Substitute plugin variables
    final_content = substitute_plugin_variables(&final_content, plugin_path, source_name);

    // Substitute user config
    if plugin_manifest.user_config.is_some() {
        let options = load_plugin_options(source_name);
        final_content = substitute_user_config_in_content(
            &final_content,
            &options,
            plugin_manifest.user_config.as_ref().unwrap(),
        );
    }

    Ok(Some(Command {
        command_type: "prompt".to_string(),
        name: command_name.to_string(),
        description,
        content: final_content,
        source: "plugin".to_string(),
        plugin: Some(source_name.to_string()),
        is_hidden: false,
        allowed_tools: Vec::new(),
    }))
}

async fn load_commands_from_path(
    command_path: &str,
    plugin_name: &str,
    source_name: &str,
    plugin_manifest: &PluginManifest,
    plugin_path: &str,
    loaded_paths: &mut HashSet<String>,
) -> Result<Vec<Command>, Box<dyn std::error::Error + Send + Sync>> {
    let metadata = tokio::fs::metadata(command_path)
        .await
        .map_err(|e| format!("Failed to stat {}: {}", command_path, e))?;

    if metadata.is_dir() {
        load_commands_from_directory(
            Path::new(command_path),
            plugin_name,
            source_name,
            plugin_manifest,
            plugin_path,
            loaded_paths,
            false,
        )
        .await
    } else if metadata.is_file()
        && Path::new(command_path)
            .extension()
            .map(|e| e.to_string_lossy() == "md")
            .unwrap_or(false)
    {
        if loaded_paths.contains(command_path) {
            return Ok(Vec::new());
        }
        loaded_paths.insert(command_path.to_string());

        let content = tokio::fs::read_to_string(command_path)
            .await
            .map_err(|e| format!("Failed to read {}: {}", command_path, e))?;
        let path_str = command_path.to_string();
        let command_name = format!(
            "{}:{}",
            plugin_name,
            Path::new(command_path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
        );

        match create_plugin_command(
            &command_name,
            &content,
            &path_str,
            source_name,
            plugin_manifest,
            plugin_path,
            false,
            false,
        )
        .await
        {
            Ok(Some(cmd)) => Ok(vec![cmd]),
            _ => Ok(Vec::new()),
        }
    } else {
        Ok(Vec::new())
    }
}

/// Clear the plugin command cache.
pub fn clear_plugin_command_cache() {
    let mut cache = PLUGIN_COMMAND_CACHE.lock().unwrap();
    *cache = None;
}
