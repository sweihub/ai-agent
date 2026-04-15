//! System prompt sections module
//!
//! Provides functions for creating and managing system prompt sections
//! that can be cached or computed on each turn.

use std::collections::HashMap;

/// A function that computes a prompt section value
pub type ComputeFn = Box<dyn Fn() -> Option<String> + Send + Sync>;

/// A system prompt section with name, compute function, and cache behavior
pub struct SystemPromptSection {
    pub name: String,
    pub compute: ComputeFn,
    pub cache_break: bool,
}

/// Create a memoized system prompt section.
/// Computed once, cached until /clear or /compact.
pub fn system_prompt_section(name: &str, compute: ComputeFn) -> SystemPromptSection {
    SystemPromptSection {
        name: name.to_string(),
        compute,
        cache_break: false,
    }
}

/// Create a volatile system prompt section that recomputes every turn.
/// This WILL break the prompt cache when the value changes.
/// Requires a reason explaining why cache-breaking is necessary.
pub fn dangerous_uncached_system_prompt_section(
    name: &str,
    compute: ComputeFn,
    _reason: &str,
) -> SystemPromptSection {
    SystemPromptSection {
        name: name.to_string(),
        compute,
        cache_break: true,
    }
}

/// Resolve all system prompt sections, returning prompt strings.
/// Uses a cache to avoid recomputing sections unnecessarily.
pub fn resolve_system_prompt_sections(
    sections: &[SystemPromptSection],
    cache: &mut HashMap<String, Option<String>>,
) -> Vec<Option<String>> {
    sections
        .iter()
        .map(|s| {
            if !s.cache_break {
                if let Some(cached) = cache.get(&s.name) {
                    return cached.clone();
                }
            }
            let value = (s.compute)();
            cache.insert(s.name.clone(), value.clone());
            value
        })
        .collect()
}

/// Clear all system prompt section state.
/// Called on /clear and /compact.
pub fn clear_system_prompt_sections(_cache: &mut HashMap<String, Option<String>>) {
    // Clear the cache
    _cache.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt_section() {
        let compute = Box::new(|| Some("test prompt".to_string()));
        let section = system_prompt_section("test", compute);

        assert_eq!(section.name, "test");
        assert!(!section.cache_break);
    }

    #[test]
    fn test_uncached_section() {
        let compute = Box::new(|| Some("test prompt".to_string()));
        let section =
            dangerous_uncached_system_prompt_section("test", compute, "needs fresh value");

        assert_eq!(section.name, "test");
        assert!(section.cache_break);
    }

    #[test]
    fn test_resolve_with_cache() {
        let compute = Box::new(|| Some("computed value".to_string()));
        let section = system_prompt_section("test", compute);

        let mut cache = HashMap::new();
        let results = resolve_system_prompt_sections(&[section], &mut cache);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], Some("computed value".to_string()));
    }

    #[test]
    fn test_cache_hit() {
        let compute = Box::new(|| Some("new value".to_string()));
        let section = system_prompt_section("test", compute);

        let mut cache = HashMap::new();
        cache.insert("test".to_string(), Some("cached value".to_string()));

        let results = resolve_system_prompt_sections(&[section], &mut cache);

        assert_eq!(results[0], Some("cached value".to_string()));
    }
}
