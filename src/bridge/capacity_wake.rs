//! Shared capacity-wake primitive for bridge poll loops.
//!
//! Translated from openclaudecode/src/bridge/capacityWake.ts
//!
//! Both replBridge.ts and bridgeMain.ts need to sleep while "at capacity"
//! but wake early when either (a) the outer loop signal aborts (shutdown),
//! or (b) capacity frees up (session done / transport lost). This module
//! encapsulates the mutable wake-controller + two-signal merger that both
//! poll loops previously duplicated byte-for-byte.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Capacity signal with cleanup
pub struct CapacitySignal {
    pub signal: Arc<AtomicBool>,
    pub cleanup: Box<dyn Fn() + Send + Sync>,
}

/// Capacity wake controller
pub struct CapacityWake {
    /// Create a signal that aborts when either the outer loop signal or the
    /// capacity-wake controller fires. Returns the merged signal and a cleanup
    /// function that removes listeners when the sleep resolves normally
    /// (without abort).
    signal: Arc<AtomicBool>,
    /// Abort the current at-capacity sleep and arm a fresh controller so the
    /// poll loop immediately re-checks for new work.
    wake: Arc<AtomicBool>,
    /// Outer signal (from the loop that owns this capacity wake)
    outer_signal: Arc<AtomicBool>,
    /// Whether the current signal has been armed
    armed: Arc<AtomicBool>,
}

impl CapacityWake {
    /// Create a new capacity wake with the given outer signal.
    pub fn new(outer_signal: Arc<AtomicBool>) -> Self {
        Self {
            signal: Arc::new(AtomicBool::new(false)),
            wake: Arc::new(AtomicBool::new(false)),
            outer_signal,
            armed: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get the capacity signal. When triggered (from either outer abort
    /// or wake() call), the poll loop should re-check for work.
    pub fn get_signal(&self) -> CapacitySignal {
        // Check if already triggered
        let triggered =
            self.outer_signal.load(Ordering::SeqCst) || self.wake.load(Ordering::SeqCst);

        // Reset wake flag after reading (so subsequent calls return untriggered)
        if self.wake.load(Ordering::SeqCst) {
            self.wake.store(false, Ordering::SeqCst);
        }

        // Arm the signal
        self.armed.store(true, Ordering::SeqCst);
        // Reset signal for next wait cycle
        self.signal.store(false, Ordering::SeqCst);

        CapacitySignal {
            signal: if triggered {
                Arc::new(AtomicBool::new(true))
            } else {
                Arc::clone(&self.signal)
            },
            cleanup: Box::new(move || {
                // Cleanup is handled by drop
            }),
        }
    }

    /// Wake up the capacity wait. This causes get_signal() to return
    /// a triggered signal, and re-arms for the next wait.
    pub fn wake(&self) {
        // Set wake trigger - get_signal() will check this and return triggered
        self.wake.store(true, Ordering::SeqCst);
    }

    /// Check if the outer signal has been triggered
    pub fn is_outer_aborted(&self) -> bool {
        self.outer_signal.load(Ordering::SeqCst)
    }
}

/// Create a capacity wake primitive for bridge poll loops.
pub fn create_capacity_wake(outer_signal: Arc<AtomicBool>) -> CapacityWake {
    CapacityWake::new(outer_signal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capacity_wake_basic() {
        let outer = Arc::new(AtomicBool::new(false));
        let wake = create_capacity_wake(Arc::clone(&outer));

        // Initially not triggered
        let signal = wake.get_signal();
        assert!(!signal.signal.load(Ordering::SeqCst));

        // Wake should trigger
        wake.wake();
        let signal2 = wake.get_signal();
        // After wake, next signal should be triggered
        assert!(signal2.signal.load(Ordering::SeqCst));
    }

    #[test]
    fn test_capacity_wake_outer_abort() {
        let outer = Arc::new(AtomicBool::new(false));
        let wake = create_capacity_wake(Arc::clone(&outer));

        // Trigger outer
        outer.store(true, Ordering::SeqCst);

        // Should return triggered signal
        let signal = wake.get_signal();
        assert!(signal.signal.load(Ordering::SeqCst));
    }
}
