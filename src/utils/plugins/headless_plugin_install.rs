// Source: ~/claudecode/openclaudecode/src/utils/plugins/headlessPluginInstall.ts
#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, Ordering};

/// Plugin installation for headless/CCR mode.

static INSTALLING: AtomicBool = AtomicBool::new(false);

/// Install plugins for headless/CCR mode.
/// Returns true if any plugins were installed (caller should refresh MCP).
pub async fn install_plugins_for_headless() -> Result<bool, Box<dyn std::error::Error + Send + Sync>>
{
    if !INSTALLING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        return Ok(false);
    }

    let zip_cache_mode = super::zip_cache::is_plugin_zip_cache_enabled();
    log::debug!(
        "install_plugins_for_headless: starting{}",
        if zip_cache_mode {
            " (zip cache mode)"
        } else {
            ""
        }
    );

    // Register seed marketplaces before diffing
    let seed_changed = super::marketplace_manager::register_seed_marketplaces().await?;
    if seed_changed {
        super::marketplace_manager::clear_marketplaces_cache();
        super::loader::clear_plugin_cache(Some(
            "headless_plugin_install: seed marketplaces registered",
        ));
    }

    // Ensure zip cache directory structure exists (stub)
    if zip_cache_mode {
        // Would create dirs in production
        log::debug!("Zip cache mode: would create directory structure");
    }

    let declared_count = super::marketplace_manager::get_declared_marketplaces().len();
    let mut metrics = HeadlessInstallMetrics::default();
    let mut plugins_changed = seed_changed;

    let result = (async {
        if declared_count == 0 {
            log::debug!("install_plugins_for_headless: no marketplaces declared");
        } else {
            let reconcile_result = super::reconciler::reconcile_marketplaces(
                Some(super::reconciler::ReconcileOptions {
                    skip: if zip_cache_mode {
                        Some(Box::new(|_name: &str, source: &super::schemas::MarketplaceSource| {
                            !super::zip_cache::is_marketplace_source_supported_by_zip_cache(source)
                        }))
                    } else {
                        None
                    },
                    on_progress: Some(Box::new(|event: super::reconciler::ReconcileProgressEvent| {
                        match event {
                            super::reconciler::ReconcileProgressEvent::Installed { name, .. } => {
                                log::debug!("install_plugins_for_headless: installed marketplace {}", name);
                            }
                            super::reconciler::ReconcileProgressEvent::Failed { name, error } => {
                                log::debug!("install_plugins_for_headless: failed to install marketplace {}: {}", name, error);
                            }
                            _ => {}
                        }
                    })),
                }),
            )
            .await?;

            let marketplaces_changed = reconcile_result.installed.len() + reconcile_result.updated.len();
            if marketplaces_changed > 0 {
                super::marketplace_manager::clear_marketplaces_cache();
                super::loader::clear_plugin_cache(Some("headless_plugin_install: marketplaces reconciled"));
                plugins_changed = true;
            }
            metrics.marketplaces_installed = marketplaces_changed;
        }

        // Zip cache: save marketplace JSONs for offline access
        if zip_cache_mode {
            super::zip_cache_adapters::sync_marketplaces_to_zip_cache().await?;
        }

        // Delisting enforcement
        let newly_delisted = super::plugin_blocklist::detect_and_uninstall_delisted_plugins().await?;
        metrics.delisted_count = newly_delisted.len();
        if !newly_delisted.is_empty() {
            plugins_changed = true;
        }

        if plugins_changed {
            super::loader::clear_plugin_cache(Some("headless_plugin_install: plugins changed"));
        }

        // Register session cleanup for extracted plugin temp dirs (stub)
        if zip_cache_mode {
            // Would register cleanup in production
            log::debug!("Zip cache mode: would register session cleanup");
        }

        Ok(plugins_changed)
    })
    .await;

    INSTALLING.store(false, Ordering::SeqCst);

    // Log telemetry (stub)
    // crate::services::analytics::log_event("tengu_headless_plugin_install", &serde_json::json!({
    //     "marketplaces_installed": metrics.marketplaces_installed,
    //     "delisted_count": metrics.delisted_count,
    // }));

    result
}

#[derive(Default)]
struct HeadlessInstallMetrics {
    marketplaces_installed: usize,
    delisted_count: usize,
}
