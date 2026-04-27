// Source: /data/home/swei/claudecode/openclaudecode/src/utils/memoize.ts
//! Memoization utilities with TTL and LRU support.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Cache entry with timestamp
#[derive(Clone)]
struct CacheEntry<T> {
    value: T,
    timestamp: Instant,
    refreshing: bool,
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
    pub fn new(
        f: impl Fn(Args) -> Result + Send + Sync + 'static,
        cache_lifetime_ms: u64,
    ) -> Self {
        Self {
            f: Arc::new(f),
            cache: Arc::new(Mutex::new(HashMap::new())),
            cache_lifetime_ms,
        }
    }

    pub fn call(&self, args: Args) -> Result {
        let mut cache_guard = self.cache.lock().unwrap();
        let now = Instant::now();

        if let Some(cached) = cache_guard.get(&args) {
            let age = now.duration_since(cached.timestamp).as_millis() as u64;

            if age <= self.cache_lifetime_ms {
                return cached.value.clone();
            }
        }

        let f = Arc::clone(&self.f);
        drop(cache_guard);

        let new_value = f(args.clone());

        let mut cache_guard = self.cache.lock().unwrap();
        cache_guard.insert(
            args,
            CacheEntry {
                value: new_value.clone(),
                timestamp: now,
                refreshing: false,
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

// ============================================================================
// Async memoization
// ============================================================================

/// Cache entry for async memoization, with unique id to detect concurrent clear/replace.
struct AsyncCacheEntry<T> {
    value: T,
    timestamp: Instant,
    refreshing: bool,
    id: u64,
}

impl<T> AsyncCacheEntry<T> {
    fn new(value: T, id: u64) -> Self {
        Self {
            value,
            timestamp: Instant::now(),
            refreshing: false,
            id,
        }
    }
}

struct AsyncInner<Args, Result> {
    f: Arc<
        dyn Fn(Args) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result> + Send>>
            + Send
            + Sync,
    >,
    cache: HashMap<Args, AsyncCacheEntry<Result>>,
    /// In-flight cold-miss dedup: shared slot + notify when result arrives
    in_flight:
        HashMap<Args, (Arc<Mutex<Option<Result>>>, Arc<tokio::sync::Notify>)>,
    cache_lifetime_ms: u64,
    next_id: u64,
}

/// Async memoized function with background refresh and cold-miss dedup.
pub struct AsyncMemoized<Args, Result> {
    inner: Arc<Mutex<AsyncInner<Args, Result>>>,
}

impl<Args, Result> AsyncMemoized<Args, Result>
where
    Args: Clone + std::fmt::Debug + Hash + Eq + Send + 'static,
    Result: Clone + Send + 'static,
{
    pub fn new(
        f: impl Fn(Args) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result> + Send>>
            + Send
            + Sync
            + 'static,
        cache_lifetime_ms: u64,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AsyncInner {
                f: Arc::new(f),
                cache: HashMap::new(),
                in_flight: HashMap::new(),
                cache_lifetime_ms,
                next_id: 1,
            })),
        }
    }

    pub async fn call(&self, args: Args) -> Result {
        let now = Instant::now();

        // 1. Check for in-flight dedup - another caller already computing
        let maybe_slot_notify = {
            let inner = self.inner.lock().unwrap();
            inner.in_flight.get(&args).map(|(s, n)| (s.clone(), n.clone()))
        };
        if let Some((slot, notify)) = maybe_slot_notify {
            notify.notified().await;
            if let Some(ref result) = *slot.lock().unwrap() {
                return result.clone();
            }
        }

        // 2. Check cache
        {
            let mut inner = self.inner.lock().unwrap();
            if let Some(cached) = inner.cache.get(&args) {
                let age = now.duration_since(cached.timestamp).as_millis() as u64;

                if age <= inner.cache_lifetime_ms {
                    return cached.value.clone();
                }

                // Stale - return stale value and refresh in background
                if !cached.refreshing {
                    let f = inner.f.clone();
                    let inner_arc = self.inner.clone();
                    let stale_args = args.clone();
                    let stale_id = cached.id;

                    tokio::spawn(async move {
                        let new_value = f(stale_args.clone()).await;
                        let mut c = inner_arc.lock().unwrap();
                        if let Some(entry) = c.cache.get(&stale_args) {
                            if entry.id == stale_id {
                                let id = c.next_id + 1;
                                c.next_id = id;
                                c.cache
                                    .insert(stale_args, AsyncCacheEntry::new(new_value, id));
                            }
                        }
                    });
                }

                return cached.value.clone();
            }
        }

        // 3. Cold miss - spawn task and wait
        let (slot, notify) = (
            Arc::new(Mutex::new(None)),
            Arc::new(tokio::sync::Notify::new()),
        );
        {
            let mut inner = self.inner.lock().unwrap();
            inner.in_flight
                .insert(args.clone(), (slot.clone(), notify.clone()));
        }

        let f = self.inner.lock().unwrap().f.clone();
        let inner_arc = self.inner.clone();
        let cold_args = args.clone();
        let result = f(args).await;

        // Store in shared slot for dedup
        {
            let mut s = slot.lock().unwrap();
            *s = Some(result.clone());
        }
        notify.notify_one();

        // Remove in-flight and store in cache
        {
            let mut c = inner_arc.lock().unwrap();
            c.in_flight.remove(&cold_args);
            let id = c.next_id + 1;
            c.next_id = id;
            c.cache
                .insert(cold_args, AsyncCacheEntry::new(result.clone(), id));
        }

        result
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.cache.clear();
        inner.in_flight.clear();
    }
}

/// Creates a memoized async function that returns cached values while refreshing in parallel.
pub fn memoize_with_ttl_async<Args, Result>(
    f: impl Fn(Args) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result> + Send>>
        + Send
        + Sync
        + 'static,
    cache_lifetime_ms: u64,
) -> AsyncMemoized<Args, Result>
where
    Args: Clone + std::fmt::Debug + Hash + Eq + Send + 'static,
    Result: Clone + Send + 'static,
{
    AsyncMemoized::new(f, cache_lifetime_ms)
}

// ============================================================================
// LRU memoization
// ============================================================================

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
    pub fn new(
        f: impl Fn(Args) -> Result + Send + Sync + 'static,
        key_fn: impl Fn(&Args) -> K + Send + Sync + 'static,
        max_cache_size: usize,
    ) -> Self {
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

        if let Some(value) = cache_guard.get(&key) {
            if let Some(pos) = order_guard.iter().position(|k| k == &key) {
                order_guard.remove(pos);
                order_guard.push(key.clone());
            }
            return value.clone();
        }

        let result = (self.f)(args.clone());

        if cache_guard.len() >= self.max_size && !order_guard.is_empty() {
            if let Some(lru_key) = order_guard.first().cloned() {
                cache_guard.remove(&lru_key);
                order_guard.remove(0);
            }
        }

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

        let result1 = memoized.call(1);
        assert_eq!(result1, 1);

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

        assert!(!memoized.has(&1));
    }

