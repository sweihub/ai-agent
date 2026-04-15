//! Native installer utilities.

use crate::constants::env::system;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::utils::platform::detect_platform as get_platform;

lazy_static::lazy_static! {
    static ref OS_RELEASE_CACHE: Mutex<Option<OsRelease>> = Mutex::new(None);
    static ref PACKAGE_MANAGER_CACHE: Mutex<Option<PackageManager>> = Mutex::new(None);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionLockContent {
    pub pid: u32,
    pub version: String,
    pub exec_path: String,
    pub acquired_at: u64,
}

#[derive(Debug, Clone)]
pub struct LockInfo {
    pub version: String,
    pub pid: u32,
    pub is_process_running: bool,
    pub exec_path: String,
    pub acquired_at: SystemTime,
    pub lock_file_path: String,
}

#[derive(Debug, Clone)]
pub struct OsRelease {
    pub id: String,
    pub id_like: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Homebrew,
    Winget,
    Pacman,
    Deb,
    Rpm,
    Apk,
    Mise,
    Asdf,
    Unknown,
}

impl PackageManager {
    pub fn as_str(&self) -> &'static str {
        match self {
            PackageManager::Homebrew => "homebrew",
            PackageManager::Winget => "winget",
            PackageManager::Pacman => "pacman",
            PackageManager::Deb => "deb",
            PackageManager::Rpm => "rpm",
            PackageManager::Apk => "apk",
            PackageManager::Mise => "mise",
            PackageManager::Asdf => "asdf",
            PackageManager::Unknown => "unknown",
        }
    }
}

const FALLBACK_STALE_MS: u64 = 2 * 60 * 60 * 1000;

pub fn is_pid_based_locking_enabled() -> bool {
    if let Ok(val) = std::env::var(system::ENABLE_PID_BASED_VERSION_LOCKING) {
        let val_lower = val.to_lowercase();
        if val_lower == "true" || val_lower == "1" || val_lower == "yes" {
            return true;
        }
        if val_lower == "false" || val_lower == "0" || val_lower == "no" || val_lower.is_empty() {
            return false;
        }
    }
    false
}

#[cfg(unix)]
pub fn is_process_running(pid: u32) -> bool {
    if pid <= 1 {
        return false;
    }
    unsafe {
        let result = libc::kill(pid as libc::pid_t, 0);
        result == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM as i32)
    }
}

#[cfg(not(unix))]
pub fn is_process_running(pid: u32) -> bool {
    if pid <= 1 {
        return false;
    }
    pid == std::process::id()
}

fn is_claude_process(pid: u32, expected_exec_path: &str) -> bool {
    if !is_process_running(pid) {
        return false;
    }
    if pid == std::process::id() {
        return true;
    }
    true
}

