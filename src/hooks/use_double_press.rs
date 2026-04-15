// Source: ~/claudecode/openclaudecode/src/hooks/useDoublePress.ts
#![allow(dead_code)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

/// Timeout in milliseconds for detecting a double press.
pub const DOUBLE_PRESS_TIMEOUT_MS: u64 = 800;

/// Internal state for the double-press tracker.
struct DoublePressState {
    last_press_timestamp: AtomicU64,
    pending: bool,
    timeout_handle: Option<tokio::task::JoinHandle<()>>,
}

/// Creates a double-press handler that calls one function on the first press
/// and another on the second press within a certain timeout.
///
/// Translation of the React `useDoublePress` hook.
/// In Rust this is a struct that manages the press state and provides
/// an async `press` method.
pub struct DoublePress {
    state: Arc<Mutex<DoublePressState>>,
    set_pending: Box<dyn Fn(bool) + Send + Sync>,
    on_double_press: Box<dyn Fn() + Send + Sync>,
    on_first_press: Option<Box<dyn Fn() + Send + Sync>>,
}

impl DoublePress {
    /// Create a new double-press handler.
    ///
    /// - `set_pending`: callback to update the pending state
    /// - `on_double_press`: callback for the second press within the timeout
    /// - `on_first_press`: optional callback for the first press
    pub fn new(
        set_pending: impl Fn(bool) + Send + Sync + 'static,
        on_double_press: impl Fn() + Send + Sync + 'static,
        on_first_press: Option<impl Fn() + Send + Sync + 'static>,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(DoublePressState {
                last_press_timestamp: AtomicU64::new(0),
                pending: false,
                timeout_handle: None,
            })),
            set_pending: Box::new(set_pending),
            on_double_press: Box::new(on_double_press),
            on_first_press: on_first_press.map(|f| Box::new(f) as Box<dyn Fn() + Send + Sync>),
        }
    }

    /// Call this on each key press. Handles the double-press logic.
    ///
    /// Translation of the returned `useCallback` function.
    pub async fn press(&self) {
        let now = Instant::now().elapsed().as_millis() as u64;

        let is_double_press;
        let last_press;
        {
            let guard = self.state.lock().await;
            last_press = guard.last_press_timestamp.load(Ordering::Relaxed);
            let time_since_last_press = now.saturating_sub(last_press);
            is_double_press =
                time_since_last_press <= DOUBLE_PRESS_TIMEOUT_MS && guard.timeout_handle.is_some();
        }

        if is_double_press {
            // Double press detected -- cancel timeout and fire double-press handler.
            self.cancel_timeout().await;
            (self.set_pending)(false);
            (self.on_double_press)();
        } else {
            // First press.
            if let Some(ref first_press) = self.on_first_press {
                first_press();
            }
            (self.set_pending)(true);

            // Clear any existing timeout and set a new one.
            self.cancel_timeout().await;

            let state = Arc::clone(&self.state);
            let set_pending = self.set_pending.clone();
            let handle = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(DOUBLE_PRESS_TIMEOUT_MS)).await;
                set_pending(false);
                let mut guard = state.lock().await;
                guard.timeout_handle = None;
            });

            {
                let mut guard = self.state.lock().await;
                guard.timeout_handle = Some(handle);
            }
        }

        {
            let mut guard = self.state.lock().await;
            guard
                .last_press_timestamp
                .store(now, Ordering::Relaxed);
        }
    }

    /// Cancel any pending timeout.
    async fn cancel_timeout(&self) {
        let handle = {
            let mut guard = self.state.lock().await;
            guard.timeout_handle.take()
        };
        if let Some(handle) = handle {
            handle.abort();
        }
    }

    /// Synchronous cleanup of timeout (used during drop).
    fn cancel_timeout_sync(&self) {
        let mut guard = match self.state.try_lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if let Some(handle) = guard.timeout_handle.take() {
            handle.abort();
        }
    }
}

impl Drop for DoublePress {
    fn drop(&mut self) {
        // Cancel timeout on drop (equivalent to useEffect cleanup).
        self.cancel_timeout_sync();
    }
}
