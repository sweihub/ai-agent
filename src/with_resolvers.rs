//! Polyfill for Promise.withResolvers() (ES2024, Node 22+).
//! Provides a promise along with its resolve and reject functions.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Mutex;

/// A simple future that can be resolved or rejected
pub struct ResolvableFuture<T> {
    state: Rc<Mutex<FutureState<T>>>,
}

enum FutureState<T> {
    Pending {
        resolve_callbacks: VecDeque<Box<dyn FnOnce(T)>>,
        reject_callbacks: VecDeque<Box<dyn FnOnce(String)>>,
    },
    Resolved(T),
    Rejected(String),
}

impl<T> ResolvableFuture<T> {
    pub fn new() -> Self {
        ResolvableFuture {
            state: Rc::new(Mutex::new(FutureState::Pending {
                resolve_callbacks: VecDeque::new(),
                reject_callbacks: VecDeque::new(),
            })),
        }
    }

    pub fn resolve(self, value: T) {
        let mut state = self.state.lock().unwrap();
        match std::mem::replace(&mut *state, FutureState::Resolved(value.clone())) {
            FutureState::Pending {
                resolve_callbacks, ..
            } => {
                for callback in resolve_callbacks {
                    callback(value.clone());
                }
            }
            _ => {}
        }
    }

    pub fn reject(self, reason: String) {
        let mut state = self.state.lock().unwrap();
        match std::mem::replace(&mut *state, FutureState::Rejected(reason.clone())) {
            FutureState::Pending {
                reject_callbacks, ..
            } => {
                for callback in reject_callbacks {
                    callback(reason.clone());
                }
            }
            _ => {}
        }
    }

    pub fn get_state(&self) -> String {
        let state = self.state.lock().unwrap();
        match &*state {
            FutureState::Pending { .. } => "pending".to_string(),
            FutureState::Resolved(_) => "resolved".to_string(),
            FutureState::Rejected(_) => "rejected".to_string(),
        }
    }
}

/// Polyfill for Promise.withResolvers()
/// Returns a promise along with its resolve and reject functions
pub fn with_resolvers<T>() -> ResolvableFuture<T> {
    ResolvableFuture::new()
}
