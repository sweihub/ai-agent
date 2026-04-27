// Source: ~/claudecode/openclaudecode/src/utils/plugins/loadPluginOutputStyles.ts
#![allow(dead_code)]

use std::collections::HashSet;
use std::path::Path;
use std::sync::Mutex;

use once_cell::sync::Lazy;

use super::frontmatter_parser::parse_frontmatter;
use super::loader::load_all_plugins_cache_only;
use super::walk_plugin_markdown::{WalkPluginMarkdownOpts, walk_plugin_markdown};

static OUTPUT_STYLE_CACHE: Lazy<Mutex<Option<Vec<OutputStyleConfig>>>> =
    Lazy::new(|| Mutex::new(None));

/// Output style configuration loaded from a plugin.
#[derive(Clone, Debug)]
pub struct OutputStyleConfig {
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub source: String,
    pub force_for_plugin: Option<bool>,
}

/// Load output styles from a directory.
async fn load_output_styles_from_directory(
    output_styles_path: &Path,
    plugin_name: &str,
    loaded_paths: &mut HashSet<String>,
) -> Result<Vec<OutputStyleConfig>, Box<dyn std::error::Error + Send + Sync>> {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let styles: Arc<Mutex<Vec<OutputStyleConfig>>> = Arc::new(Mutex::new(Vec::new()));

    walk_plugin_markdown(
        output_styles_path,
        |full_path, _namespace| {
            let plugin_name = plugin_name.to_string();
            let styles = Arc::clone(&styles);

            Box::pin(async move {
                match load_output_style_from_file(&full_path, &plugin_name, &mut HashSet::new())
                    .await
                {
                    Ok(Some(style)) => styles.lock().await.push(style),
                    _ => {}
                }
            })
        },
        WalkPluginMarkdownOpts {
            stop_at_skill_dir: Some(false),
            log_label: Some("output-styles".to_string()),
        },
    )
    .await
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    Ok(Arc::try_unwrap(styles).unwrap().into_inner())
}

/// Load a single output style from a file.
async fn load_output_style_from_file(
    file_path: &str,
    plugin_name: &str,
    loaded_paths: &mut HashSet<String>,
) -> Result<Option<OutputStyleConfig>, Box<dyn std::error::Error + Send + Sync>> {
    if loaded_paths.contains(file_path) {
        return Ok(None);
    }
    loaded_paths.insert(file_path.to_string());

    let content = tokio::fs::read_to_string(file_path)
        .await
        .map_err(|e| format!("Failed to read {}: {}", file_path, e))?;
    let (frontmatter, markdown_content) = parse_frontmatter(&content, file_path);

    let file_name = std::path::Path::new(file_path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let base_style_name = frontmatter
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&file_name);

    let name = format!("{}:{}", plugin_name, base_style_name);

    let description = frontmatter
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or(&format!("Output style from {} plugin", plugin_name))
        .to_string();

    let force_for_plugin = frontmatter.get("force-for-plugin").and_then(|v| match v {
        serde_json::Value::Bool(b) => Some(*b),
        serde_json::Value::String(s) => match s.as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        },
        _ => None,
    });

    Ok(Some(OutputStyleConfig {
        name,
        description,
        prompt: markdown_content.trim().to_string(),
        source: "plugin".to_string(),
        force_for_plugin,
    }))
}

/// Load plugin output styles from all enabled plugins.
pub async fn load_plugin_output_styles()
-> Result<Vec<OutputStyleConfig>, Box<dyn std::error::Error + Send + Sync>> {
    {
        let cache = OUTPUT_STYLE_CACHE.lock().unwrap();
        if let Some(ref styles) = *cache {
            return Ok(styles.clone());
        }
    }

    let plugin_result = load_all_plugins_cache_only().await?;
    let mut all_styles = Vec::new();

    for plugin in &plugin_result.enabled {
        let mut loaded_paths = HashSet::new();

        // Load from default output-styles directory
        if let Some(ref output_styles_path) = plugin.output_styles_path {
            match load_output_styles_from_directory(
                Path::new(output_styles_path),
                &plugin.name,
                &mut loaded_paths,
            )
            .await
            {
                Ok(styles) => {
                    log::debug!(
                        "Loaded {} output styles from plugin {} default directory",
                        styles.len(),
                        plugin.name
                    );
                    all_styles.extend(styles);
                }
                Err(e) => log::debug!(
                    "Failed to load output styles from plugin {} default directory: {}",
                    plugin.name,
                    e
                ),
            }
        }

        // Load from additional paths
        if let Some(ref output_styles_paths) = plugin.output_styles_paths {
            for style_path in output_styles_paths {
                let metadata = match tokio::fs::metadata(style_path).await {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                if metadata.is_dir() {
                    if let Ok(styles) = load_output_styles_from_directory(
                        Path::new(style_path),
                        &plugin.name,
                        &mut loaded_paths,
                    )
                    .await
                    {
                        all_styles.extend(styles);
                    }
                } else if metadata.is_file()
                    && Path::new(style_path)
                        .extension()
                        .map(|e| e.to_string_lossy() == "md")
                        .unwrap_or(false)
                {
                    if let Ok(Some(style)) =
                        load_output_style_from_file(style_path, &plugin.name, &mut loaded_paths)
                            .await
                    {
                        all_styles.push(style);
                    }
                }
            }
        }
    }

    log::debug!("Total plugin output styles loaded: {}", all_styles.len());

    {
        let mut cache = OUTPUT_STYLE_CACHE.lock().unwrap();
        *cache = Some(all_styles.clone());
    }

    Ok(all_styles)
}

/// Clear the plugin output style cache.
pub fn clear_plugin_output_style_cache() {
    let mut cache = OUTPUT_STYLE_CACHE.lock().unwrap();
    *cache = None;
}
