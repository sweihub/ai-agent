// Source: /data/home/swei/claudecode/openclaudecode/src/utils/cwd.ts
use std::cell::RefCell;
use std::path::PathBuf;

thread_local! {
    static CWD_OVERRIDE: RefCell<Option<PathBuf>> = RefCell::new(None);
}

static ORIGINAL_CWD: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

pub fn run_with_cwd_override<T, F>(cwd: PathBuf, f: F) -> T
where
    F: FnOnce() -> T,
{
    CWD_OVERRIDE.with(|override_cell| {
        let old = override_cell.borrow().clone();
        *override_cell.borrow_mut() = Some(cwd);
        let result = f();
        *override_cell.borrow_mut() = old;
        result
    })
}

pub fn pwd() -> PathBuf {
    CWD_OVERRIDE
        .with(|override_cell| override_cell.borrow().clone())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

pub fn get_cwd() -> PathBuf {
    pwd().or_else(|| get_original_cwd())
}

pub fn get_original_cwd() -> PathBuf {
    ORIGINAL_CWD
        .get_or_init(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .clone()
}
