//! Minimum display time throttle utility

use std::collections::VecDeque;

pub struct MinDisplayTimeTracker<T: Clone> {
    value: T,
    last_shown_at: u64,
    min_ms: u64,
}

impl<T: Clone> MinDisplayTimeTracker<T> {
    pub fn new(initial_value: T, min_ms: u64) -> Self {
        let now = now_timestamp();
        Self {
            value: initial_value,
            last_shown_at: now,
            min_ms,
        }
    }

    pub fn update(&mut self, new_value: T) -> T {
        let now = now_timestamp();
        let elapsed = now.saturating_sub(self.last_shown_at);

        if elapsed >= self.min_ms {
            self.last_shown_at = now;
            self.value = new_value.clone();
            new_value
        } else {
            self.value.clone()
        }
    }

    pub fn get_displayed(&self) -> &T {
        &self.value
    }
}

fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub struct ThrottleState<T> {
    queue: VecDeque<T>,
    last_update: u64,
    throttle_ms: u64,
}

impl<T> ThrottleState<T> {
    pub fn new(throttle_ms: u64) -> Self {
        Self {
            queue: VecDeque::new(),
            last_update: 0,
            throttle_ms,
        }
    }

    pub fn push(&mut self, value: T) {
        self.queue.push_back(value);
    }

    pub fn should_update(&self) -> bool {
        let now = now_timestamp();
        now.saturating_sub(self.last_update) >= self.throttle_ms
    }

    pub fn drain(&mut self) -> Option<T> {
        if self.should_update() {
            self.last_update = now_timestamp();
            self.queue.pop_front()
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_display_time_immediate() {
        let mut tracker = MinDisplayTimeTracker::new("initial".to_string(), 100);
        assert_eq!(tracker.update("first".to_string()), "first");
    }

    #[test]
    fn test_throttle_state() {
        let mut state = ThrottleState::new(1000);
        state.push("value1");
        assert!(!state.should_update());
    }
}
