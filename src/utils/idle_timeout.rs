//! Idle timeout management utilities
//!
//! Creates an idle timeout manager for SDK mode.
//! Automatically exits the process after the specified idle duration.

use crate::constants::env::ai_code;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Creates an idle timeout manager for SDK mode.
///
/// # Arguments
/// * `is_idle` - Function that returns true if the system is currently idle
///
/// # Returns
/// Object with start/stop methods to control the idle timer
pub fn create_idle_timeout_manager<F>(is_idle: F) -> IdleTimeoutManager
where
    F: Fn() -> bool + Send + Sync + 'static,
{
    let exit_after_stop_delay = std::env::var(ai_code::EXIT_AFTER_STOP_DELAY)
        .ok()
        .and_then(|v| v.parse::<u64>().ok());

    let delay_ms = exit_after_stop_delay.filter(|&d| d > 0);

    IdleTimeoutManager {
        is_idle: Arc::new(is_idle),
        delay_ms,
        timer: Mutex::new(None),
    }
}

/// Idle timeout manager
pub struct IdleTimeoutManager {
    is_idle: Arc<dyn Fn() -> bool + Send + Sync + 'static>,
    delay_ms: Option<u64>,
    timer: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl IdleTimeoutManager {
    /// Start the idle timer
    pub fn start(&self) {
        let delay_ms = match self.delay_ms {
            Some(d) => d,
            None => return, // No delay configured
        };

        // Clear any existing timer
        self.stop();

        let is_idle_fn = Arc::clone(&self.is_idle);
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let timer = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;

            let idle_duration = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
                - start_time;

            if is_idle_fn() && idle_duration >= delay_ms {
                log::info!("Exiting after {}ms of idle time", delay_ms);
                std::process::exit(0);
            }
        });

        *self.timer.lock().unwrap() = Some(timer);
    }

    /// Stop the idle timer
    pub fn stop(&self) {
        if let Some(timer) = self.timer.lock().unwrap().take() {
            timer.abort();
        }
    }
}

impl Drop for IdleTimeoutManager {
    fn drop(&mut self) {
        self.stop();
    }
}
