use std::sync::atomic::{AtomicBool, Ordering};

static SYNC_CACHE_DIRTY: AtomicBool = AtomicBool::new(false);

pub fn is_sync_cache_dirty() -> bool {
    SYNC_CACHE_DIRTY.load(Ordering::SeqCst)
}

pub fn set_sync_cache_dirty(_dirty: bool) {
    SYNC_CACHE_DIRTY.store(_dirty, Ordering::SeqCst);
}
