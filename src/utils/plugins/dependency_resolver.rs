// Source: ~/claudecode/openclaudecode/src/utils/plugins/dependencyResolver.ts
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use super::plugin_identifier::parse_plugin_identifier;
use super::types::PluginId;

/// Synthetic marketplace sentinel for --plugin-dir plugins.
const INLINE_MARKETPLACE: &str = "inline";

/// Normalize a dependency reference to fully-qualified "name@marketplace" form.
pub fn qualify_dependency(dep: &str, declaring_plugin_id: &str) -> String {
    let parsed = parse_plugin_identifier(dep);
    if parsed.marketplace.is_some() {
        return dep.to_string();
    }
    let declaring = parse_plugin_identifier(declaring_plugin_id);
    match declaring.marketplace {
        Some(ref m) if m == INLINE_MARKETPLACE => dep.to_string(),
        None => dep.to_string(),
        Some(ref mkt) => format!("{}@{}", dep, mkt),
    }
}

/// Minimal shape the resolver needs from a marketplace lookup.
pub struct DependencyLookupResult {
    pub dependencies: Option<Vec<String>>,
}

/// Result of dependency resolution.
pub enum ResolutionResult {
    Ok {
        closure: Vec<PluginId>,
    },
    Cycle {
        chain: Vec<PluginId>,
    },
    NotFound {
        missing: PluginId,
        required_by: PluginId,
    },
    CrossMarketplace {
        dependency: PluginId,
        required_by: PluginId,
    },
}

impl ResolutionResult {
    pub fn is_ok(&self) -> bool {
        matches!(self, ResolutionResult::Ok { .. })
    }
}

/// Walk the transitive dependency closure of `root_id` via DFS.
pub async fn resolve_dependency_closure<F, Fut>(
    root_id: &PluginId,
    lookup: F,
    already_enabled: &HashSet<PluginId>,
    allowed_cross_marketplaces: &HashSet<String>,
) -> ResolutionResult
where
    F: Fn(PluginId) -> Fut,
    Fut: std::future::Future<Output = Option<DependencyLookupResult>>,
{
    let root_marketplace = parse_plugin_identifier(root_id).marketplace;
    let mut closure: Vec<PluginId> = Vec::new();
    let mut visited: HashSet<PluginId> = HashSet::new();
    let mut stack: Vec<PluginId> = Vec::new();

    async fn walk<F, Fut>(
        id: PluginId,
        required_by: PluginId,
        root_id: &PluginId,
        root_marketplace: Option<&str>,
        already_enabled: &HashSet<PluginId>,
        allowed_cross_marketplaces: &HashSet<String>,
        visited: &mut HashSet<PluginId>,
        stack: &mut Vec<PluginId>,
        closure: &mut Vec<PluginId>,
        lookup: &F,
    ) -> Option<ResolutionResult>
    where
        F: Fn(PluginId) -> Fut,
        Fut: std::future::Future<Output = Option<DependencyLookupResult>>,
    {
        // Skip already-enabled dependencies, but never skip the root
        if id != *root_id && already_enabled.contains(&id) {
            return None;
        }

        // Security: block auto-install across marketplace boundaries
        let id_marketplace = parse_plugin_identifier(&id).marketplace;
        if let (Some(id_mkt), Some(root_mkt)) = (id_marketplace.as_deref(), root_marketplace) {
            if id_mkt != root_mkt && !allowed_cross_marketplaces.contains(id_mkt) {
                return Some(ResolutionResult::CrossMarketplace {
                    dependency: id.clone(),
                    required_by,
                });
            }
        }

        if stack.contains(&id) {
            return Some(ResolutionResult::Cycle {
                chain: {
                    let mut c = stack.clone();
                    c.push(id.clone());
                    c
                },
            });
        }

        if visited.contains(&id) {
            return None;
        }
        visited.insert(id.clone());

        let entry = lookup(id.clone()).await;
        let entry = match entry {
            Some(e) => e,
            None => {
                return Some(ResolutionResult::NotFound {
                    missing: id,
                    required_by,
                });
            }
        };

        stack.push(id.clone());
        for raw_dep in entry.dependencies.unwrap_or_default() {
            let dep = qualify_dependency(&raw_dep, &id);
            if let Some(err) = walk(
                dep,
                id.clone(),
                root_id,
                root_marketplace,
                already_enabled,
                allowed_cross_marketplaces,
                visited,
                stack,
                closure,
                lookup,
            )
            .await
            {
                return Some(err);
            }
        }
        stack.pop();

        closure.push(id);
        None
    }

    let result = walk(
        root_id.clone(),
        root_id.clone(),
        root_id,
        root_marketplace.as_deref(),
        already_enabled,
        allowed_cross_marketplaces,
        &mut visited,
        &mut stack,
        &mut closure,
        &lookup,
    )
    .await;

    match result {
        Some(err) => err,
        None => ResolutionResult::Ok { closure },
    }
}

