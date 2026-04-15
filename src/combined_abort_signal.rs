use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct CombinedAbortSignal {
    aborted: Arc<AtomicBool>,
}

impl CombinedAbortSignal {
    pub fn new() -> Self {
        Self {
            aborted: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn abort(&self) {
        self.aborted.store(true, Ordering::SeqCst);
    }

    pub fn is_aborted(&self) -> bool {
        self.aborted.load(Ordering::SeqCst)
    }
}

impl Default for CombinedAbortSignal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "tokio")]
impl CombinedAbortSignal {
    pub fn to_tokio_signal(&self) -> tokio::sync::watch::Receiver<bool> {
        let (tx, rx) = tokio::sync::watch::channel(self.is_aborted());
        if self.is_aborted() {
            let _ = tx.send(true);
        }
        rx
    }
}
