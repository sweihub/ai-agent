// Source: ~/claudecode/openclaudecode/src/utils/plugins/reconciler.ts
#![allow(dead_code)]

use std::collections::HashMap;

use super::marketplace_manager::{
    add_marketplace_source, get_declared_marketplaces, load_known_marketplaces_config,
    DeclaredMarketplace,
};
use super::schemas::{is_local_marketplace_source, MarketplaceSource};

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
                } else {
                    // Simplified comparison - stub
                    source_changed.push(SourceChangedEntry {
                        name: name.clone(),
                        declared_source: intent.source.clone(),
                        materialized_source: MarketplaceSource::Url { url: String::new() },
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
pub async fn reconcile_marketplaces(opts: Option<ReconcileOptions>) -> Result<ReconcileResult, Box<dyn std::error::Error + Send + Sync>> {
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
        work.push((entry.name.clone(), entry.declared_source, "update".to_string()));
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

        if action == "update"
            && is_local_marketplace_source(&source)
        {
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
