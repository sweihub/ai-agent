// Source: Internal module — async stream interface for CLI/TUI users
// Provides futures::Stream-based event consumption for the AI Agent SDK.

use crate::types::AgentEvent;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;

// Re-use the Stream trait from futures_util (already a dependency)
use futures_util::Stream;

/// A subscriber to agent events across multiple queries.
///
/// Returned by [`crate::agent::Agent::subscribe`]. Events from the *current*
/// and *subsequent* queries flow to the subscriber until the associated
/// [`CancelGuard`] is dropped.
///
/// # Example
///
/// ```rust,ignore
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
/// Dropping this guard stops event delivery to the associated subscriber.
/// The subscriber's stream will return `None` on the next poll.
pub struct CancelGuard {
    _sender: Option<mpsc::Sender<AgentEvent>>,
}

impl CancelGuard {
    /// Create a new guard with the given sender.
    pub(crate) fn new(sender: mpsc::Sender<AgentEvent>) -> Self {
        Self {
            _sender: Some(sender),
        }
    }
}

impl Drop for CancelGuard {
    fn drop(&mut self) {
        // Drop the sender to close the channel, which will cause the receiver to return None
        self._sender.take();
    }
}
