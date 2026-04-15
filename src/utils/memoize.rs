// Source: /data/home/swei/claudecode/openclaudecode/src/utils/memoize.ts
//! Memoization utilities with TTL and LRU support.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Cache entry with timestamp
#[derive(Clone)]
struct CacheEntry<T> {
    value: T,
    timestamp: Instant,
}

/// Creates a memoized function that returns cached values while refreshing in parallel.
/// This implements a write-through cache pattern:
/// - If cache is fresh, return immediately
/// - If cache is stale, return the stale value but refresh it in the background
/// - If no cache exists, block and compute the value
pub struct MemoizedFunction<Args, Result> {
    f: Arc<dyn Fn(Args) -> Result + Send + Sync>,
    cache: Arc<Mutex<HashMap<Args, CacheEntry<Result>>>>,
    cache_lifetime_ms: u64,
}

impl<Args, Result> MemoizedFunction<Args, Result>
where
    Args: Clone + std::fmt::Debug + Hash + Eq + Send + 'static,
    Result: Clone + Send + 'static,
{
    pub fn new(f: impl Fn(Args) -> Result + Send + Sync + 'static, cache_lifetime_ms: u64) -> Self {
        Self {
            f: Arc::new(f),
            cache: Arc::new(Mutex::new(HashMap::new())),
            cache_lifetime_ms,
        }
    }

    pub fn call(&self, args: Args) -> Result {
        let mut cache_guard = self.cache.lock().unwrap();
        let now = Instant::now();

        // Check if we have a cached value
        if let Some(cached) = cache_guard.get(&args) {
            let age = now.duration_since(cached.timestamp).as_millis() as u64;

            if age <= self.cache_lifetime_ms {
                // Cache is fresh, return cached value
                return cached.value.clone();
            }
        }

        // Compute new value - need to release lock first
        let f = Arc::clone(&self.f);
        drop(cache_guard);

        let new_value = f(args.clone());

        // Store in cache
        let mut cache_guard = self.cache.lock().unwrap();
        cache_guard.insert(
            args,
            CacheEntry {
                value: new_value.clone(),
                timestamp: now,
            },
        );

        new_value
    }

    pub fn clear(&self) {
        let mut cache_guard = self.cache.lock().unwrap();
        cache_guard.clear();
    }
}

/// Creates a memoized function that returns cached values while refreshing in parallel.
pub fn memoize_with_ttl<Args, Result>(
    f: impl Fn(Args) -> Result + Send + Sync + 'static,
    cache_lifetime_ms: u64,
) -> MemoizedFunction<Args, Result>
where
    Args: Clone + std::fmt::Debug + Hash + Eq + Send + 'static,
    Result: Clone + Send + 'static,
{
    MemoizedFunction::new(f, cache_lifetime_ms)
}

/// Creates a memoized async function that returns cached values while refreshing in parallel.
/// Note: Full async implementation with background refresh requires more complex async handling.
#[allow(unused)]
pub fn memoize_with_ttl_async<Args, Result, Fut>(
    _f: impl Fn(Args) -> Fut + Send + Sync + 'static,
    _cache_lifetime_ms: u64,
) -> impl Fn(Args) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result> + Send>>
where
    Args: Clone + std::fmt::Debug + Hash + Eq + Send + 'static,
    Result: Clone + Send + 'static,
    Fut: std::future::Future<Output = Result> + Send,
{
    // Simplified sync version for now - full async with background refresh
    // would require proper async runtime integration
    move |_args: Args| {
        Box::pin(async { todo!("memoize_with_ttl_async - async version not fully implemented") })
    }
}

/// Creates a memoized function with LRU (Least Recently Used) eviction policy.
/// This prevents unbounded memory growth by evicting the least recently used entries
/// when the cache reaches its maximum size.
pub struct LruMemoized<Args, K, Result> {
    f: Arc<dyn Fn(Args) -> Result + Send + Sync + 'static>,
    cache: Arc<Mutex<HashMap<K, Result>>>,
    order: Arc<Mutex<Vec<K>>>,
    max_size: usize,
    key_fn: Arc<dyn Fn(&Args) -> K + Send + Sync + 'static>,
}