/// Result from verify_and_demote: demoted plugins and their errors.
pub struct VerifyAndDemoteResult {
    pub demoted: HashSet<String>,
    pub errors: Vec<PluginError>,
}

/// Plugin error types.
pub enum PluginError {
    DependencyUnsatisfied {
        source: String,
        plugin: String,
        dependency: String,
        reason: String, // "not-enabled" or "not-found"
    },
    // Other error variants omitted for brevity
}

pub struct LoadedPlugin {
    pub source: String,
    pub enabled: bool,
    pub name: String,
    pub manifest: PluginManifest,
}

pub struct PluginManifest {
    pub dependencies: Option<Vec<String>>,
}

/// Load-time safety net: verify all manifest dependencies are also in the enabled set.
pub fn verify_and_demote(plugins: &[LoadedPlugin]) -> VerifyAndDemoteResult {
    let known: HashSet<_> = plugins.iter().map(|p| p.source.clone()).collect();
    let enabled: HashSet<_> = plugins
        .iter()
        .filter(|p| p.enabled)
        .map(|p| p.source.clone())
        .collect();

    let known_by_name: HashSet<_> = plugins
        .iter()
        .map(|p| parse_plugin_identifier(&p.source).name.clone())
        .collect();

    let mut enabled_by_name: HashMap<String, i32> = HashMap::new();
    for id in &enabled {
        let n = parse_plugin_identifier(id).name;
        *enabled_by_name.entry(n).or_insert(0) += 1;
    }

    let mut errors = Vec::new();
    let mut current_enabled = enabled.clone();
    let mut changed = true;

    while changed {
        changed = false;
        for p in plugins {
            if !current_enabled.contains(&p.source) {
                continue;
            }
            for raw_dep in p.manifest.dependencies.iter().flatten() {
                let dep = qualify_dependency(raw_dep, &p.source);
                let is_bare = parse_plugin_identifier(&dep).marketplace.is_none();
                let satisfied = if is_bare {
                    enabled_by_name.get(&dep).copied().unwrap_or(0) > 0
                } else {
                    current_enabled.contains(&dep)
                };

                if !satisfied {
                    current_enabled.remove(&p.source);
                    let count = enabled_by_name.get(&p.name).copied().unwrap_or(0);
                    if count <= 1 {
                        enabled_by_name.remove(&p.name);
                    } else {
                        enabled_by_name.insert(p.name.clone(), count - 1);
                    }
                    errors.push(PluginError::DependencyUnsatisfied {
                        source: p.source.clone(),
                        plugin: p.name.clone(),
                        dependency: dep.clone(),
                        reason: if (is_bare && known_by_name.contains(&dep)) || known.contains(&dep)
                        {
                            "not-enabled".to_string()
                        } else {
                            "not-found".to_string()
                        },
                    });
                    changed = true;
                    break;
                }
            }
        }
    }

    let demoted: HashSet<_> = plugins
        .iter()
        .filter(|p| p.enabled && !current_enabled.contains(&p.source))
        .map(|p| p.source.clone())
        .collect();

    VerifyAndDemoteResult { demoted, errors }
}

/// Find all enabled plugins that declare `plugin_id` as a dependency.
pub fn find_reverse_dependents(plugin_id: &PluginId, plugins: &[LoadedPlugin]) -> Vec<String> {
    let target_name = parse_plugin_identifier(plugin_id).name;
    plugins
        .iter()
        .filter(|p| {
            p.enabled
                && p.source != *plugin_id
                && p.manifest.dependencies.iter().flatten().any(|d| {
                    let qualified = qualify_dependency(d, &p.source);
                    if parse_plugin_identifier(&qualified).marketplace.is_some() {
                        qualified == *plugin_id
                    } else {
                        qualified == target_name
                    }
                })
        })
        .map(|p| p.name.clone())
        .collect()
}

/// Format the "(+ N dependencies)" suffix for install success messages.
pub fn format_dependency_count_suffix(installed_deps: &[String]) -> String {
    if installed_deps.is_empty() {
        return String::new();
    }
    let n = installed_deps.len();
    format!(
        " (+ {} {})",
        n,
        if n == 1 { "dependency" } else { "dependencies" }
    )
}

/// Format the "warning: required by X, Y" suffix.
pub fn format_reverse_dependents_suffix(rdeps: Option<&[String]>) -> String {
    match rdeps {
        Some(d) if !d.is_empty() => {
            format!(" — warning: required by {}", d.join(", "))
        }
        _ => String::new(),
    }
}
