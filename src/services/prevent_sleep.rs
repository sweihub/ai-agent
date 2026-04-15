use once_cell::sync::Lazy;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

const CAFFEINATE_TIMEOUT_SECONDS: u64 = 300;
const RESTART_INTERVAL_MS: u64 = 4 * 60 * 1000;

static REF_COUNT: AtomicUsize = AtomicUsize::new(0);
static CAFFEINE_PROCESS: Lazy<Mutex<Option<u32>>> = Lazy::new(|| Mutex::new(None));
static CLEANUP_REGISTERED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

pub fn start_prevent_sleep() {
    REF_COUNT.fetch_add(1, Ordering::SeqCst);

    if REF_COUNT.load(Ordering::SeqCst) == 1 {
        spawn_caffeinate();
    }
}

pub fn stop_prevent_sleep() {
    if REF_COUNT.load(Ordering::SeqCst) > 0 {
        REF_COUNT.fetch_sub(1, Ordering::SeqCst);
    }

    if REF_COUNT.load(Ordering::SeqCst) == 0 {
        kill_caffeinate();
    }
}

pub fn force_stop_prevent_sleep() {
    REF_COUNT.store(0, Ordering::SeqCst);
    kill_caffeinate();
}

fn spawn_caffeinate() {
    if cfg!(not(target_os = "macos")) {
        return;
    }

    let mut process = CAFFEINE_PROCESS.lock().unwrap();
    if process.is_some() {
        return;
    }

    let mut cmd = Command::new("caffeinate");
    cmd.args(["-i", "-t", &CAFFEINATE_TIMEOUT_SECONDS.to_string()]);
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());

    match cmd.spawn() {
        Ok(child) => {
            *process = Some(child.id());
        }
        Err(_) => {
            *process = None;
        }
    }
}

fn kill_caffeinate() {
    if cfg!(not(target_os = "macos")) {
        return;
    }

    let mut process = CAFFEINE_PROCESS.lock().unwrap();

    if let Some(pid) = process.take() {
        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("kill").arg("-9").arg(pid.to_string()).output();
        }
    }
}
