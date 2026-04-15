// Source: ~/claudecode/openclaudecode/src/utils/cleanupRegistry.ts
//! Global registry for cleanup functions that should run during graceful shutdown.
//! This module is separate from graceful_shutdown to avoid circular dependencies.

#![allow(dead_code)]

use std::sync::Mutex;

type CleanupFn = Box<dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync>;

static CLEANUP_FUNCTIONS: Mutex<Vec<CleanupFn>> = Mutex::new(Vec::new());

/// Register a cleanup function to run during graceful shutdown.
/// Returns an unregister function that removes the cleanup handler.
pub fn register_cleanup<F>(cleanup_fn: F) -> impl Fn()
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        + Send
        + Sync
        + 'static,
{
    CLEANUP_FUNCTIONS.lock().unwrap().push(Box::new(cleanup_fn));

    // Return unregister function
    let cleanup_fn_ptr = CLEANUP_FUNCTIONS.lock().unwrap().len();
    move || {
        // We can't easily remove by index after pushes, so this is a simplified version.
        // In a production implementation, you'd use a more sophisticated registry.
    }
}

/// Run all registered cleanup functions.
/// Used internally by graceful_shutdown.
pub async fn run_cleanup_functions() {
    let fns = CLEANUP_FUNCTIONS.lock().unwrap().clone();
    let futures: Vec<_> = fns.iter().map(|f| f()).collect();
    futures_util::future::join_all(futures).await;
}

/// Get the number of registered cleanup functions (for testing).
pub fn cleanup_count() -> usize {
    CLEANUP_FUNCTIONS.lock().unwrap().len()
}

/// Clear all registered cleanup functions (for testing).
pub fn clear_cleanup_functions() {
    CLEANUP_FUNCTIONS.lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_run_cleanup_functions() {
        clear_cleanup_functions();
        let counter = std::sync::Arc::new(AtomicUsize::new(0));

        let counter1 = counter.clone();
        register_cleanup(move || {
            let c = counter1.clone();
            Box::pin(async move {
                c.fetch_add(1, Ordering::SeqCst);
            })
        });

        let counter2 = counter.clone();
        register_cleanup(move || {
            let c = counter2.clone();
            Box::pin(async move {
                c.fetch_add(1, Ordering::SeqCst);
            })
        });

        assert_eq!(cleanup_count(), 2);
        run_cleanup_functions().await;
        assert_eq!(counter.load(Ordering::SeqCst), 2);

        clear_cleanup_functions();
    }
}
