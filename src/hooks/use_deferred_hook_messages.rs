// Source: ~/claudecode/openclaudecode/src/hooks/useDeferredHookMessages.ts
#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::Mutex;

/// A deferred hook message that will be resolved from a promise.
pub trait HookResultMessage: Send + Sync + Clone + 'static {}
impl<T: Send + Sync + Clone + 'static> HookResultMessage for T {}

/// A message type used by the session system.
pub trait Message: Send + Sync + Clone + 'static {}
impl<T: Send + Sync + Clone + 'static> Message for T {}

/// Internal state for deferred hook messages.
struct DeferredState<M: Message> {
    /// The pending promise, set to None once resolved or cancelled.
    pending: Option<tokio::sync::oneshot::Receiver<Vec<Box<dyn HookResultMessage>>>>,
    /// Whether the promise has already been resolved.
    resolved: AtomicBool,
}

/// Manages deferred SessionStart hook messages so the REPL can render
/// immediately instead of blocking on hook execution (~500ms).
///
/// Hook messages are injected asynchronously when the promise resolves.
/// Returns a callback that `on_submit` should call before the first API
/// request to ensure the model always sees hook context.
///
/// Translation of the React `useDeferredHookMessages` hook.
/// In Rust this is a struct that holds the pending state and provides
/// an async `wait_for_messages` method.
pub struct DeferredHookMessages<M: Message> {
    state: Arc<Mutex<DeferredState<M>>>,
    set_messages: Arc<dyn Fn(Vec<Box<dyn HookResultMessage>>) + Send + Sync>,
}

impl<M: Message> DeferredHookMessages<M> {
    /// Create a new deferred hook messages manager.
    ///
    /// - `pending_messages`: an optional oneshot receiver that will resolve to
    ///   the hook messages.
    /// - `set_messages`: callback to prepend messages to the message list.
    pub fn new(
        pending_messages: Option<tokio::sync::oneshot::Receiver<Vec<Box<dyn HookResultMessage>>>>,
        set_messages: impl Fn(Vec<Box<dyn HookResultMessage>>) + Send + Sync + 'static,
    ) -> Self {
        let state = Arc::new(Mutex::new(DeferredState {
            pending: pending_messages,
            resolved: AtomicBool::new(pending_messages.is_none()),
        }));
        let set_messages = Arc::new(set_messages);

        // Start background task to handle the promise (equivalent to useEffect).
        {
            let state = Arc::clone(&state);
            let set_messages = Arc::clone(&set_messages);
            tokio::spawn(async move {
                let msgs = {
                    let mut guard = state.lock().await;
                    guard.pending.take()
                };
                if let Some(rx) = msgs {
                    if let Ok(msgs) = rx.await {
                        let already_resolved = state.lock().await.resolved.load(Ordering::SeqCst);
                        if already_resolved {
                            return;
                        }
                        state.lock().await.resolved.store(true, Ordering::SeqCst);
                        if !msgs.is_empty() {
                            set_messages(msgs);
                        }
                    }
                }
            });
        }

        Self { state, set_messages }
    }

    /// Await this to ensure hook messages are resolved before the first API request.
    ///
    /// Translation of the returned `useCallback` async function.
    pub async fn wait_for_messages(&self) {
        let already_resolved = {
            let guard = self.state.lock().await;
            guard.resolved.load(Ordering::SeqCst)
        };
        if already_resolved {
            return;
        }

        let rx = {
            let mut guard = self.state.lock().await;
            guard.pending.take()
        };

        if let Some(rx) = rx {
            if let Ok(msgs) = rx.await {
                let already_resolved = {
                    let guard = self.state.lock().await;
                    guard.resolved.load(Ordering::SeqCst)
                };
                if already_resolved {
                    return;
                }
                {
                    let mut guard = self.state.lock().await;
                    guard.resolved.store(true, Ordering::SeqCst);
                    guard.pending = None;
                }
                if !msgs.is_empty() {
                    (self.set_messages)(msgs);
                }
            }
        }
    }
}
