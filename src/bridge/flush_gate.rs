//! State machine for gating message writes during an initial flush.
//!
//! Translated from openclaudecode/src/bridge/flushGate.ts
//!
//! When a bridge session starts, historical messages are flushed to the
//! server via a single HTTP POST. During that flush, new messages must
//! be queued to prevent them from arriving at the server interleaved
//! with the historical messages.
//!
//! Lifecycle:
//!   start() -> enqueue() returns true, items are queued
//!   end()   -> returns queued items for draining, enqueue() returns false
//!   drop()  -> discards queued items (permanent transport close)
//!   deactivate() -> clears active flag without dropping items
//!                   (transport replacement - new transport will drain)

use std::mem;

/// FlushGate is a state machine for gating message writes during an initial flush.
#[derive(Debug, Clone)]
pub struct FlushGate<T> {
    active: bool,
    pending: Vec<T>,
}

impl<T> Default for FlushGate<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FlushGate<T> {
    /// Create a new FlushGate.
    pub fn new() -> Self {
        Self {
            active: false,
            pending: Vec::new(),
        }
    }

    /// Whether the flush is currently active.
    pub fn active(&self) -> bool {
        self.active
    }

    /// Number of pending items in the queue.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Mark flush as in-progress. enqueue() will start queuing items.
    pub fn start(&mut self) {
        self.active = true;
    }

    /// End the flush and return any queued items for draining.
    /// Caller is responsible for sending the returned items.
    pub fn end(&mut self) -> Vec<T> {
        self.active = false;
        mem::take(&mut self.pending)
    }

    /// If flush is active, queue the items and return true.
    /// If flush is not active, return false (caller should send directly).
    pub fn enqueue(&mut self, items: Vec<T>) -> bool {
        if !self.active {
            return false;
        }
        self.pending.extend(items);
        true
    }

    /// Enqueue a single item.
    pub fn enqueue_one(&mut self, item: T) -> bool {
        if !self.active {
            return false;
        }
        self.pending.push(item);
        true
    }

    /// Discard all queued items (permanent transport close).
    /// Returns the number of items dropped.
    pub fn drop(&mut self) -> usize {
        self.active = false;
        let count = self.pending.len();
        self.pending.clear();
        count
    }

    /// Clear the active flag without dropping queued items.
    /// Used when the transport is replaced - the new
    /// transport's flush will drain the pending items.
    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flush_gate_basic() {
        let mut gate: FlushGate<i32> = FlushGate::new();

        // Initially not active
        assert!(!gate.active());
        assert!(!gate.enqueue_one(1));

        // Start flush
        gate.start();
        assert!(gate.active());
        assert!(gate.enqueue_one(1));
        assert!(gate.enqueue(vec![2, 3]));
        assert_eq!(gate.pending_count(), 3);

        // End flush
        let items = gate.end();
        assert!(!gate.active());
        assert_eq!(items, vec![1, 2, 3]);
        assert_eq!(gate.pending_count(), 0);

        // After end, enqueue should return false
        assert!(!gate.enqueue_one(4));
    }

    #[test]
    fn test_flush_gate_drop() {
        let mut gate: FlushGate<i32> = FlushGate::new();
        gate.start();
        gate.enqueue(vec![1, 2, 3]);

        let dropped = gate.drop();
        assert_eq!(dropped, 3);
        assert!(!gate.active());
        assert_eq!(gate.pending_count(), 0);
    }

    #[test]
    fn test_flush_gate_deactivate() {
        let mut gate: FlushGate<i32> = FlushGate::new();
        gate.start();
        gate.enqueue(vec![1, 2, 3]);

        gate.deactivate();
        assert!(!gate.active());
        // Pending items should remain
        assert_eq!(gate.pending_count(), 3);
    }
}
