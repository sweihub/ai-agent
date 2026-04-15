use std::sync::Arc;

pub fn create_abort_controller(_max_listeners: usize) -> Arc<std::sync::atomic::AtomicBool> {
    Arc::new(std::sync::atomic::AtomicBool::new(false))
}

pub fn create_child_abort_controller(
    parent: &Arc<std::sync::atomic::AtomicBool>,
    _max_listeners: Option<usize>,
) -> Arc<std::sync::atomic::AtomicBool> {
    let child = Arc::new(std::sync::atomic::AtomicBool::new(false));

    if parent.load(std::sync::atomic::Ordering::SeqCst) {
        child.store(true, std::sync::atomic::Ordering::SeqCst);
        return child;
    }

    child
}
