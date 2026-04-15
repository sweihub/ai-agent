// Source: ~/claudecode/openclaudecode/src/utils/lazySchema.ts

use std::cell::OnceCell;

/// A lazy-initialized value.
/// Returns a memoized factory function that constructs the value on first call.
/// Used to defer schema construction from module init time to first access.
pub struct LazySchema<T> {
    cell: OnceCell<T>,
    factory: fn() -> T,
}

impl<T> LazySchema<T> {
    /// Create a new lazy schema with a factory function.
    pub const fn new(factory: fn() -> T) -> Self {
        Self {
            cell: OnceCell::new(),
            factory,
        }
    }

    /// Get the value, constructing it on first access.
    pub fn get(&self) -> &T {
        self.cell.get_or_init(self.factory)
    }
}

impl<T> Default for LazySchema<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            cell: OnceCell::new(),
            factory: T::default,
        }
    }
}

/// Convenience function to create a lazy schema.
pub fn lazy_schema<T>(factory: fn() -> T) -> LazySchema<T> {
    LazySchema::new(factory)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_lazy_schema() {
        static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

        fn factory() -> usize {
            CALL_COUNT.fetch_add(1, Ordering::SeqCst);
            42
        }

        let schema = lazy_schema(factory);
        assert_eq!(schema.get(), &42);
        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);

        // Second call should not invoke factory again
        assert_eq!(schema.get(), &42);
        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);
    }
}
