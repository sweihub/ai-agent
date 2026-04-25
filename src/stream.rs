// Source: Internal module — async stream interface for CLI/TUI users
// Provides futures::Stream-based event consumption for the AI Agent SDK.

use crate::types::AgentEvent;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;

// Re-use the Stream trait from futures_util (already a dependency)
use futures_util::Stream;
use std::sync::Arc;

/// Manage a collection of event channel senders for broadcasting agent events
/// to multiple subscribers.
///
/// Stores `Vec<mpsc::Sender<AgentEvent>>` protected by a `parking_lot::Mutex`.
/// `broadcast()` sends an event to all active subscribers and removes closed
/// channels. `subscribe()` creates a new (`EventSubscriber`, `CancelGuard`) pair.
#[derive(Clone)]
pub struct EventBroadcasters {
    senders: Arc<parking_lot::Mutex<Vec<mpsc::Sender<AgentEvent>>>>,
}

impl EventBroadcasters {
    pub fn new() -> Self {
        Self {
            senders: Arc::new(parking_lot::Mutex::new(Vec::new())),
        }
    }

    /// Send an event to all active subscribers.
    ///
    /// Iterates all channel senders, attempting to send the event to each.
    /// Removes (and drops) senders whose receiver has been disconnected,
    /// keeping the list lean.
    pub fn broadcast(&self, event: &AgentEvent) {
        let mut senders = self.senders.lock();
        senders.retain(|tx| tx.try_send(event.clone()).is_ok());
    }

    /// Subscribe to events.
    ///
    /// Creates a new channel and registers the sender. Returns an
    /// `EventSubscriber` (the receiver) and a `CancelGuard` that will
    /// automatically remove the sender when dropped.
    pub fn subscribe(&self) -> (EventSubscriber, CancelGuard) {
        let (tx, rx) = mpsc::channel(256);
        let index = {
            let mut senders = self.senders.lock();
            let index = senders.len();
            senders.push(tx);
            index
        };
        let guard = CancelGuard::new(Arc::clone(&self.senders), index);
        (EventSubscriber::new(rx), guard)
    }
}

impl Default for EventBroadcasters {
    fn default() -> Self {
        Self::new()
    }
}

/// A subscriber to agent events across multiple queries.
///
/// Returned by [`crate::agent::Agent::subscribe`]. Events from the *current*
/// and *subsequent* queries flow to the subscriber until the associated
/// [`CancelGuard`] is dropped.
///
/// # Example
///
/// ```rust,ignore
/// let agent = Agent::new("claude-sonnet-4-6");
/// let (mut sub, _guard) = agent.subscribe();
///
/// tokio::spawn(async move {
///     agent.query("hello").await;
/// });
///
/// while let Some(ev) = sub.next().await {
///     // render in TUI
/// }
/// ```
pub struct EventSubscriber {
    receiver: mpsc::Receiver<AgentEvent>,
}

impl EventSubscriber {
    pub(crate) fn new(receiver: mpsc::Receiver<AgentEvent>) -> Self {
        Self { receiver }
    }
}

impl Stream for EventSubscriber {
    type Item = AgentEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.receiver).poll_recv(cx) {
            Poll::Ready(Some(event)) => Poll::Ready(Some(event)),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Token that unsubscribes the [`EventSubscriber`] when dropped.
///
/// Dropping this guard removes the sender from the [`EventBroadcasters`],
/// which closes the channel and causes the subscriber's stream to return `None`.
pub struct CancelGuard {
    senders: Option<Arc<parking_lot::Mutex<Vec<mpsc::Sender<AgentEvent>>>>>,
    index: usize,
}

impl CancelGuard {
    /// Create a new guard that will remove the sender at `index` when dropped.
    pub(crate) fn new(
        senders: Arc<parking_lot::Mutex<Vec<mpsc::Sender<AgentEvent>>>>,
        index: usize,
    ) -> Self {
        Self {
            senders: Some(senders),
            index,
        }
    }
}

impl Drop for CancelGuard {
    fn drop(&mut self) {
        if let Some(ref senders) = self.senders {
            let mut s = senders.lock();
            if self.index < s.len() {
                s.remove(self.index);
            }
        }
    }
}
