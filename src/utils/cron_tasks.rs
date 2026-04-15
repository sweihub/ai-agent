//! Scheduled prompts, stored in <project>/.ai/scheduled_tasks.json.
//!
//! Tasks come in two flavors:
//!   - One-shot (recurring: false/undefined) — fire once, then auto-delete.
//!   - Recurring (recurring: true) — fire on schedule, reschedule from now,
//!     persist until explicitly deleted via CronDelete or auto-expire after
//!     a configurable limit (DEFAULT_CRON_JITTER_CONFIG.recurring_max_age_ms).
//!
//! File format:
//!   { "tasks": [{ id, cron, prompt, createdAt, recurring?, permanent? }] }

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::bootstrap::state::{
    add_session_cron_task, get_project_root, get_session_cron_tasks, remove_session_cron_tasks,
};
use crate::utils::cron::{compute_next_cron_run, parse_cron_expression};
use crate::utils::debug::log_for_debugging;
use crate::utils::errors::is_fs_inaccessible;
use crate::utils::fs::get_fs_implementation;
use crate::utils::json::safe_parse_json;
use crate::utils::log::log_error;
use crate::utils::slow_operations::json_stringify;

/// A scheduled cron task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronTask {
    /// Unique identifier (8 hex chars).
    pub id: String,
    /// 5-field cron string (local time) — validated on write, re-validated on read.
    pub cron: String,
    /// Prompt to enqueue when the task fires.
    pub prompt: String,
    /// Epoch ms when the task was created.
    pub created_at: u64,
    /// Epoch ms of the most recent fire.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_fired_at: Option<u64>,
    /// When true, the task reschedules after firing instead of being deleted.
    #[serde(default)]
    pub recurring: bool,
    /// When true, the task is exempt from recurringMaxAgeMs auto-expiry.
    #[serde(default)]
    pub permanent: bool,
    /// Runtime-only flag. false → session-scoped (never written to disk).
    #[serde(default, skip_serializing)]
    pub durable: bool,
    /// Runtime-only. When set, the task was created by an in-process teammate.
    #[serde(default, skip_serializing)]
    pub agent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CronFile {
    tasks: Vec<CronTask>,
}

const CRON_FILE_REL: &str = ".ai/scheduled_tasks.json";

/// Path to the cron file.
pub fn get_cron_file_path(dir: Option<&str>) -> PathBuf {
    let root = dir.unwrap_or_else(get_project_root);
    PathBuf::from(root).join(CRON_FILE_REL)
}

/// Read and parse .ai/scheduled_tasks.json.
/// Returns an empty task list if the file is missing, empty, or malformed.
/// Tasks with invalid cron strings are silently dropped.
pub async fn read_cron_tasks(dir: Option<&str>) -> Vec<CronTask> {
    let fs = get_fs_implementation();
    let path = get_cron_file_path(dir);

    let raw = match fs.read_to_string(&path) {
        Ok(content) => content,
        Err(e) => {
            if is_fs_inaccessible(&e) {
                return vec![];
            }
            log_error(&e);
            return vec![];
        }
    };

    let parsed = match safe_parse_json(&raw, false) {
        Some(v) => v,
        None => return vec![],
    };

    let obj = match parsed.as_object() {
        Some(o) => o,
        None => return vec![],
    };

    let tasks = match obj.get("tasks").and_then(|t| t.as_array()) {
        Some(t) => t,
        None => return vec![],
    };

    let mut out = Vec::new();
    for t in tasks {
        let task = match t.as_object() {
            Some(o) => o,
            None => continue,
        };

        let id = match task.get("id").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => {
                log_for_debugging(&format!(
                    "[ScheduledTasks] skipping malformed task: {}",
                    json_stringify(t, None)
                ));
                continue;
            }
        };

        let cron = match task.get("cron").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => {
                log_for_debugging(&format!(
                    "[ScheduledTasks] skipping malformed task: {}",
                    json_stringify(t, None)
                ));
                continue;
            }
        };

        let prompt = match task.get("prompt").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => {
                log_for_debugging(&format!(
                    "[ScheduledTasks] skipping malformed task: {}",
                    json_stringify(t, None)
                ));
                continue;
            }
        };

        let created_at = match task.get("createdAt").and_then(|v| v.as_u64()) {
            Some(n) => n,
            None => {
                log_for_debugging(&format!(
                    "[ScheduledTasks] skipping malformed task: {}",
                    json_stringify(t, None)
                ));
                continue;
            }
        };

        if !parse_cron_expression(&cron).is_some() {
            log_for_debugging(&format!(
                "[ScheduledTasks] skipping task {} with invalid cron '{}'",
                id, cron
            ));
            continue;
        }

        let last_fired_at = task.get("lastFiredAt").and_then(|v| v.as_u64());
        let recurring = task
            .get("recurring")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let permanent = task
            .get("permanent")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        out.push(CronTask {
            id,
            cron,
            prompt,
            created_at,
            last_fired_at,
            recurring,
            permanent,
            durable: true,
            agent_id: None,
        });
    }

    out
}

