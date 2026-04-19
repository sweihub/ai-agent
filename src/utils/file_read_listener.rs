// Source: ~/claudecode/openclaudecode/src/tools/FileReadTool/FileReadTool.ts
//! File read listener utilities for detecting magic doc headers when files are read.
//! Provides a listener pattern for registering callbacks that fire when files are read.

use std::sync::{Arc, Mutex};

/// Callback type for file read events
pub type FileReadListener = Arc<
    dyn Fn(&str, &str) + Send + Sync,
>;

lazy_static::lazy_static! {
    static ref FILE_READ_LISTENERS: Arc<Mutex<Vec<FileReadListener>>> =
        Arc::new(Mutex::new(Vec::new()));
}

/// Register a listener that fires when a file is read.
/// The listener receives the file path and content.
pub fn register_file_read_listener(listener: FileReadListener) {
    let mut listeners = FILE_READ_LISTENERS.lock().unwrap();
    listeners.push(listener);
}

/// Notify all registered listeners about a file read.
/// Call this when a file is successfully read to fire the listeners.
pub fn notify_file_read_listeners(file_path: &str, content: &str) {
    let listeners = FILE_READ_LISTENERS.lock().unwrap();
    for listener in listeners.iter() {
        listener(file_path, content);
    }
}

/// Clear all registered listeners (for testing)
pub fn clear_file_read_listeners() {
    let mut listeners = FILE_READ_LISTENERS.lock().unwrap();
    listeners.clear();
}

/// Get the number of registered listeners
pub fn get_file_read_listener_count() -> usize {
    FILE_READ_LISTENERS.lock().unwrap().len()
}
