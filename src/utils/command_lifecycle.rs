// Source: ~/claudecode/openclaudecode/src/utils/commandLifecycle.ts
//! Command lifecycle state tracking.
//! Tracks started/completed state transitions for commands.

#![allow(dead_code)]

use std::cell::RefCell;

#[derive(Clone, Debug)]
pub enum CommandLifecycleState {
    Started,
    Completed,
}

type ListenerFn = Box<dyn Fn(String, CommandLifecycleState)>;

thread_local! {
    static LISTENER: RefCell<Option<ListenerFn>> = const { RefCell::new(None) };
}

/// Set the command lifecycle listener.
pub fn set_command_lifecycle_listener(cb: Option<ListenerFn>) {
    LISTENER.with(|l| *l.borrow_mut() = cb);
}

/// Notify the command lifecycle listener of a state change.
pub fn notify_command_lifecycle(uuid: String, state: CommandLifecycleState) {
    LISTENER.with(|l| {
        if let Some(ref cb) = *l.borrow() {
            cb(uuid, state);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn test_set_and_notify() {
        let called = Rc::new(Cell::new(false));
        let called_clone = called.clone();
        let cb = Box::new(move |_uuid: String, _state: CommandLifecycleState| {
            called_clone.set(true);
        });
        set_command_lifecycle_listener(Some(cb));
        notify_command_lifecycle("test-uuid".to_string(), CommandLifecycleState::Started);
        assert!(called.get());

        // Clean up
        set_command_lifecycle_listener(None);
    }
}
