// Source: /data/home/swei/claudecode/openclaudecode/src/utils/bash/specs/sleep.ts
use std::time::Duration;

#[cfg(feature = "tokio")]
pub async fn sleep(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}

#[cfg(feature = "tokio")]
pub async fn sleep_with_abort(ms: u64, abort: &tokio::sync::watch::Receiver<bool>) -> bool {
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_millis(ms)) => false,
        _ = abort.changed() => abort.get().copied().unwrap_or(false),
    }
}

#[cfg(feature = "tokio")]
pub async fn with_timeout<T, F>(future: F, ms: u64, message: &str) -> Result<T, String>
where
    F: Future<Output = T>,
{
    tokio::time::timeout(Duration::from_millis(ms), future)
        .await
        .map_err(|_| message.to_string())
}

#[cfg(not(feature = "tokio"))]
pub fn sleep_sync(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}
