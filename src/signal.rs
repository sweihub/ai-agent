// Source: /data/home/swei/claudecode/openclaudecode/src/utils/signal.ts
use std::collections::HashSet;

pub struct Signal<T: Clone + 'static> {
    listeners: HashSet<Box<dyn Fn(T) + Send + Sync>>,
}

impl<T: Clone + 'static> Signal<T> {
    pub fn new() -> Self {
        Self {
            listeners: HashSet::new(),
        }
    }

    pub fn subscribe<F>(&mut self, listener: F)
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        self.listeners.insert(Box::new(listener));
    }

    pub fn emit(&self, value: T) {
        for listener in &self.listeners {
            listener(value.clone());
        }
    }

    pub fn clear(&mut self) {
        self.listeners.clear();
    }
}

impl<T: Clone + 'static> Default for Signal<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_signal<T: Clone + 'static>() -> Signal<T> {
    Signal::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal() {
        let mut signal: Signal<String> = create_signal();
        let received = std::sync::Mutex::new(Vec::new());

        signal.subscribe({
            let received = received.clone();
            move |v| {
                received.lock().unwrap().push(v);
            }
        });

        signal.emit("hello".to_string());
        signal.emit("world".to_string());

        let r = received.lock().unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r[0], "hello");
        assert_eq!(r[1], "world");
    }

    #[test]
    fn test_signal_clear() {
        let mut signal: Signal<()> = create_signal();
        let called = std::sync::Mutex::new(0);

        signal.subscribe(Box::new(|_| {
            *called.lock().unwrap() += 1;
        }));

        signal.emit(());
        signal.clear();
        signal.emit(());

        assert_eq!(*called.lock().unwrap(), 1);
    }
}
