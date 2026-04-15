use std::time::{Duration, Instant};

pub struct TimeoutState {
    start_time: Option<Instant>,
    duration: Duration,
}

impl TimeoutState {
    pub fn new(duration_ms: u64) -> Self {
        Self {
            start_time: Some(Instant::now()),
            duration: Duration::from_millis(duration_ms),
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(start) = self.start_time {
            start.elapsed() >= self.duration
        } else {
            false
        }
    }

    pub fn remaining_ms(&self) -> u64 {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed();
            if elapsed >= self.duration {
                0
            } else {
                (self.duration - elapsed).as_millis() as u64
            }
        } else {
            0
        }
    }

    pub fn reset(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn stop(&mut self) {
        self.start_time = None;
    }
}

pub async fn sleep_ms(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await
}

pub async fn timeout_ms<T, F>(ms: u64, future: F) -> Result<T, std::io::Error>
where
    F: std::future::Future<Output = T>,
{
    match tokio::time::timeout(Duration::from_millis(ms), future).await {
        Ok(result) => Ok(result),
        Err(_) => Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timeout_success() {
        let result = timeout_ms(1000, async { 42 }).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_timeout_expired() {
        let result = timeout_ms(10, async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            42
        })
        .await;
        assert!(result.is_err());
    }
}
