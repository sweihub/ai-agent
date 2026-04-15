use std::sync::Arc;
use tokio::sync::Mutex;

pub struct DeferredMessageState {
    resolved: bool,
    pending: bool,
}

impl DeferredMessageState {
    pub fn new() -> Self {
        Self {
            resolved: false,
            pending: true,
        }
    }

    pub fn resolve(&mut self) {
        self.resolved = true;
        self.pending = false;
    }

    pub fn is_resolved(&self) -> bool {
        self.resolved
    }

    pub fn is_pending(&self) -> bool {
        self.pending
    }
}

impl Default for DeferredMessageState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DeferredMessageManager {
    states: Arc<Mutex<HashMap<String, DeferredMessageState>>>,
}

impl DeferredMessageManager {
    pub fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register(&self, id: String) {
        let mut states = self.states.lock().await;
        states.insert(id, DeferredMessageState::new());
    }

    pub async fn resolve(&self, id: &str) -> bool {
        let mut states = self.states.lock().await;
        if let Some(state) = states.get_mut(id) {
            state.resolve();
            true
        } else {
            false
        }
    }

    pub async fn is_resolved(&self, id: &str) -> bool {
        let states = self.states.lock().await;
        states.get(id).map(|s| s.is_resolved()).unwrap_or(false)
    }

    pub async fn is_pending(&self, id: &str) -> bool {
        let states = self.states.lock().await;
        states.get(id).map(|s| s.is_pending()).unwrap_or(false)
    }

    pub async fn remove(&self, id: &str) {
        let mut states = self.states.lock().await;
        states.remove(id);
    }

    pub async fn clear(&self) {
        let mut states = self.states.lock().await;
        states.clear();
    }
}

impl Default for DeferredMessageManager {
    fn default() -> Self {
        Self::new()
    }
}

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deferred_message_manager() {
        let manager = DeferredMessageManager::new();
        manager.register("test-1".to_string()).await;

        assert!(manager.is_pending("test-1").await);
        assert!(!manager.is_resolved("test-1").await);

        manager.resolve("test-1").await;

        assert!(!manager.is_pending("test-1").await);
        assert!(manager.is_resolved("test-1").await);
    }
}
