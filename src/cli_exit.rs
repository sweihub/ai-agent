#![allow(dead_code)]

pub fn exit_with_code(code: i32) -> ! {
    std::process::exit(code)
}

pub fn handle_exit(force: bool) {
    if force {
        exit_with_code(0);
    }
}
