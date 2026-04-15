use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub const DEFAULT_BLINK_INTERVAL_MS: u64 = 600;

pub struct BlinkState {
    enabled: AtomicBool,
    interval_ms: AtomicU64,
    last_toggle: AtomicU64,
}

impl BlinkState {
    pub fn new(enabled: bool, interval_ms: u64) -> Self {
        Self {
            enabled: AtomicBool::new(enabled),
            interval_ms: AtomicU64::new(interval_ms),
            last_toggle: AtomicU64::new(now_timestamp()),
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    pub fn set_interval_ms(&self, interval_ms: u64) {
        self.interval_ms.store(interval_ms, Ordering::SeqCst);
    }

    pub fn get_interval_ms(&self) -> u64 {
        self.interval_ms.load(Ordering::SeqCst)
    }

    pub fn should_show(&self) -> bool {
        if !self.is_enabled() {
            return true;
        }

        let now = now_timestamp();
        let interval = self.get_interval_ms();
        let elapsed = now.saturating_sub(self.last_toggle.load(Ordering::SeqCst));

        if elapsed >= interval {
            self.last_toggle.store(now, Ordering::SeqCst);
            return false;
        }

        true
    }
}

fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blink_state_disabled() {
        let state = BlinkState::new(false, 600);
        assert!(state.should_show());
    }

    #[test]
    fn test_blink_state_enabled() {
        let state = BlinkState::new(true, 100);
        let _ = state.should_show();
        let _ = state.should_show();
    }
}
