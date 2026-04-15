pub struct EventEmitter<T> {
    listeners: Vec<Box<dyn Fn(T) + Send + Sync>>,
}

impl<T: Send + 'static> EventEmitter<T> {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    pub fn on<F>(&mut self, callback: F)
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        self.listeners.push(Box::new(callback));
    }

    pub fn emit(&self, event: T) {
        for listener in &self.listeners {
            listener(event.clone());
        }
    }

    pub fn clear(&mut self) {
        self.listeners.clear();
    }

    pub fn listener_count(&self) -> usize {
        self.listeners.len()
    }
}

impl<T> Default for EventEmitter<T> {
    fn default() -> Self {
        Self::new()
    }
}
