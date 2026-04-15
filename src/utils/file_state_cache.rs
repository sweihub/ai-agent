//! File state cache utilities
//! Translated from /data/home/swei/claudecode/openclaudecode/src/utils/fileStateCache.ts

use lru::LruCache;
use std::path::Path;

/// File state representing cached file content
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FileState {
    pub content: String,
    pub timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// True when this entry was populated by auto-injection (e.g. AI.md) and
    /// the injected content did not match disk (stripped HTML comments, stripped
    /// frontmatter, truncated MEMORY.md). The model has only seen a partial view;
    /// Edit/Write must require an explicit Read first. `content` here holds the
    /// RAW disk bytes (for getChangedFiles diffing), not what the model saw.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_partial_view: Option<bool>,
}

/// Default max entries for read file state caches
pub const READ_FILE_STATE_CACHE_SIZE: usize = 100;

/// Default size limit for file state caches (25MB)
/// This prevents unbounded memory growth from large file contents
pub const DEFAULT_MAX_CACHE_SIZE_BYTES: usize = 25 * 1024 * 1024;

/// A file state cache that normalizes all path keys before access.
/// This ensures consistent cache hits regardless of whether callers pass
/// relative vs absolute paths with redundant segments (e.g. /foo/../bar)
/// or mixed path separators on Windows (/ vs \).
pub struct FileStateCache {
    cache: LruCache<String, FileState>,
    max_size_bytes: usize,
}

impl FileStateCache {
    /// Create a new FileStateCache with the given max entries and max size in bytes
    pub fn new(max_entries: usize, max_size_bytes: usize) -> Self {
        Self {
            cache: LruCache::new(
                std::num::NonZeroUsize::new(max_entries)
                    .unwrap_or(std::num::NonZeroUsize::new(1).unwrap()),
            ),
            max_size_bytes,
        }
    }

    /// Get a value from the cache
    pub fn get(&mut self, key: &str) -> Option<FileState> {
        let normalized = normalize_path(key);
        self.cache.get(&normalized).cloned()
    }

    /// Set a value in the cache
    pub fn set(&mut self, key: String, value: FileState) -> &mut Self {
        let normalized = normalize_path(&key);
        self.cache.push(normalized, value);
        self
    }

    /// Check if the cache contains a key
    pub fn contains(&mut self, key: &str) -> bool {
        let normalized = normalize_path(key);
        self.cache.contains(&normalized)
    }

    /// Delete a key from the cache
    pub fn remove(&mut self, key: &str) -> Option<FileState> {
        let normalized = normalize_path(key);
        self.cache.pop(&normalized)
    }

    /// Clear all entries from the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get the current number of entries in the cache
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Get the maximum number of entries
    pub fn max_entries(&self) -> Option<usize> {
        self.cache.cap().get().try_into().ok()
    }

    /// Get the maximum size in bytes
    pub fn max_size(&self) -> usize {
        self.max_size_bytes
    }

    /// Get an iterator over the cache entries
    pub fn iter(&mut self) -> impl Iterator<Item = (&String, &FileState)> {
        self.cache.iter()
    }

    /// Get an iterator over the cache keys
    pub fn keys(&mut self) -> impl Iterator<Item = &String> {
        self.cache.iter().map(|(k, _)| k)
    }

    /// Get an iterator over the cache entries as (key, value) pairs
    pub fn entries(&mut self) -> impl Iterator<Item = (&String, &FileState)> {
        self.cache.iter()
    }
}

/// Normalize a file path for consistent cache keys
fn normalize_path(path: &str) -> String {
    // Use std::path to normalize the path
    let path_obj = Path::new(path);
    let components: Vec<String> = path_obj
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => Some(s.to_string_lossy().to_string()),
            std::path::Component::ParentDir => Some("..".to_string()),
            _ => None,
        })
        .collect();

    if components.is_empty() {
        path.to_string()
    } else {
        components.join(std::path::MAIN_SEPARATOR_STR)
    }
}

/// Factory function to create a size-limited FileStateCache.
/// Uses LRU cache's built-in size-based eviction to prevent memory bloat.
/// Note: Images are not cached (see FileReadTool) so size limit is mainly
/// for large text files, notebooks, and other editable content.
pub fn create_file_state_cache_with_size_limit(
    max_entries: usize,
    max_size_bytes: usize,
) -> FileStateCache {
    FileStateCache::new(max_entries, max_size_bytes)
}

/// Helper function to convert cache to object (used by compact.rs)
pub fn cache_to_object(cache: &mut FileStateCache) -> std::collections::HashMap<String, FileState> {
    cache.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

/// Helper function to get all keys from cache (used by several components)
pub fn cache_keys(cache: &mut FileStateCache) -> Vec<String> {
    cache.keys().cloned().collect()
}

/// Helper function to clone a FileStateCache
/// Preserves size limit configuration from the source cache
pub fn clone_file_state_cache(cache: &FileStateCache) -> FileStateCache {
    let max_entries = cache.max_entries().unwrap_or(READ_FILE_STATE_CACHE_SIZE);
    let max_size = cache.max_size();
    FileStateCache::new(max_entries, max_size)
}

/// Merge two file state caches, with more recent entries (by timestamp) overriding older ones
pub fn merge_file_state_caches(
    first: &mut FileStateCache,
    second: &mut FileStateCache,
) -> FileStateCache {
    let max_entries = first.max_entries().unwrap_or(READ_FILE_STATE_CACHE_SIZE);
    let max_size = first.max_size();
    let mut merged = FileStateCache::new(max_entries, max_size);

    for (file_path, file_state) in first.entries() {
        merged.set(file_path.clone(), file_state.clone());
    }

    for (file_path, file_state) in second.entries() {
        if let Some(existing) = merged.get(file_path) {
            // Only override if the new entry is more recent
            if file_state.timestamp > existing.timestamp {
                merged.set(file_path.clone(), file_state.clone());
            }
        } else {
            merged.set(file_path.clone(), file_state.clone());
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_state_cache_basic() {
        let mut cache = FileStateCache::new(10, 1000);

        let state = FileState {
            content: "hello".to_string(),
            timestamp: 1000,
            ..Default::default()
        };

        cache.set("test.txt".to_string(), state.clone());

        let retrieved = cache.get("test.txt");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "hello");
    }

    #[test]
    fn test_file_state_cache_normalize_path() {
        let mut cache = FileStateCache::new(10, 1000);

        let state = FileState {
            content: "hello".to_string(),
            timestamp: 1000,
            ..Default::default()
        };

        // Using normalized path should work
        cache.set("test.txt".to_string(), state.clone());

        assert!(cache.contains("test.txt"));
    }

    #[test]
    fn test_read_file_state_cache_size() {
        assert_eq!(READ_FILE_STATE_CACHE_SIZE, 100);
    }

    #[test]
    fn test_default_max_cache_size() {
        assert_eq!(DEFAULT_MAX_CACHE_SIZE_BYTES, 25 * 1024 * 1024);
    }
}
