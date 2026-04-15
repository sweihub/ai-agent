use std::sync::RwLock;

static LISTENER: RwLock<Option<fn(String, CommandLifecycleState)>> = RwLock::new(None);

#[derive(Debug, Clone, Copy)]
pub enum CommandLifecycleState {
    Started,
    Completed,
}

pub fn set_command_lifecycle_listener(cb: Option<fn(String, CommandLifecycleState)>) {
    let mut listener = LISTENER.write().unwrap();
    *listener = cb;
}

pub fn notify_command_lifecycle(uuid: &str, state: CommandLifecycleState) {
    let listener = LISTENER.read().unwrap();
    if let Some(cb) = *listener {
        cb(uuid.to_string(), state);
    }
}
