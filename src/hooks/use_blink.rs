// Source: ~/claudecode/openclaudecode/src/hooks/useBlink.ts
#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use tokio::sync::broadcast;

const BLINK_INTERVAL_MS: u64 = 600;

/// Shared blink clock state -- all instances derive from this single source.
struct BlinkClock {
    /// Monotonic start of the clock.
    start: Instant,
    /// Number of active subscribers.
    subscriber_count: AtomicU64,
    /// Whether the terminal is focused (paused when blurred).
    focused: AtomicBool,
    /// Broadcast sender for tick events.
    sender: broadcast::Sender<()>,
}

static BLINK_CLOCK: LazyLock<Arc<BlinkClock>> = LazyLock::new(|| {
    let (tx, _) = broadcast::channel::<()>(1);
    Arc::new(BlinkClock {
        start: Instant::now(),
        subscriber_count: AtomicU64::new(0),
        focused: AtomicBool::new(true),
        sender: tx,
    })
});

/// Subscription handle that tracks its associated blink state.
pub struct BlinkSubscription {
    enabled: bool,
    interval_ms: u64,
}

impl BlinkSubscription {
    /// Returns true when the element should be visible in the blink cycle.
    pub fn is_visible(&self) -> bool {
        if !self.enabled || !BLINK_CLOCK.focused.load(Ordering::Relaxed) {
            return true;
        }
        let elapsed_ms = BLINK_CLOCK.start.elapsed().as_millis() as u64;
        (elapsed_ms / self.interval_ms) % 2 == 0
    }
}

impl Drop for BlinkSubscription {
    fn drop(&mut self) {
        BLINK_CLOCK
            .subscriber_count
            .fetch_sub(1, Ordering::Relaxed);
    }
}

/// Hook for synchronized blinking animations that pause when offscreen.
///
/// Returns a subscription and the current blink state.
/// All instances blink together because they derive state from the same
/// animation clock. The clock only runs when at least one subscriber is visible.
/// Pauses when the terminal is blurred.
///
/// Translation of the React `useBlink` hook.
/// In Rust this is a constructor that returns a managed subscription.
pub fn use_blink(enabled: bool, interval_ms: Option<u64>) -> BlinkSubscription {
    let interval = interval_ms.unwrap_or(BLINK_INTERVAL_MS);
    BLINK_CLOCK
        .subscriber_count
        .fetch_add(1, Ordering::Relaxed);
    BlinkSubscription {
        enabled,
        interval_ms: interval,
    }
}

/// Update the terminal focus state for the blink clock.
pub fn set_blink_clock_focused(focused: bool) {
    BLINK_CLOCK.focused.store(focused, Ordering::Relaxed);
}

/// Run the blink tick driver at the given interval.
///
/// This is the async translation of `useAnimationFrame`.
pub async fn run_blink_clock(interval_ms: u64) {
    let mut interval = tokio::time::interval(Duration::from_millis(interval_ms));
    loop {
        interval.tick().await;
        if BLINK_CLOCK.focused.load(Ordering::Relaxed)
            && BLINK_CLOCK.subscriber_count.load(Ordering::Relaxed) > 0
        {
            let _ = BLINK_CLOCK.sender.send(());
        }
    }
}
