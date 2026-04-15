//! Plugin loader - ported from ~/claudecode/openclaudecode/src/utils/plugins/pluginLoader.ts
//!
//! This module provides plugin loading functionality for discovering, validating,
//! and loading plugins from various sources (local directories, git repos, npm packages).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::plugin::types::{
    CommandMetadata, LoadedPlugin, PluginComponent, PluginError, PluginManifest,
};

/// Validates a git URL
fn validate_git_url(url: &str) -> Result<String, PluginError> {
    // Check for SSH format (git@host:path)
    if url.starts_with("git@") {
        return Ok(url.to_string());
    }

    // Check for HTTPS/HTTP/FILE protocols
    if let Ok(parsed) = url::Url::parse(url) {
        let scheme = parsed.scheme();
        if ["https", "http", "file"].contains(&scheme) {
            return Ok(url.to_string());
        }
    }

    Err(PluginError::GenericError {
        source: "plugin_loader".to_string(),
        plugin: None,
        error: format!("Invalid git URL: {}", url),
    })
}

/// Check if a path exists
#[allow(dead_code)]
async fn path_exists(path: &Path) -> bool {
    path.exists()
}

/// Validate plugin manifest fields
#[allow(dead_code)]
fn validate_manifest(manifest: &PluginManifest) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Name is required
    if manifest.name.is_empty() {
        errors.push("Plugin name is required".to_string());
    }

    // Check name format (kebab-case recommended)
    if manifest.name.contains(' ') {
        errors.push(format!(
            "Plugin name '{}' should not contain spaces. Use kebab-case.",
            manifest.name
        ));
    }

    // Validate commands if present
    if let Some(ref commands) = manifest.commands {
        // Commands can be either:
        // 1. A string (single path)
        // 2. An array of strings (multiple paths)
        // 3. An object mapping command names to metadata
        match commands {
            serde_json::Value::String(_) => {}
            serde_json::Value::Array(arr) => {
                for item in arr {
                    if !item.is_string() {
                        errors.push("Commands array must contain strings".to_string());
                    }
                }
            }
            serde_json::Value::Object(obj) => {
                for (cmd_name, metadata) in obj {
                    if let serde_json::Value::Object(meta) = metadata {
                        // Validate metadata fields
                        if let Some(source) = meta.get("source") {
                            if !source.is_string() {
                                errors.push(format!(
                                    "Command '{}' source must be a string",
                                    cmd_name
                                ));
                            }
                        }
                        if let Some(content) = meta.get("content") {
                            if !content.is_string() {
                                errors.push(format!(
                                    "Command '{}' content must be a string",
                                    cmd_name
                                ));
                            }
                        }
                    }
                }
            }
            _ => {
                errors.push("Commands must be a string, array, or object".to_string());
            }
        }
    }

    // Validate skills if present
    if let Some(ref skills) = manifest.skills {
        match skills {
            serde_json::Value::String(_) => {}
            serde_json::Value::Array(arr) => {
                for item in arr {
                    if !item.is_string() {
                        errors.push("Skills array must contain strings".to_string());
                    }
                }
            }
            _ => {
                errors.push("Skills must be a string or array".to_string());
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate manifest against schema (simplified version)
fn validate_manifest_schema(manifest: &PluginManifest) -> Result<(), String> {
    // Check required fields from PluginManifestMetadataSchema
    if manifest.name.is_empty() {
        return Err("name is required".to_string());
    }

    // Validate commands format
    if let Some(ref commands) = manifest.commands {
        let valid = match commands {
            serde_json::Value::String(_) => true,
            serde_json::Value::Array(arr) => arr.iter().all(|v| v.is_string()),
            serde_json::Value::Object(obj) => obj.values().all(|v| {
                if let serde_json::Value::Object(meta) = v {
                    meta.contains_key("source") || meta.contains_key("content")
                } else {
                    false
                }
            }),
            _ => false,
        };
        if !valid {
            return Err(
                "commands must be a string, array, or object with source/content fields"
                    .to_string(),
            );
        }
    }

    // Validate agents format
    if let Some(ref agents) = manifest.agents {
        let valid = match agents {
            serde_json::Value::String(_) => true,
            serde_json::Value::Array(arr) => arr.iter().all(|v| v.is_string()),
            _ => false,
        };
        if !valid {
            return Err("agents must be a string or array".to_string());
        }
    }

    // Validate skills format
    if let Some(ref skills) = manifest.skills {
        let valid = match skills {
            serde_json::Value::String(_) => true,
            serde_json::Value::Array(arr) => arr.iter().all(|v| v.is_string()),
            _ => false,
        };
        if !valid {
            return Err("skills must be a string or array".to_string());
        }
    }

    // Validate hooks format
    if let Some(ref hooks) = manifest.hooks {
        if !hooks.is_object() {
            return Err("hooks must be an object".to_string());
        }
    }

    // Validate output_styles format
    if let Some(ref output_styles) = manifest.output_styles {
        let valid = match output_styles {
            serde_json::Value::String(_) => true,
            serde_json::Value::Array(arr) => arr.iter().all(|v| v.is_string()),
            _ => false,
        };
        if !valid {
            return Err("output_styles must be a string or array".to_string());
        }
    }

    Ok(())
}

/// Load plugin manifest from a JSON file
pub fn load_plugin_manifest(manifest_path: &Path) -> Result<PluginManifest, PluginError> {
    if !manifest_path.exists() {
        return Err(PluginError::PathNotFound {
            source: "plugin_loader".to_string(),
            plugin: None,
            path: manifest_path.display().to_string(),
            component: PluginComponent::Commands,
        });
    }

    let content =
        fs::read_to_string(manifest_path).map_err(|e| PluginError::ManifestParseError {
            source: "plugin_loader".to_string(),
            plugin: None,
            manifest_path: manifest_path.display().to_string(),
            parse_error: e.to_string(),
        })?;

    let parsed: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| PluginError::ManifestParseError {
            source: "plugin_loader".to_string(),
            plugin: None,
            manifest_path: manifest_path.display().to_string(),
            parse_error: e.to_string(),
        })?;

    // Parse into PluginManifest
    let manifest: PluginManifest =
        serde_json::from_value(parsed).map_err(|e| PluginError::ManifestParseError {
            source: "plugin_loader".to_string(),
            plugin: None,
            manifest_path: manifest_path.display().to_string(),
            parse_error: e.to_string(),
        })?;

    // Validate schema
    validate_manifest_schema(&manifest).map_err(|err| PluginError::ManifestValidationError {
        source: "plugin_loader".to_string(),
        plugin: Some(manifest.name.clone()),
        manifest_path: manifest_path.display().to_string(),
        validation_errors: vec![err],
    })?;

    Ok(manifest)
}

/// Load plugin manifest, returning a default if not found
pub fn load_plugin_manifest_or_default(
    manifest_path: &Path,
    plugin_name: &str,
    source: &str,
) -> PluginManifest {
    match load_plugin_manifest(manifest_path) {
        Ok(manifest) => manifest,
        Err(_) => PluginManifest {
            name: plugin_name.to_string(),
            version: None,
            description: Some(format!("Plugin from {}", source)),
            author: None,
            homepage: None,
            repository: None,
            license: None,
            keywords: None,
            dependencies: None,
            commands: None,
            agents: None,
            skills: None,
            hooks: None,
            output_styles: None,
            channels: None,
            mcp_servers: None,
            lsp_servers: None,
            settings: None,
            user_config: None,
        },
    }
}

/// Clone a git repository to a target path
pub async fn git_clone(
    git_url: &str,
    target_path: &Path,
    branch: Option<&str>,
    sha: Option<&str>,
) -> Result<(), PluginError> {
    let validated_url = validate_git_url(git_url)?;

    // Ensure parent directory exists
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| PluginError::GenericError {
            source: "plugin_loader".to_string(),
            plugin: None,
            error: format!("Failed to create parent directory: {}", e),
        })?;
    }

    // Build git clone arguments
    let mut args = vec![
        "clone".to_string(),
        "--depth".to_string(),
        "1".to_string(),
        "--recurse-submodules".to_string(),
        "--shallow-submodules".to_string(),
    ];

    // Add branch flag
    if let Some(branch) = branch {
        args.push("--branch".to_string());
        args.push(branch.to_string());
    }

    // If sha is specified, use --no-checkout
    if sha.is_some() {
        args.push("--no-checkout".to_string());
    }

    args.push(validated_url);
    args.push(target_path.display().to_string());

    // Run git clone
    let output =
        Command::new("git")
            .args(&args)
            .output()
            .map_err(|e| PluginError::GenericError {
                source: "plugin_loader".to_string(),
                plugin: None,
                error: format!("Failed to execute git: {}", e),
            })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PluginError::GenericError {
            source: "plugin_loader".to_string(),
            plugin: None,
            error: format!("Git clone failed: {}", stderr),
        });
    }

    // If sha is specified, fetch and checkout that specific commit
    if let Some(sha) = sha {
        // Try shallow fetch first
        let fetch_result = Command::new("git")
            .args(&["fetch", "--depth", "1", "origin", sha])
            .current_dir(target_path)
            .output();

        let fetch_success = match fetch_result {
            Ok(output) => output.status.success(),
            Err(_) => false,
        };

        if !fetch_success {
            // Fall back to unshallow fetch
            let _ = Command::new("git")
                .args(&["fetch", "--unshallow"])
                .current_dir(target_path)
                .output();
        }

        // Checkout the specific commit
        let checkout_output = Command::new("git")
            .args(&["checkout", sha])
            .current_dir(target_path)
            .output()
            .map_err(|e| PluginError::GenericError {
                source: "plugin_loader".to_string(),
                plugin: None,
                error: format!("Failed to checkout commit: {}", e),
            })?;

        if !checkout_output.status.success() {
            let stderr = String::from_utf8_lossy(&checkout_output.stderr);
            return Err(PluginError::GenericError {
                source: "plugin_loader".to_string(),
                plugin: None,
                error: format!("Failed to checkout commit {}: {}", sha, stderr),
            });
        }
    }

    Ok(())
}

