//! Watch file system for changes

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

/// File system watcher for monitoring changes
pub struct FileWatcher {
    watcher: RecommendedWatcher,
    receiver: Receiver<Result<Event, notify::Error>>,
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new() -> Result<Self, notify::Error> {
        let (tx, rx) = channel();

        let watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )?;

        Ok(Self {
            watcher,
            receiver: rx,
        })
    }

    /// Watch a file or directory
    pub fn watch(&mut self, path: impl AsRef<Path>) -> Result<(), notify::Error> {
        self.watcher.watch(path.as_ref(), RecursiveMode::Recursive)
    }

    /// Unwatch a path
    pub fn unwatch(&mut self, path: impl AsRef<Path>) -> Result<(), notify::Error> {
        self.watcher.unwatch(path.as_ref())
    }

    /// Get the next event (non-blocking)
    pub fn try_recv(&self) -> Option<Result<Event, notify::Error>> {
        self.receiver.try_recv().ok()
    }

    /// Get the next event (blocking)
    pub fn recv(&self) -> Result<Event, notify::Error> {
        self.receiver
            .recv()
            .map_err(|e| notify::Error::generic(&e.to_string()))?
    }
}

/// File change event types
#[derive(Debug, Clone)]
pub enum FileEvent {
    Created(String),
    Modified(String),
    Deleted(String),
    Renamed(String, String),
}

impl From<Event> for FileEvent {
    fn from(event: Event) -> Self {
        let paths: Vec<String> = event
            .paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        match event.kind {
            notify::EventKind::Create(_) => {
                FileEvent::Created(paths.first().cloned().unwrap_or_default())
            }
            notify::EventKind::Modify(_) => {
                FileEvent::Modified(paths.first().cloned().unwrap_or_default())
            }
            notify::EventKind::Remove(_) => {
                FileEvent::Deleted(paths.first().cloned().unwrap_or_default())
            }
            notify::EventKind::Other => {
                FileEvent::Modified(paths.first().cloned().unwrap_or_default())
            }
            _ => FileEvent::Modified(paths.first().cloned().unwrap_or_default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_watcher() {
        // This would require actual file system operations
        // skipping for unit tests
    }
}