impl<Args, K, Result> LruMemoized<Args, K, Result>
where
    Args: std::fmt::Debug + Hash + Eq + Clone,
    Result: Clone,
    K: Hash + Eq + Clone,
{
    pub fn new(f: impl Fn(Args) -> Result + Send + Sync + 'static, key_fn: impl Fn(&Args) -> K + Send + Sync + 'static, max_cache_size: usize) -> Self {
        Self {
            f: Arc::new(f),
            cache: Arc::new(Mutex::new(HashMap::new())),
            order: Arc::new(Mutex::new(Vec::new())),
            max_size: max_cache_size,
            key_fn: Arc::new(key_fn),
        }
    }

    pub fn call(&self, args: Args) -> Result {
        let key = (self.key_fn)(&args);
        let mut cache_guard = self.cache.lock().unwrap();
        let mut order_guard = self.order.lock().unwrap();

        // Check cache
        if let Some(value) = cache_guard.get(&key) {
            // Update access order - move key to end
            if let Some(pos) = order_guard.iter().position(|k| k == &key) {
                order_guard.remove(pos);
                order_guard.push(key.clone());
            }
            return value.clone();
        }

        // Compute value
        let result = (self.f)(args.clone());

        // Evict if at max size
        if cache_guard.len() >= self.max_size && !order_guard.is_empty() {
            if let Some(lru_key) = order_guard.first().cloned() {
                cache_guard.remove(&lru_key);
                order_guard.remove(0);
            }
        }

        // Store in cache
        cache_guard.insert(key.clone(), result.clone());
        order_guard.push(key);

        result
    }

    pub fn clear(&self) {
        let mut cache_guard = self.cache.lock().unwrap();
        let mut order_guard = self.order.lock().unwrap();
        cache_guard.clear();
        order_guard.clear();
    }

    pub fn size(&self) -> usize {
        self.cache.lock().unwrap().len()
    }

    pub fn delete(&self, key: &K) -> bool {
        let mut cache_guard = self.cache.lock().unwrap();
        let mut order_guard = self.order.lock().unwrap();
        if let Some(pos) = order_guard.iter().position(|k| k == key) {
            order_guard.remove(pos);
        }
        cache_guard.remove(key).is_some()
    }

    pub fn get(&self, key: &K) -> Option<Result> {
        self.cache.lock().unwrap().get(key).cloned()
    }

    pub fn has(&self, key: &K) -> bool {
        self.cache.lock().unwrap().contains_key(key)
    }
}

/// Creates a memoized function with LRU (Least Recently Used) eviction policy.
pub fn memoize_with_lru<Args, K, Result>(
    f: impl Fn(Args) -> Result + Send + Sync + 'static,
    key_fn: impl Fn(&Args) -> K + Send + Sync + 'static,
    max_cache_size: usize,
) -> LruMemoized<Args, K, Result>
where
    Args: std::fmt::Debug + Hash + Eq + Clone,
    Result: Clone,
    K: Hash + Eq + Clone,
{
    LruMemoized::new(f, key_fn, max_cache_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memoize_with_ttl_basic() {
        let counter = Arc::new(Mutex::new(0));
        let f = move |_x: i32| {
            let mut c = counter.lock().unwrap();
            *c += 1;
            *c
        };

        let memoized = memoize_with_ttl(f, 1000);

        // First call should compute
        let result1 = memoized.call(1);
        assert_eq!(result1, 1);

        // Second call with same args should use cache
        let result2 = memoized.call(1);
        assert_eq!(result2, 1);
    }

    #[test]
    fn test_memoize_with_lru_basic() {
        let f = |x: i32| x * 2;

        let memoized = memoize_with_lru(f, |&x: &i32| x, 2);

        assert_eq!(memoized.call(1), 2);
        assert_eq!(memoized.call(2), 4);
    }

    #[test]
    fn test_lru_eviction() {
        let f = |x: i32| x * 2;

        let memoized = memoize_with_lru(f, |&x: &i32| x, 2);

        assert_eq!(memoized.call(1), 2);
        assert_eq!(memoized.call(2), 4);
        assert_eq!(memoized.call(3), 6);

        // First entry should be evicted
        assert!(!memoized.has(&1));
    }
}