// Source: ~/claudecode/openclaudecode/src/utils/plugins/reconciler.ts
#![allow(dead_code)]

use std::collections::HashMap;

use super::marketplace_manager::{
    DeclaredMarketplace, add_marketplace_source, get_declared_marketplaces,
    load_known_marketplaces_config,
};
use super::schemas::{MarketplaceSource, is_local_marketplace_source};

/// Result of comparing declared vs materialized marketplaces.
pub struct MarketplaceDiff {
    pub missing: Vec<String>,
    pub source_changed: Vec<SourceChangedEntry>,
    pub up_to_date: Vec<String>,
}

pub struct SourceChangedEntry {
    pub name: String,
    pub declared_source: MarketplaceSource,
    pub materialized_source: MarketplaceSource,
}

/// Compare declared intent (settings) against materialized state (JSON).
pub fn diff_marketplaces(
    declared: &HashMap<String, DeclaredMarketplace>,
    materialized: &HashMap<String, super::types::KnownMarketplace>,
    _project_root: Option<&str>,
) -> MarketplaceDiff {
    let mut missing = Vec::new();
    let mut source_changed = Vec::new();
    let mut up_to_date = Vec::new();

    for (name, intent) in declared {
        let state = materialized.get(name);

        match state {
            None => missing.push(name.clone()),
            Some(state_entry) => {
                if intent.source_is_fallback.unwrap_or(false) {
                    up_to_date.push(name.clone());
                } else if sources_equal_serialized(&intent.source, &state_entry.source) {
                    up_to_date.push(name.clone());
                } else {
                    let materialized_source =
                        plugin_source_to_marketplace_source(&state_entry.source);
                    source_changed.push(SourceChangedEntry {
                        name: name.clone(),
                        declared_source: intent.source.clone(),
                        materialized_source,
                    });
                }
            }
        }
    }

    MarketplaceDiff {
        missing,
        source_changed,
        up_to_date,
    }
}

/// Compare two sources by serializing to JSON for deep equality.
fn sources_equal_serialized(a: &MarketplaceSource, b: &super::types::PluginSource) -> bool {
    let a_json = serde_json::to_value(a).unwrap_or_default();
    let b_json = serde_json::to_value(b).unwrap_or_default();
    a_json == b_json
}

/// Convert a PluginSource to a MarketplaceSource for diff reporting.
fn plugin_source_to_marketplace_source(source: &super::types::PluginSource) -> MarketplaceSource {
    match source {
        super::types::PluginSource::Relative(path) => MarketplaceSource::Directory {
            path: path.clone(),
        },
        super::types::PluginSource::Github {
            repo,
            ref_,
            path,
            ..
        } => MarketplaceSource::Github {
            repo: repo.clone(),
            ref_: ref_.clone(),
            path: path.clone(),
        },
        super::types::PluginSource::Git { url, ref_, .. } => MarketplaceSource::Git {
            url: url.clone(),
            ref_: ref_.clone(),
            path: None,
        },
        super::types::PluginSource::GitSubdir {
            repo, subdir, ref_, ..
        } => MarketplaceSource::GitSubdir {
            url: repo.clone(),
            path: subdir.clone(),
            ref_: ref_.clone(),
        },
        super::types::PluginSource::Url { url, .. } => MarketplaceSource::Url {
            url: url.clone(),
        },
        super::types::PluginSource::Npm { package, .. } => MarketplaceSource::Url {
            url: format!("npm:{}", package),
        },
        super::types::PluginSource::Pip { package, .. } => MarketplaceSource::Url {
            url: format!("pip:{}", package),
        },
        super::types::PluginSource::Settings { .. } => MarketplaceSource::Settings {
            name: String::new(),
            plugins: Vec::new(),
        },
    }
}

fn _sources_equal(a: &MarketplaceSource, b: &MarketplaceSource) -> bool {
    a == b
}

fn _normalize_source(source: &MarketplaceSource, _project_root: Option<&str>) -> MarketplaceSource {
    source.clone()
}

fn _find_canonical_git_root(base: &str) -> Option<String> {
    Some(base.to_string())
}

