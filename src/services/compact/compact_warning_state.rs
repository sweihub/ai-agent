use std::sync::atomic::{AtomicBool, Ordering};

static COMPACT_WARNING_STORE: AtomicBool = AtomicBool::new(false);

pub fn suppress_compact_warning() {
    COMPACT_WARNING_STORE.store(true, Ordering::SeqCst);
}

pub fn clear_compact_warning_suppression() {
    COMPACT_WARNING_STORE.store(false, Ordering::SeqCst);
}
