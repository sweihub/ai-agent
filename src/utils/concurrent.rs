// Source: /data/home/swei/claudecode/openclaudecode/src/utils/generators.ts
//! Concurrent execution utilities for running multiple async operations with a concurrency limit.
//!
//! Translated from TypeScript generators.ts

use std::sync::Arc;
use tokio::sync::Mutex;

/// Type alias for boxed futures
pub type BoxFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>;

/// Simple concurrent batch that spawns all tasks at once
/// Returns results in input order
pub async fn join_all_concurrent<T, R, F>(items: Vec<T>, f: F) -> Vec<R>
where
    T: Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> BoxFuture<R> + Send + Clone + 'static,
{
    if items.is_empty() {
        return vec![];
    }

    let handles: Vec<_> = items
        .into_iter()
        .map(|item| {
            let f = f.clone();
            tokio::spawn(async move { f(item).await })
        })
        .collect();

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn double(x: i32) -> BoxFuture<i32> {
        Box::pin(async move { x * 2 })
    }

    #[tokio::test]
    async fn test_join_all_concurrent() {
        let items = vec![1, 2, 3];
        let result = join_all_concurrent(items, double).await;
        assert_eq!(result, vec![2, 4, 6]);
    }
}