/// Install a plugin from npm
pub async fn install_from_npm(
    package_name: &str,
    target_path: &Path,
    version: Option<&str>,
) -> Result<(), PluginError> {
    // Build package spec
    let package_spec = match version {
        Some(v) => format!("{}@{}", package_name, v),
        None => package_name.to_string(),
    };

    // Ensure parent directory exists
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| PluginError::GenericError {
            source: "plugin_loader".to_string(),
            plugin: Some(package_name.to_string()),
            error: format!("Failed to create parent directory: {}", e),
        })?;
    }

    // Run npm install
    let install_result = Command::new("npm")
        .args(&[
            "install",
            &package_spec,
            "--prefix",
            &target_path.display().to_string(),
        ])
        .output()
        .map_err(|e| PluginError::GenericError {
            source: "plugin_loader".to_string(),
            plugin: Some(package_name.to_string()),
            error: format!("Failed to execute npm: {}", e),
        })?;

    if !install_result.status.success() {
        let stderr = String::from_utf8_lossy(&install_result.stderr);
        return Err(PluginError::GenericError {
            source: "plugin_loader".to_string(),
            plugin: Some(package_name.to_string()),
            error: format!("npm install failed: {}", stderr),
        });
    }

    // Find the actual package location in node_modules
    let node_modules_path = target_path.join("node_modules").join(package_name);
    if !node_modules_path.exists() {
        return Err(PluginError::GenericError {
            source: "plugin_loader".to_string(),
            plugin: Some(package_name.to_string()),
            error: format!("Package not found in node_modules: {}", package_name),
        });
    }

    Ok(())
}

