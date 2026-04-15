#![allow(dead_code)]

use std::collections::HashMap;

pub fn retry_with_backoff<T, E, F>(mut operation: F, max_retries: u32) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut delays = [100, 200, 400, 800, 1600];
    for i in 0..max_retries {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if i == max_retries - 1 => return Err(e),
            Err(_) => {
                if let Some(delay) = delays.get(i as usize) {
                    std::thread::sleep(std::time::Duration::from_millis(*delay));
                }
            }
        }
    }
    operation()
}
