use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub const HINT_COOLDOWN_MS: u64 = 30000;
pub const FOCUS_CHECK_DEBOUNCE_MS: u64 = 1000;

pub struct ClipboardImageHintState {
    enabled: AtomicBool,
    last_hint_time: AtomicU64,
    last_focused: AtomicBool,
}

impl ClipboardImageHintState {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled: AtomicBool::new(enabled),
            last_hint_time: AtomicU64::new(0),
            last_focused: AtomicBool::new(false),
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    pub fn on_focus_changed(&self, is_focused: bool) -> bool {
        if !self.is_enabled() {
            return false;
        }

        let was_focused = self.last_focused.load(Ordering::SeqCst);
        self.last_focused.store(is_focused, Ordering::SeqCst);

        if was_focused || !is_focused {
            return false;
        }

        let now = now_timestamp();
        let last_hint = self.last_hint_time.load(Ordering::SeqCst);

        if now.saturating_sub(last_hint) < HINT_COOLDOWN_MS {
            return false;
        }

        true
    }

    pub fn mark_hint_shown(&self) {
        self.last_hint_time.store(now_timestamp(), Ordering::SeqCst);
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
    fn test_clipboard_image_hint_state() {
        let state = ClipboardImageHintState::new(true);

        assert!(!state.on_focus_changed(true));
        assert!(state.on_focus_changed(false));
    }

    #[test]
    fn test_cooldown() {
        let state = ClipboardImageHintState::new(true);

        state.on_focus_changed(true);
        state.mark_hint_shown();

        assert!(!state.on_focus_changed(true));
    }
}