/// Sync check for whether the cron file has any valid tasks.
pub fn has_cron_tasks_sync(dir: Option<&str>) -> bool {
    let path = get_cron_file_path(dir);

    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let parsed = match safe_parse_json(&raw, false) {
        Some(v) => v,
        None => return false,
    };

    let obj = match parsed.as_object() {
        Some(o) => o,
        None => return false,
    };

    let tasks = match obj.get("tasks").and_then(|t| t.as_array()) {
        Some(t) => t,
        None => return false,
    };

    !tasks.is_empty()
}

/// Overwrite .ai/scheduled_tasks.json with the given tasks.
pub async fn write_cron_tasks(
    tasks: &[CronTask],
    dir: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let root = dir.unwrap_or_else(get_project_root);
    let claude_dir = PathBuf::from(root).join(".ai");

    let fs = get_fs_implementation();
    fs.create_dir_all(&claude_dir).await?;

    // Strip the runtime-only `durable` flag
    let body = CronFile {
        tasks: tasks.to_vec(),
    };

    let json = json_stringify(&body, Some(2)) + "\n";
    let path = get_cron_file_path(dir);
    fs.write(&path, json).await?;

    Ok(())
}

/// Append a task. Returns the generated id.
pub async fn add_cron_task(
    cron: &str,
    prompt: &str,
    recurring: bool,
    durable: bool,
    agent_id: Option<&str>,
) -> String {
    use uuid::Uuid;

    // Short ID — 8 hex chars
    let id = Uuid::new_v4().to_string()[..8].to_string();
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let task = CronTask {
        id: id.clone(),
        cron: cron.to_string(),
        prompt: prompt.to_string(),
        created_at,
        last_fired_at: None,
        recurring,
        permanent: false,
        durable,
        agent_id: agent_id.map(String::from),
    };

    if !durable {
        add_session_cron_task(task);
        return id;
    }

    let mut tasks = read_cron_tasks(None).await;
    tasks.push(task);
    write_cron_tasks(&tasks, None).await.ok();

    id
}

/// Remove tasks by id.
pub async fn remove_cron_tasks(ids: &[String], dir: Option<&str>) {
    if ids.is_empty() {
        return;
    }

    // Sweep session store first
    if dir.is_none() {
        let removed = remove_session_cron_tasks(ids);
        if removed == ids.len() {
            return;
        }
    }

    let id_set: std::collections::HashSet<_> = ids.iter().collect();
    let tasks = read_cron_tasks(dir).await;
    let remaining: Vec<_> = tasks
        .into_iter()
        .filter(|t| !id_set.contains(&t.id))
        .collect();

    if remaining.len() != tasks.len() {
        write_cron_tasks(&remaining, dir).await.ok();
    }
}

/// Stamp `lastFiredAt` on the given recurring tasks and write back.
pub async fn mark_cron_tasks_fired(ids: &[String], fired_at: u64, dir: Option<&str>) {
    if ids.is_empty() {
        return;
    }

    let id_set: std::collections::HashSet<_> = ids.iter().collect();
    let mut tasks = read_cron_tasks(dir).await;
    let mut changed = false;

    for t in &mut tasks {
        if id_set.contains(&t.id) {
            t.last_fired_at = Some(fired_at);
            changed = true;
        }
    }

    if changed {
        write_cron_tasks(&tasks, dir).await.ok();
    }
}

/// File-backed tasks + session-only tasks, merged.
pub async fn list_all_cron_tasks(dir: Option<&str>) -> Vec<CronTask> {
    let file_tasks = read_cron_tasks(dir).await;
    if dir.is_some() {
        return file_tasks;
    }

    let session_tasks: Vec<_> = get_session_cron_tasks()
        .into_iter()
        .map(|mut t| {
            t.durable = false;
            t
        })
        .collect();

    let mut result = file_tasks;
    result.extend(session_tasks);
    result
}