pub fn read_lock_content(lock_file_path: &str) -> Option<VersionLockContent> {
    let path = Path::new(lock_file_path);
    match std::fs::read_to_string(path) {
        Ok(content) => {
            if content.trim().is_empty() {
                return None;
            }
            match serde_json::from_str::<VersionLockContent>(&content) {
                Ok(parsed) => {
                    if parsed.version.is_empty() || parsed.exec_path.is_empty() {
                        None
                    } else {
                        Some(parsed)
                    }
                }
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

pub fn is_lock_active(lock_file_path: &str) -> bool {
    let content = match read_lock_content(lock_file_path) {
        Some(c) => c,
        None => return false,
    };

    let pid = content.pid;

    if !is_process_running(pid) {
        return false;
    }

    if !is_claude_process(pid, &content.exec_path) {
        return false;
    }

    let path = Path::new(lock_file_path);
    if let Ok(metadata) = std::fs::metadata(path) {
        if let Ok(modified) = metadata.modified() {
            let age = SystemTime::now()
                .duration_since(modified)
                .unwrap_or(std::time::Duration::ZERO)
                .as_millis() as u64;

            if age > FALLBACK_STALE_MS {
                if !is_process_running(pid) {
                    return false;
                }
            }
        }
    }

    true
}

pub fn try_acquire_lock(
    version_path: &str,
    lock_file_path: &str,
) -> Option<Box<dyn FnOnce() + Send>> {
    let version_name = Path::new(version_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    if is_lock_active(lock_file_path) {
        if read_lock_content(lock_file_path).is_some() {
            return None;
        }
    }

    let lock_content = VersionLockContent {
        pid: std::process::id(),
        version: version_name.clone(),
        exec_path: std::env::current_exe()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_default(),
        acquired_at: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    };

    let json = match serde_json::to_string_pretty(&lock_content) {
        Ok(j) => j,
        Err(_) => return None,
    };

    if let Err(_) = std::fs::write(lock_file_path, &json) {
        return None;
    }

    if let Some(verify_content) = read_lock_content(lock_file_path) {
        if verify_content.pid != std::process::id() {
            return None;
        }
    } else {
        return None;
    }

    let lock_path = lock_file_path.to_string();
    Some(Box::new(move || {
        if let Some(current_content) = read_lock_content(&lock_path) {
            if current_content.pid == std::process::id() {
                let _ = std::fs::remove_file(&lock_path);
            }
        }
    }))
}

pub fn acquire_process_lifetime_lock(version_path: &str, lock_file_path: &str) -> bool {
    if let Some(release) = try_acquire_lock(version_path, lock_file_path) {
        std::mem::forget(release);
        true
    } else {
        false
    }
}

pub fn with_lock<F>(version_path: &str, lock_file_path: &str, callback: F) -> bool
where
    F: FnOnce() + Send,
{
    if let Some(_release) = try_acquire_lock(version_path, lock_file_path) {
        callback();
        true
    } else {
        false
    }
}

pub fn get_all_lock_info(locks_dir: &str) -> Vec<LockInfo> {
    let mut lock_infos = Vec::new();
    let path = Path::new(locks_dir);

    if !path.is_dir() {
        return lock_infos;
    }

    if let Ok(lock_files) = std::fs::read_dir(path) {
        for lock_entry in lock_files.flatten() {
            let lock_file_path = lock_entry.path();
            if let Some(file_name) = lock_file_path.file_name() {
                if file_name.to_string_lossy().ends_with(".lock") {
                    if let Some(content) = read_lock_content(&lock_file_path.to_string_lossy()) {
                        lock_infos.push(LockInfo {
                            version: content.version,
                            pid: content.pid,
                            is_process_running: is_process_running(content.pid),
                            exec_path: content.exec_path,
                            acquired_at: SystemTime::UNIX_EPOCH
                                + std::time::Duration::from_millis(content.acquired_at),
                            lock_file_path: lock_file_path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    lock_infos
}

pub fn cleanup_stale_locks(locks_dir: &str) -> usize {
    let path = Path::new(locks_dir);
    let mut cleaned_count = 0;

    if !path.is_dir() {
        return 0;
    }

    if let Ok(lock_entries) = std::fs::read_dir(path) {
        for lock_entry in lock_entries.flatten() {
            let lock_path = lock_entry.path();
            if let Some(file_name) = lock_path.file_name() {
                if file_name.to_string_lossy().ends_with(".lock") {
                    if lock_path.is_dir() {
                        if std::fs::remove_dir_all(&lock_path).is_ok() {
                            cleaned_count += 1;
                        }
                    } else if !is_lock_active(&lock_path.to_string_lossy()) {
                        if std::fs::remove_file(&lock_path).is_ok() {
                            cleaned_count += 1;
                        }
                    }
                }
            }
        }
    }

    cleaned_count
}

fn is_distro_family(os_release: &OsRelease, families: &[&str]) -> bool {
    if families.iter().any(|f| *f == os_release.id) {
        return true;
    }
    os_release
        .id_like
        .iter()
        .any(|like| families.contains(&like.as_str()))
}

pub fn get_os_release() -> Option<OsRelease> {
    let mut cache = OS_RELEASE_CACHE.lock().ok()?;
    if cache.is_some() {
        return cache.clone();
    }

    let content = std::fs::read_to_string("/etc/os-release").ok()?;

    let mut id = String::new();
    let mut id_like = Vec::new();

    for line in content.lines() {
        if let Some(prefix) = line.strip_prefix("ID=") {
            id = prefix.trim_matches('"').trim_matches('\'').to_string();
        }
        if let Some(prefix) = line.strip_prefix("ID_LIKE=") {
            let value = prefix.trim_matches('"').trim_matches('\'');
            id_like = value.split_whitespace().map(|s| s.to_string()).collect();
        }
    }

    if id.is_empty() {
        return None;
    }

    let result = OsRelease { id, id_like };
    *cache = Some(result.clone());
    Some(result)
}

pub fn detect_mise() -> bool {
    let exec_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_default();

    exec_path.contains("/mise/installs/") || exec_path.contains("\\mise\\installs\\")
}

pub fn detect_asdf() -> bool {
    let exec_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_default();

    exec_path.contains("/.asdf/installs/")
        || exec_path.contains("\\.asdf\\installs\\")
        || exec_path.contains("/asdf/installs/")
        || exec_path.contains("\\asdf\\installs\\")
}

pub fn detect_homebrew() -> bool {
    let platform = get_platform();

    if platform != "macos" && platform != "linux" && platform != "wsl" {
        return false;
    }

    let exec_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_default();

    exec_path.contains("/Caskroom/")
}

pub fn detect_winget() -> bool {
    let platform = get_platform();

    if platform != "windows" {
        return false;
    }

    let exec_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_default();

    exec_path.contains("Microsoft\\WinGet\\Packages")
        || exec_path.contains("Microsoft/WinGet/Packages")
        || exec_path.contains("Microsoft\\WinGet\\Links")
        || exec_path.contains("Microsoft/WinGet/Links")
}

pub async fn detect_pacman() -> bool {
    let platform = get_platform();

    if platform != "linux" {
        return false;
    }

    if let Some(os_release) = get_os_release() {
        if !is_distro_family(&os_release, &["arch"]) {
            return false;
        }
    }

    let exec_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let output = tokio::process::Command::new("pacman")
        .args(["-Qo", &exec_path])
        .output()
        .await;

    match output {
        Ok(o) => o.status.success() && !o.stdout.is_empty(),
        Err(_) => false,
    }
}

pub async fn detect_deb() -> bool {
    let platform = get_platform();

    if platform != "linux" {
        return false;
    }

    if let Some(os_release) = get_os_release() {
        if !is_distro_family(&os_release, &["debian"]) {
            return false;
        }
    }

    let exec_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let output = tokio::process::Command::new("dpkg")
        .args(["-S", &exec_path])
        .output()
        .await;

    match output {
        Ok(o) => o.status.success() && !o.stdout.is_empty(),
        Err(_) => false,
    }
}

pub async fn detect_rpm() -> bool {
    let platform = get_platform();

    if platform != "linux" {
        return false;
    }

    if let Some(os_release) = get_os_release() {
        if !is_distro_family(&os_release, &["fedora", "rhel", "suse"]) {
            return false;
        }
    }

    let exec_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let output = tokio::process::Command::new("rpm")
        .args(["-qf", &exec_path])
        .output()
        .await;

    match output {
        Ok(o) => o.status.success() && !o.stdout.is_empty(),
        Err(_) => false,
    }
}

pub async fn detect_apk() -> bool {
    let platform = get_platform();

    if platform != "linux" {
        return false;
    }

    if let Some(os_release) = get_os_release() {
        if !is_distro_family(&os_release, &["alpine"]) {
            return false;
        }
    }

    let exec_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let output = tokio::process::Command::new("apk")
        .args(["info", "--who-owns", &exec_path])
        .output()
        .await;

    match output {
        Ok(o) => o.status.success() && !o.stdout.is_empty(),
        Err(_) => false,
    }
}

pub async fn get_package_manager() -> PackageManager {
    if let Ok(cache) = PACKAGE_MANAGER_CACHE.lock() {
        if let Some(pm) = *cache {
            return pm;
        }
    }

    let pm = get_package_manager_impl().await;

    if let Ok(mut cache) = PACKAGE_MANAGER_CACHE.lock() {
        *cache = Some(pm);
    }

    pm
}

async fn get_package_manager_impl() -> PackageManager {
    if detect_homebrew() {
        return PackageManager::Homebrew;
    }

    if detect_winget() {
        return PackageManager::Winget;
    }

    if detect_mise() {
        return PackageManager::Mise;
    }

    if detect_asdf() {
        return PackageManager::Asdf;
    }

    if detect_pacman().await {
        return PackageManager::Pacman;
    }

    if detect_apk().await {
        return PackageManager::Apk;
    }

    if detect_deb().await {
        return PackageManager::Deb;
    }

    if detect_rpm().await {
        return PackageManager::Rpm;
    }

    PackageManager::Unknown
}

pub fn is_native_installer_available() -> bool {
    #[cfg(target_os = "macos")]
    {
        Command::new("brew")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("apt-get")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        false
    }
}

pub fn install_package(package: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("brew")
            .args(["install", package])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("sudo")
            .args(["apt-get", "install", "-y", package])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err("Unsupported platform".to_string())
    }
}
