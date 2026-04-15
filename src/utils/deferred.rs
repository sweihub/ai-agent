pub struct Deferred<T> {
    value: Option<T>,
    resolved: bool,
}

impl<T> Deferred<T> {
    pub fn new() -> Self {
        Self {
            value: None,
            resolved: false,
        }
    }

    pub fn resolve(&mut self, value: T) {
        self.value = Some(value);
        self.resolved = true;
    }

    pub fn is_resolved(&self) -> bool {
        self.resolved
    }

    pub fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn take(&mut self) -> Option<T> {
        self.value.take()
    }
}

impl<T> Default for Deferred<T> {
    fn default() -> Self {
        Self::new()
    }
}