/// Next fire time in epoch ms for a cron string, strictly after `from_ms`.
/// Returns None if invalid or no match in the next 366 days.
pub fn next_cron_run_ms(cron: &str, from_ms: u64) -> Option<u64> {
    let fields = parse_cron_expression(cron)?;
    let next = compute_next_cron_run(&fields, from_ms);
    next
}

/// Cron scheduler tuning knobs.
#[derive(Debug, Clone)]
pub struct CronJitterConfig {
    /// Recurring-task forward delay as a fraction of the interval between fires.
    pub recurring_frac: f64,
    /// Upper bound on recurring forward delay regardless of interval length.
    pub recurring_cap_ms: u64,
    /// One-shot backward lead: maximum ms a task may fire early.
    pub one_shot_max_ms: u64,
    /// One-shot backward lead: minimum ms a task fires early.
    pub one_shot_floor_ms: u64,
    /// Jitter fires landing on minutes where minute % N === 0.
    pub one_shot_minute_mod: u32,
    /// Recurring tasks auto-expire this many ms after creation.
    pub recurring_max_age_ms: u64,
}

impl Default for CronJitterConfig {
    fn default() -> Self {
        Self {
            recurring_frac: 0.1,
            recurring_cap_ms: 15 * 60 * 1000,
            one_shot_max_ms: 90 * 1000,
            one_shot_floor_ms: 0,
            one_shot_minute_mod: 30,
            recurring_max_age_ms: 7 * 24 * 60 * 60 * 1000,
        }
    }
}

pub const DEFAULT_CRON_JITTER_CONFIG: CronJitterConfig = CronJitterConfig {
    recurring_frac: 0.1,
    recurring_cap_ms: 15 * 60 * 1000,
    one_shot_max_ms: 90 * 1000,
    one_shot_floor_ms: 0,
    one_shot_minute_mod: 30,
    recurring_max_age_ms: 7 * 24 * 60 * 60 * 1000,
};

/// taskId is an 8-hex-char UUID slice → parse as u32 → [0, 1).
fn jitter_frac(task_id: &str) -> f64 {
    let frac = u32::from_str_radix(&task_id[..8], 16).unwrap_or(0) as f64 / 0x1_0000_0000_f64;
    if frac.is_finite() {
        frac
    } else {
        0.0
    }
}

/// Same as next_cron_run_ms, plus a deterministic per-task delay.
pub fn jittered_next_cron_run_ms(
    cron: &str,
    from_ms: u64,
    task_id: &str,
    cfg: &CronJitterConfig,
) -> Option<u64> {
    let t1 = next_cron_run_ms(cron, from_ms)?;
    let t2 = next_cron_run_ms(cron, t1)?;

    // No second match in the next year → nothing to proportion against
    if t2.is_none() {
        return Some(t1);
    }

    let t2 = t2.unwrap();
    let jitter = (jitter_frac(task_id) * cfg.recurring_frac * (t2 - t1) as f64) as u64;
    let jitter = jitter.min(cfg.recurring_cap_ms);

    Some(t1 + jitter)
}

/// Same as next_cron_run_ms, minus a deterministic per-task lead time.
pub fn one_shot_jittered_next_cron_run_ms(
    cron: &str,
    from_ms: u64,
    task_id: &str,
    cfg: &CronJitterConfig,
) -> Option<u64> {
    let t1 = next_cron_run_ms(cron, from_ms)?;

    // Cron resolution is 1 minute
    let minutes = (t1 / 60000) as u32 % 60;
    if minutes % cfg.one_shot_minute_mod != 0 {
        return Some(t1);
    }

    let lead = cfg.one_shot_floor_ms
        + (jitter_frac(task_id) * (cfg.one_shot_max_ms - cfg.one_shot_floor_ms) as f64) as u64;

    Some(t1.saturating_sub(lead).max(from_ms))
}

/// A task is "missed" when its next scheduled run is in the past.
pub fn find_missed_tasks(tasks: &[CronTask], now_ms: u64) -> Vec<CronTask> {
    tasks
        .iter()
        .filter(|t| {
            if let Some(next) = next_cron_run_ms(&t.cron, t.created_at) {
                next < now_ms
            } else {
                false
            }
        })
        .cloned()
        .collect()
}