/// Progress event for reconciliation.
pub enum ReconcileProgressEvent {
    Installing {
        name: String,
        action: String,
        index: usize,
        total: usize,
    },
    Installed {
        name: String,
        already_materialized: bool,
    },
    Failed {
        name: String,
        error: String,
    },
}

/// Options for reconciliation.
pub struct ReconcileOptions {
    pub skip: Option<Box<dyn Fn(&str, &MarketplaceSource) -> bool>>,
    pub on_progress: Option<Box<dyn Fn(ReconcileProgressEvent)>>,
}

/// Result of marketplace reconciliation.
pub struct ReconcileResult {
    pub installed: Vec<String>,
    pub updated: Vec<String>,
    pub failed: Vec<(String, String)>,
    pub up_to_date: Vec<String>,
    pub skipped: Vec<String>,
}

/// Make known_marketplaces.json consistent with declared intent.
pub async fn reconcile_marketplaces(
    opts: Option<ReconcileOptions>,
) -> Result<ReconcileResult, Box<dyn std::error::Error + Send + Sync>> {
    let declared = get_declared_marketplaces();
    if declared.is_empty() {
        return Ok(ReconcileResult {
            installed: Vec::new(),
            updated: Vec::new(),
            failed: Vec::new(),
            up_to_date: Vec::new(),
            skipped: Vec::new(),
        });
    }

    let materialized = match load_known_marketplaces_config().await {
        Ok(m) => m,
        Err(e) => {
            log::error!("Failed to load known marketplaces config: {}", e);
            HashMap::new()
        }
    };

    let diff = diff_marketplaces(&declared, &materialized, None);

    let mut work = Vec::new();

    for name in &diff.missing {
        if let Some(intent) = declared.get(name) {
            work.push((name.clone(), intent.source.clone(), "install".to_string()));
        }
    }

    for entry in diff.source_changed {
        work.push((
            entry.name.clone(),
            entry.declared_source,
            "update".to_string(),
        ));
    }

    let mut skipped = Vec::new();
    let mut to_process = Vec::new();

    for (name, source, action) in work {
        if let Some(skip_fn) = opts.as_ref().and_then(|o| o.skip.as_ref()) {
            if skip_fn(&name, &source) {
                skipped.push(name);
                continue;
            }
        }

        if action == "update" && is_local_marketplace_source(&source) {
            skipped.push(name);
            continue;
        }

        to_process.push((name, source, action));
    }

    if to_process.is_empty() {
        return Ok(ReconcileResult {
            installed: Vec::new(),
            updated: Vec::new(),
            failed: Vec::new(),
            up_to_date: diff.up_to_date,
            skipped,
        });
    }

    let mut installed = Vec::new();
    let mut updated = Vec::new();
    let mut failed = Vec::new();

    for (i, (name, source, action)) in to_process.iter().enumerate() {
        if let Some(on_progress) = opts.as_ref().and_then(|o| o.on_progress.as_ref()) {
            on_progress(ReconcileProgressEvent::Installing {
                name: name.clone(),
                action: action.clone(),
                index: i + 1,
                total: to_process.len(),
            });
        }

        match add_marketplace_source(source).await {
            Ok(result) => {
                if action == "install" {
                    installed.push(name.clone());
                } else {
                    updated.push(name.clone());
                }

                if let Some(on_progress) = opts.as_ref().and_then(|o| o.on_progress.as_ref()) {
                    on_progress(ReconcileProgressEvent::Installed {
                        name: name.clone(),
                        already_materialized: result.already_materialized,
                    });
                }
            }
            Err(e) => {
                let error = e.to_string();
                failed.push((name.clone(), error.clone()));

                if let Some(on_progress) = opts.as_ref().and_then(|o| o.on_progress.as_ref()) {
                    on_progress(ReconcileProgressEvent::Failed {
                        name: name.clone(),
                        error,
                    });
                }

                log::error!("Failed to reconcile marketplace {}: {}", name, e);
            }
        }
    }

    Ok(ReconcileResult {
        installed,
        updated,
        failed,
        up_to_date: diff.up_to_date,
        skipped,
    })
}

async fn _path_exists(source: &MarketplaceSource) -> bool {
    if let MarketplaceSource::Directory { path } | MarketplaceSource::File { path } = source {
        tokio::fs::metadata(path).await.is_ok()
    } else {
        true
    }
}