    #[tokio::test]
    async fn test_async_memoize_basic() {
        let counter = Arc::new(Mutex::new(0));
        let counter2 = counter.clone();
        let f = move |x: i32| {
            let counter = counter2.clone();
            let fut = Box::pin(async move {
                let mut c = counter.lock().unwrap();
                *c += 1;
                x * 2
            });
            fut as std::pin::Pin<Box<dyn std::future::Future<Output = i32> + Send>>
        };

        let memoized = memoize_with_ttl_async(f, 1000);

        let r1 = memoized.call(1).await;
        assert_eq!(r1, 2);

        let r2 = memoized.call(1).await;
        assert_eq!(r2, 2);

        // Should still be 1 (cached)
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_async_memoize_clear() {
        let counter = Arc::new(Mutex::new(0));
        let counter2 = counter.clone();
        let f = move |x: i32| {
            let counter = counter2.clone();
            let fut = Box::pin(async move {
                let mut c = counter.lock().unwrap();
                *c += 1;
                x * 2
            });
            fut as std::pin::Pin<Box<dyn std::future::Future<Output = i32> + Send>>
        };

        let memoized = memoize_with_ttl_async(f, 1000);
        assert_eq!(memoized.call(1).await, 2);
        memoized.clear();
        assert_eq!(memoized.call(1).await, 2);

        // Should be 2 after clear
        assert_eq!(*counter.lock().unwrap(), 2);
    }
}
