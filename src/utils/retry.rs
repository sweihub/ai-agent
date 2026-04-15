#![allow(dead_code)]

use std::collections::HashMap;

pub fn retry_with_backoff<F, T, E>(mut op: F, max_retries: u32) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut attempts = 0;
    loop {
        match op() {
            Ok(v) => return Ok(v),
            Err(e) if attempts >= max_retries => return Err(e),
            Err(_) => {
                attempts += 1;
                let delay = 2u64.pow(attempts) * 100;
                std::thread::sleep(std::time::Duration::from_millis(delay));
            }
        }
    }
}

pub fn retry_async_with_backoff<F, T, E>(
    op: F,
    max_retries: u32,
) -> impl std::future::Future<Output = Result<T, E>>
where
    F: std::future::Future<Output = Result<T, E>>,
{
    async {
        let mut attempts = 0;
        loop {
            match op.await {
                Ok(v) => return Ok(v),
                Err(e) if attempts >= max_retries => return Err(e),
                Err(_) => {
                    attempts += 1;
                    let delay = 2u64.pow(attempts) * 100;
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                }
            }
        }
    }
}
