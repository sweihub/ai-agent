/// Global mutex that serializes tests touching shared global mutable state.
/// With ~1300 tests across 5 threads, any test that mutates a OnceLock<Mutex<...>>
/// (TASK_STORE, TODOS, CRON_JOBS, TEAMS, INBOX, CONFIG, LOADED_SKILLS, etc.)
/// can trample another test mid-flight.
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

/// Acquire the global test serialization lock.
pub fn get_serialization_lock() -> std::sync::MutexGuard<'static, ()> {
    TEST_MUTEX.lock().unwrap()
}
