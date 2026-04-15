// Source: ~/claudecode/openclaudecode/src/utils/stream.rs

use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Mutex;

/// A stream that can be used as an async iterator.
pub struct Stream<T> {
    queue: Vec<T>,
    read_tx: Option<tokio::sync::oneshot::Sender<Result<T, StreamError>>>,
    is_done: bool,
    error: Option<StreamError>,
    started: bool,
    returned: Option<Box<dyn FnOnce() + Send>>,
}

/// Error type for stream operations.
#[derive(Debug, Clone)]
pub struct StreamError(pub String);

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StreamError {}

impl<T: Send> Stream<T> {
    /// Create a new stream.
    pub fn new(returned: Option<Box<dyn FnOnce() + Send>>) -> Self {
        Self {
            queue: Vec::new(),
            read_tx: None,
            is_done: false,
            error: None,
            started: false,
            returned,
        }
    }

    /// Get the next item from the stream.
    pub async fn next_item(&mut self) -> Option<Result<T, StreamError>> {
        if !self.queue.is_empty() {
            return Some(Ok(self.queue.remove(0)));
        }
        if self.is_done {
            return None;
        }
        if let Some(error) = self.error.take() {
            return Some(Err(error));
        }

        // Wait for a value
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.read_tx = Some(tx);

        match rx.await {
            Ok(Ok(value)) => Some(Ok(value)),
            Ok(Err(e)) => Some(Err(e)),
            Err(_) => None, // Channel cancelled
        }
    }

    /// Enqueue a value to the stream.
    pub fn enqueue(&mut self, value: T) {
        if let Some(tx) = self.read_tx.take() {
            let _ = tx.send(Ok(value));
        } else {
            self.queue.push(value);
        }
    }

    /// Mark the stream as done.
    pub fn done(&mut self) {
        self.is_done = true;
        if let Some(tx) = self.read_tx.take() {
            // Send done signal (channel will be dropped)
            drop(tx);
        }
    }

    /// Mark the stream as having an error.
    pub fn error(&mut self, error: StreamError) {
        self.error = Some(error.clone());
        if let Some(tx) = self.read_tx.take() {
            let _ = tx.send(Err(error));
        }
    }

    /// Return (close) the stream.
    pub fn return_stream(&mut self) {
        self.is_done = true;
        if let Some(returned) = self.returned.take() {
            returned();
        }
    }
}

impl<T: Send> tokio_stream::Stream for Stream<T> {
    type Item = Result<T, StreamError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !self.queue.is_empty() {
            return Poll::Ready(Some(Ok(self.queue.remove(0))));
        }
        if self.is_done {
            return Poll::Ready(None);
        }
        if let Some(error) = self.error.take() {
            return Poll::Ready(Some(Err(error)));
        }

        // We need to wait - register waker
        // This is a simplified implementation
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stream_enqueue_before_read() {
        let mut stream: Stream<i32> = Stream::new(None);
        stream.enqueue(1);
        stream.enqueue(2);

        let item = stream.next_item().await.unwrap().unwrap();
        assert_eq!(item, 1);

        let item = stream.next_item().await.unwrap().unwrap();
        assert_eq!(item, 2);
    }

    #[tokio::test]
    async fn test_stream_done() {
        let mut stream: Stream<i32> = Stream::new(None);
        stream.enqueue(1);
        stream.done();

        let item = stream.next_item().await.unwrap().unwrap();
        assert_eq!(item, 1);

        let item = stream.next_item().await;
        assert!(item.is_none());
    }
}
