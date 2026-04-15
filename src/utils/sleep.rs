// Source: /data/home/swei/claudecode/openclaudecode/src/utils/bash/specs/sleep.ts
#![allow(dead_code)]

use std::time::Duration;
use tokio::time::sleep;

pub async fn sleep_ms(ms: u64) {
    sleep(Duration::from_millis(ms)).await;
}

pub async fn sleep_secs(secs: u64) {
    sleep(Duration::from_secs(secs)).await;
}

pub async fn with_timeout<T>(
    promise: impl std::future::Future<Output = Result<T, String>>,
    ms: u64,
    timeout_msg: &str,
) -> Result<T, String> {
    tokio::select! {
        result = promise => result,
        _ = sleep(Duration::from_millis(ms)) => Err(timeout_msg.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sleep_ms() {
        let start = std::time::Instant::now();
        sleep_ms(10).await;
        assert!(start.elapsed().as_millis() >= 10);
    }

    #[tokio::test]
    async fn test_with_timeout_success() {
        let result = with_timeout(async { Ok(42) }, 1000, "timeout").await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_with_timeout_fail() {
        let result = with_timeout(
            async {
                sleep_ms(50).await;
                Ok(42)
            },
            10,
            "timeout",
        )
        .await;
        assert!(result.is_err());
    }
}
