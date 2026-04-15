// Source: /data/home/swei/claudecode/openclaudecode/src/utils/sequential.ts
//! Sequential execution utilities for running async functions in order.

use std::future::Future;
use std::pin::Pin;

/// Run async functions sequentially, waiting for each to complete before starting the next.
pub async fn sequential<T, R, F>(items: T, mut f: F) -> Vec<R>
where
    T: IntoIterator<Item = R>,
    F: FnMut(R) -> Pin<Box<dyn Future<Output = R> + Send>>,
    R: std::fmt::Debug,
{
    let mut results = Vec::new();

    for item in items {
        let result = f(item).await;
        results.push(result);
    }

    results
}

/// Run async functions sequentially with index, waiting for each to complete.
pub async fn sequential_with_index<T, R, F>(items: T, mut f: F) -> Vec<R>
where
    T: IntoIterator<Item = R>,
    F: FnMut(R, usize) -> Pin<Box<dyn Future<Output = R> + Send>>,
    R: std::fmt::Debug,
{
    let mut results = Vec::new();

    for (index, item) in items.into_iter().enumerate() {
        let result = f(item, index).await;
        results.push(result);
    }

    results
}
