// Source: /data/home/swei/claudecode/openclaudecode/src/state/store.ts
//! Simple state store implementation

use std::collections::VecDeque;
use std::fmt::Debug;
use std::marker::PhantomData;

/// Listener callback type
type Listener = Box<dyn Fn() + Send + Sync>;

/// Callback for state changes
type OnChange<T> = Box<dyn Fn(&T, &T) + Send + Sync>;

/// A simple state store
#[derive(Clone)]
pub struct Store<T: Clone + PartialEq + Send + Sync> {
    inner: std::sync::Arc<StoreInner<T>>,
}

struct StoreInner<T: Clone + PartialEq> {
    state: std::sync::Mutex<T>,
    listeners: std::sync::Mutex<VecDeque<Listener>>,
    on_change: Option<OnChange<T>>,
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Store<T> {
    /// Create a new store with initial state
    pub fn new(initial_state: T) -> Self {
        Self {
            inner: std::sync::Arc::new(StoreInner {
                state: std::sync::Mutex::new(initial_state),
                listeners: std::sync::Mutex::new(VecDeque::new()),
                on_change: None,
            }),
        }
    }

    /// Create a new store with initial state and change callback
    pub fn with_on_change(
        initial_state: T,
        on_change: impl Fn(&T, &T) + Send + Sync + 'static,
    ) -> Self {
        Self {
            inner: std::sync::Arc::new(StoreInner {
                state: std::sync::Mutex::new(initial_state),
                listeners: std::sync::Mutex::new(VecDeque::new()),
                on_change: Some(Box::new(on_change)),
            }),
        }
    }

    /// Get the current state
    pub fn get_state(&self) -> T {
        self.inner.state.lock().unwrap().clone()
    }

    /// Update the state using an updater function
    pub fn set_state(&self, updater: impl FnOnce(T) -> T + Send + Sync) {
        let mut state = self.inner.state.lock().unwrap();
        let prev = state.clone();
        let next = updater(prev.clone());

        if next == prev {
            return; // State didn't change
        }

        *state = next.clone();

        // Call on_change callback if set
        if let Some(ref callback) = self.inner.on_change {
            callback(&next, &prev);
        }

        // Notify all listeners
        let listeners = self.inner.listeners.lock().unwrap();
        for listener in listeners.iter() {
            listener();
        }
    }

    /// Subscribe to state changes
    pub fn subscribe(&self, listener: impl Fn() + Send + Sync + 'static) -> impl Fn() {
        let listener = Box::new(listener) as Listener;
        self.inner.listeners.lock().unwrap().push_back(listener);

        Box::new(move || {
            // Listener will be dropped when the returned closure is dropped
        })
    }
}

impl<T: Clone + PartialEq + Debug + Send + Sync> Debug for Store<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store")
            .field("state", &self.inner.state.lock().unwrap())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_basic() {
        let store = Store::new(42i32);
        assert_eq!(store.get_state(), 42);
    }

    #[test]
    fn test_store_set_state() {
        let store = Store::new(0i32);
        store.set_state(|_| 10);
        assert_eq!(store.get_state(), 10);
    }

    #[test]
    fn test_store_subscription() {
        let store = Store::new(0i32);
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();

        store.subscribe(move || {
            called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        });

        store.set_state(|_| 5);

        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_store_no_change() {
        let store = Store::new(42i32);
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        store.subscribe(move || {
            call_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });

        // Set to same value - should not trigger listener
        store.set_state(|s| s);

        std::thread::sleep(std::time::Duration::from_millis(10));
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[test]
    fn test_store_on_change() {
        let store = Store::with_on_change(0i32, |new_val, old_val| {
            assert_eq!(*old_val, 0);
            assert_eq!(*new_val, 42);
        });

        store.set_state(|_| 42);
        assert_eq!(store.get_state(), 42);
    }
}
