pub fn initiate_graceful_shutdown() {
    std::process::exit(0);
}

pub fn is_shutdown_requested() -> bool {
    false
}

pub fn set_shutdown_handler(_handler: impl Fn()) {}