/// Copy a directory recursively (non-async for simplicity)
#[allow(dead_code)]
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), PluginError> {
    if !src.exists() {
        return Err(PluginError::PathNotFound {
            source: "plugin_loader".to_string(),
            plugin: None,
            path: src.display().to_string(),
            component: PluginComponent::Commands,
        });
    }

    // Create destination directory
    fs::create_dir_all(dest).map_err(|e| PluginError::GenericError {
        source: "plugin_loader".to_string(),
        plugin: None,
        error: format!("Failed to create destination directory: {}", e),
    })?;

    // Copy entries
    for entry in fs::read_dir(src).map_err(|e| PluginError::GenericError {
        source: "plugin_loader".to_string(),
        plugin: None,
        error: format!("Failed to read source directory: {}", e),
    })? {
        let entry = entry.map_err(|e| PluginError::GenericError {
            source: "plugin_loader".to_string(),
            plugin: None,
            error: format!("Failed to read directory entry: {}", e),
        })?;

        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path).map_err(|e| PluginError::GenericError {
                source: "plugin_loader".to_string(),
                plugin: None,
                error: format!("Failed to copy file: {}", e),
            })?;
        }
    }

    Ok(())
}

/// Validate plugin paths from manifest
#[allow(dead_code)]
fn validate_plugin_paths(
    paths: &[String],
    plugin_path: &Path,
    _plugin_name: &str,
    _source: &str,
    _component: PluginComponent,
) -> Vec<(String, bool)> {
    paths
        .iter()
        .map(|rel_path| {
            let full_path = plugin_path.join(rel_path);
            (rel_path.clone(), full_path.exists())
        })
        .collect()
}

