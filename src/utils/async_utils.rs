pub struct AsyncTimer {
    duration_ms: u64,
}

impl AsyncTimer {
    pub fn new(duration_ms: u64) -> Self {
        Self { duration_ms }
    }

    pub async fn wait(&self) {
        tokio::time::sleep(std::time::Duration::from_millis(self.duration_ms)).await;
    }
}

pub struct Debouncer<T> {
    delay_ms: u64,
    pending: Option<tokio::sync::oneshot::Sender<T>>,
}

impl<T: Send + 'static> Debouncer<T> {
    pub fn new(delay_ms: u64) -> Self {
        Self {
            delay_ms,
            pending: None,
        }
    }

    pub async fn debounce<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
        F: Send + 'static,
        R: Send + 'static,
    {
        if let Some(sender) = self.pending.take() {
            let _ = sender.send(());
        }

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.pending = Some(tx);

        tokio::select! {
            _ = rx => {},
            _ = tokio::time::sleep(std::time::Duration::from_millis(self.delay_ms)) => {},
        }

        f()
    }
}
