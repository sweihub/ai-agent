use std::sync::atomic::{AtomicBool, Ordering};

static SINK_KILLSWITCH: AtomicBool = AtomicBool::new(false);

pub fn is_sink_killswitch_active() -> bool {
    SINK_KILLSWITCH.load(Ordering::SeqCst)
}

pub fn set_sink_killswitch(_active: bool) {
    SINK_KILLSWITCH.store(_active, Ordering::SeqCst);
}