/// Create a LoadedPlugin from a plugin directory path
pub async fn create_plugin_from_path(
    plugin_path: &Path,
    source: &str,
    enabled: bool,
    fallback_name: &str,
) -> Result<LoadedPlugin, PluginError> {
    // Step 1: Load or create the plugin manifest
    // Try multiple possible manifest locations
    let possible_manifest_paths = vec![
        plugin_path.join(".ai-plugin").join("plugin.json"),
        plugin_path.join("plugin.json"),
        plugin_path.join("claude_plugin.json"),
    ];

    let mut manifest: Option<PluginManifest> = None;
    for manifest_path in &possible_manifest_paths {
        if manifest_path.exists() {
            match load_plugin_manifest(manifest_path) {
                Ok(m) => {
                    manifest = Some(m);
                    break;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    // If no manifest found, create default
    let manifest = manifest.unwrap_or_else(|| PluginManifest {
        name: fallback_name.to_string(),
        version: None,
        description: Some(format!("Plugin from {}", source)),
        author: None,
        homepage: None,
        repository: None,
        license: None,
        keywords: None,
        dependencies: None,
        commands: None,
        agents: None,
        skills: None,
        hooks: None,
        output_styles: None,
        channels: None,
        mcp_servers: None,
        lsp_servers: None,
        settings: None,
        user_config: None,
    });

    // Step 2: Create the base plugin object
    let mut plugin = LoadedPlugin {
        name: manifest.name.clone(),
        manifest: manifest.clone(),
        path: plugin_path.display().to_string(),
        source: source.to_string(),
        repository: source.to_string(),
        enabled: Some(enabled),
        is_builtin: None,
        sha: None,
        commands_path: None,
        commands_paths: None,
        commands_metadata: None,
        agents_path: None,
        agents_paths: None,
        skills_path: None,
        skills_paths: None,
        output_styles_path: None,
        output_styles_paths: None,
        hooks_config: None,
        mcp_servers: None,
        lsp_servers: None,
        settings: None,
    };

    // Step 3: Auto-detect optional directories
    let commands_dir = plugin_path.join("commands");
    let agents_dir = plugin_path.join("agents");
    let skills_dir = plugin_path.join("skills");
    let output_styles_dir = plugin_path.join("output-styles");

    // Register detected directories
    if commands_dir.exists() {
        plugin.commands_path = Some(commands_dir.display().to_string());
    }

    if agents_dir.exists() {
        plugin.agents_path = Some(agents_dir.display().to_string());
    }

    if skills_dir.exists() {
        plugin.skills_path = Some(skills_dir.display().to_string());
    }

    if output_styles_dir.exists() {
        plugin.output_styles_path = Some(output_styles_dir.display().to_string());
    }

    // Step 3a: Process command paths from manifest
    if let Some(ref commands) = manifest.commands {
        let cmd_paths: Vec<String> = match commands {
            serde_json::Value::String(s) => vec![s.clone()],
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            serde_json::Value::Object(obj) => {
                // Object mapping format - collect source paths
                let mut paths: Vec<String> = Vec::new();
                let mut metadata_map: HashMap<String, CommandMetadata> = HashMap::new();

                for (cmd_name, metadata) in obj {
                    if let serde_json::Value::Object(meta) = metadata {
                        if let Some(source) = meta.get("source").and_then(|v| v.as_str()) {
                            paths.push(source.to_string());
                        }

                        // Build metadata
                        let meta_obj = CommandMetadata {
                            source: meta
                                .get("source")
                                .and_then(|v| v.as_str().map(String::from)),
                            content: meta
                                .get("content")
                                .and_then(|v| v.as_str().map(String::from)),
                            description: meta
                                .get("description")
                                .and_then(|v| v.as_str().map(String::from)),
                            argument_hint: meta
                                .get("argumentHint")
                                .and_then(|v| v.as_str().map(String::from)),
                            model: meta.get("model").and_then(|v| v.as_str().map(String::from)),
                            allowed_tools: meta.get("allowedTools").and_then(|v| {
                                v.as_array().map(|arr| {
                                    arr.iter()
                                        .filter_map(|item| item.as_str().map(String::from))
                                        .collect()
                                })
                            }),
                        };
                        metadata_map.insert(cmd_name.clone(), meta_obj);
                    }
                }

                plugin.commands_metadata = Some(metadata_map);
                paths
            }
            _ => vec![],
        };

        // Validate and set command paths
        if !cmd_paths.is_empty() {
            let validated: Vec<String> = cmd_paths
                .iter()
                .filter(|p| plugin_path.join(p).exists())
                .map(|p| plugin_path.join(p).display().to_string())
                .collect();

            if !validated.is_empty() {
                plugin.commands_paths = Some(validated);
            }
        }
    }

    // Step 4: Process agent paths from manifest
    if let Some(ref agents) = manifest.agents {
        let agent_paths: Vec<String> = match agents {
            serde_json::Value::String(s) => vec![s.clone()],
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            _ => vec![],
        };

        if !agent_paths.is_empty() {
            let validated: Vec<String> = agent_paths
                .iter()
                .filter(|p| plugin_path.join(p).exists())
                .map(|p| plugin_path.join(p).display().to_string())
                .collect();

            if !validated.is_empty() {
                plugin.agents_paths = Some(validated);
            }
        }
    }

    // Step 5: Process skill paths from manifest
    if let Some(ref skills) = manifest.skills {
        let skill_paths: Vec<String> = match skills {
            serde_json::Value::String(s) => vec![s.clone()],
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            _ => vec![],
        };

        if !skill_paths.is_empty() {
            let validated: Vec<String> = skill_paths
                .iter()
                .filter(|p| plugin_path.join(p).exists())
                .map(|p| plugin_path.join(p).display().to_string())
                .collect();

            if !validated.is_empty() {
                plugin.skills_paths = Some(validated);
            }
        }
    }

    // Step 6: Process output styles from manifest
    if let Some(ref output_styles) = manifest.output_styles {
        let style_paths: Vec<String> = match output_styles {
            serde_json::Value::String(s) => vec![s.clone()],
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            _ => vec![],
        };

        if !style_paths.is_empty() {
            let validated: Vec<String> = style_paths
                .iter()
                .filter(|p| plugin_path.join(p).exists())
                .map(|p| plugin_path.join(p).display().to_string())
                .collect();

            if !validated.is_empty() {
                plugin.output_styles_paths = Some(validated);
            }
        }
    }

    // Step 7: Load hooks configuration if present
    let hooks_path = plugin_path.join("hooks").join("hooks.json");
    if hooks_path.exists() {
        match fs::read_to_string(&hooks_path) {
            Ok(content) => {
                if let Ok(hooks_config) = serde_json::from_str::<serde_json::Value>(&content) {
                    plugin.hooks_config = Some(hooks_config);
                }
            }
            Err(_) => {}
        }
    }

    Ok(plugin)
}

/// Load a single plugin from a path
///
/// This function handles loading a plugin from:
/// - A local directory (looking for manifest.json or claude_plugin.json)
/// - A git repository (clone and load)
/// - An npm package (install and load)
///
/// # Arguments
/// * `path` - The path to load the plugin from. Can be:
///   - A local directory path
///   - A git URL (https://, git@, or file://)
///   - An npm package name (optionally with version)
pub async fn load_plugin(path: &Path) -> Result<LoadedPlugin, PluginError> {
    let path_str = path.display().to_string();

    // Determine the source type and load accordingly
    let (plugin_path, source, plugin_name) = if path.is_dir() {
        // Local directory
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        (path.to_path_buf(), path_str.clone(), name)
    } else if path_str.starts_with("git@")
        || path_str.starts_with("https://")
        || path_str.starts_with("http://")
        || path_str.starts_with("file://")
    {
        // Git repository URL
        let temp_dir = std::env::temp_dir().join(format!(
            "plugin_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));

        git_clone(&path_str, &temp_dir, None, None).await?;

        // Extract plugin name from URL
        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("git-plugin")
            .to_string();

        (temp_dir, path_str.clone(), name)
    } else if !path_str.contains('/') && !path_str.contains('\\') {
        // Could be npm package name
        let temp_dir = std::env::temp_dir().join(format!(
            "npm_plugin_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));

        install_from_npm(&path_str, &temp_dir, None).await?;

        (temp_dir, format!("npm:{}", path_str), path_str.clone())
    } else {
        return Err(PluginError::GenericError {
            source: "plugin_loader".to_string(),
            plugin: None,
            error: format!("Invalid plugin path: {}", path_str),
        });
    };

    // Create the plugin from the loaded path
    create_plugin_from_path(&plugin_path, &source, true, &plugin_name).await
}

/// Load plugins from a directory
///
/// Scans the specified directory for plugin subdirectories and loads each one.
///
/// # Arguments
/// * `dir` - The directory to scan for plugins
///
/// # Returns
/// A vector of successfully loaded plugins
pub async fn load_plugins_from_dir(dir: &Path) -> Vec<LoadedPlugin> {
    let mut plugins = Vec::new();

    if !dir.exists() || !dir.is_dir() {
        return plugins;
    }

    // Read directory entries
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return plugins,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            match load_plugin(&path).await {
                Ok(plugin) => plugins.push(plugin),
                Err(e) => {
                    // Log error but continue loading other plugins
                    eprintln!("Failed to load plugin from {}: {:?}", path.display(), e);
                }
            }
        }
    }

    plugins
}

/// Load plugins from multiple sources
///
/// # Arguments
/// * `sources` - A slice of paths to load plugins from
///
/// # Returns
/// A vector of successfully loaded plugins
pub async fn load_plugins_from_sources(sources: &[PathBuf]) -> Vec<LoadedPlugin> {
    let mut plugins = Vec::new();

    for source in sources {
        match load_plugin(source).await {
            Ok(plugin) => plugins.push(plugin),
            Err(e) => {
                eprintln!("Failed to load plugin from {}: {:?}", source.display(), e);
            }
        }
    }

    plugins
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_git_url_https() {
        let result = validate_git_url("https://github.com/user/repo.git");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_git_url_ssh() {
        let result = validate_git_url("git@github.com:user/repo.git");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_git_url_invalid() {
        let result = validate_git_url("ftp://github.com/user/repo.git");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_manifest_schema_valid() {
        let manifest = PluginManifest {
            name: "test-plugin".to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("A test plugin".to_string()),
            author: None,
            homepage: None,
            repository: None,
            license: None,
            keywords: None,
            dependencies: None,
            commands: None,
            agents: None,
            skills: None,
            hooks: None,
            output_styles: None,
            channels: None,
            mcp_servers: None,
            lsp_servers: None,
            settings: None,
            user_config: None,
        };

        let result = validate_manifest_schema(&manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_manifest_schema_empty_name() {
        let manifest = PluginManifest {
            name: "".to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("A test plugin".to_string()),
            author: None,
            homepage: None,
            repository: None,
            license: None,
            keywords: None,
            dependencies: None,
            commands: None,
            agents: None,
            skills: None,
            hooks: None,
            output_styles: None,
            channels: None,
            mcp_servers: None,
            lsp_servers: None,
            settings: None,
            user_config: None,
        };

        let result = validate_manifest_schema(&manifest);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_plugin_manifest_from_file() {
        // Create a temp directory with a manifest
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("plugin.json");

        let manifest_content = r#"{
            "name": "test-plugin",
            "version": "1.0.0",
            "description": "A test plugin"
        }"#;

        fs::write(&manifest_path, manifest_content).unwrap();

        let result = load_plugin_manifest(&manifest_path);
        assert!(result.is_ok());
        let manifest = result.unwrap();
        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.version, Some("1.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_load_plugin_manifest_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("nonexistent.json");

        let result = load_plugin_manifest(&manifest_path);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_plugin_from_path_with_manifest() {
        // Create a temp directory with a manifest and commands
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path();

        // Create manifest
        let manifest_content = r#"{
            "name": "my-test-plugin",
            "version": "1.0.0",
            "description": "A test plugin"
        }"#;
        fs::write(plugin_dir.join("plugin.json"), manifest_content).unwrap();

        // Create commands directory
        fs::create_dir(plugin_dir.join("commands")).unwrap();
        fs::write(
            plugin_dir.join("commands").join("test.md"),
            "# Test Command",
        )
        .unwrap();

        let result = create_plugin_from_path(plugin_dir, "test", true, "fallback").await;
        assert!(result.is_ok());

        let plugin = result.unwrap();
        assert_eq!(plugin.name, "my-test-plugin");
        assert!(plugin.commands_path.is_some());
    }

    #[tokio::test]
    async fn test_load_plugins_from_dir_empty() {
        let temp_dir = TempDir::new().unwrap();
        let plugins = load_plugins_from_dir(temp_dir.path()).await;
        assert!(plugins.is_empty());
    }

    #[tokio::test]
    async fn test_load_plugins_from_dir_with_plugins() {
        // Create a temp directory with plugin subdirectories
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path();

        // Create first plugin
        let plugin1_dir = plugins_dir.join("plugin1");
        fs::create_dir(&plugin1_dir).unwrap();
        fs::write(plugin1_dir.join("plugin.json"), r#"{"name": "plugin1"}"#).unwrap();

        // Create second plugin
        let plugin2_dir = plugins_dir.join("plugin2");
        fs::create_dir(&plugin2_dir).unwrap();
        fs::write(plugin2_dir.join("plugin.json"), r#"{"name": "plugin2"}"#).unwrap();

        let plugins = load_plugins_from_dir(plugins_dir).await;
        assert_eq!(plugins.len(), 2);
    }
}
