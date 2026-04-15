//! Session file access hooks.

use std::path::Path;

/// File access hook types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAccessHookType {
    PreRead,
    PostRead,
    PreWrite,
    PostWrite,
}

/// File access event
#[derive(Debug, Clone)]
pub struct FileAccessEvent {
    pub path: String,
    pub hook_type: FileAccessHookType,
    pub session_id: String,
}

/// File access hook
pub trait FileAccessHook: Send + Sync {
    fn on_access(&self, event: FileAccessEvent);
}

/// File access hook registry
pub struct FileAccessHookRegistry {
    hooks: Vec<Box<dyn FileAccessHook>>,
}

impl FileAccessHookRegistry {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    pub fn register(&mut self, hook: Box<dyn FileAccessHook>) {
        self.hooks.push(hook);
    }

    pub fn notify(&self, event: FileAccessEvent) {
        for hook in &self.hooks {
            hook.on_access(event.clone());
        }
    }
}

impl Default for FileAccessHookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a file is accessible in the current session context
pub fn is_file_accessible(path: &Path) -> bool {
    // For now, allow all file access
    // In production, this would check against session permissions
    true
}
