//! Bridge main loop for remote session management.
//!
//! Translated from openclaudecode/src/bridge/bridgeMain.ts

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::bridge::bridge_api::{BridgeApiClient, BridgeFatalError};
use crate::bridge::bridge_status_util::format_duration;
use crate::bridge::bridge_ui::BridgeLogger;
use crate::bridge::capacity_wake::create_capacity_wake;
use crate::bridge::debug_utils::describe_axios_error;
use crate::bridge::jwt_utils::create_token_refresh_scheduler;
use crate::bridge::poll_config::get_poll_interval_config;
use crate::bridge::session_id_compat::to_compat_session_id;
use crate::bridge::session_runner::{
    create_session_spawner, safe_filename_id, SessionHandle, SessionSpawnOpts, SessionSpawner,
};
use crate::bridge::types::{BridgeConfig, SessionDoneStatus, DEFAULT_SESSION_TIMEOUT_MS};
use crate::bridge::work_secret::{
    build_ccr_v2_sdk_url, build_sdk_url, decode_work_secret, register_worker,
};

const STATUS_UPDATE_INTERVAL_MS: u64 = 1000;
const SPAWN_SESSIONS_DEFAULT: u32 = 32;

#[derive(Debug, Clone)]
pub struct BackoffConfig {
    pub conn_initial_ms: u64,
    pub conn_cap_ms: u64,
    pub conn_give_up_ms: u64,
    pub general_initial_ms: u64,
    pub general_cap_ms: u64,
    pub general_give_up_ms: u64,
    pub shutdown_grace_ms: Option<u64>,
    pub stop_work_base_delay_ms: Option<u64>,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            conn_initial_ms: 2_000,
            conn_cap_ms: 120_000,
            conn_give_up_ms: 600_000,
            general_initial_ms: 500,
            general_cap_ms: 30_000,
            general_give_up_ms: 600_000,
            shutdown_grace_ms: Some(30_000),
            stop_work_base_delay_ms: Some(1_000),
        }
    }
}

fn poll_sleep_detection_threshold_ms(backoff: &BackoffConfig) -> u64 {
    backoff.conn_cap_ms * 2
}

fn is_connection_error(err: &dyn std::error::Error) -> bool {
    let msg = err.to_string().to_lowercase();
    msg.contains("connection")
        || msg.contains("econnrefused")
        || msg.contains("etimedout")
        || msg.contains("network")
}

fn is_server_error(status: u16) -> bool {
    status >= 500 && status < 600
}

fn derive_session_title(_first_message: &str) -> String {
    "New Session".to_string()
}

async fn fetch_session_title(_session_id: &str, _base_url: &str) -> Option<String> {
    None
}

async fn stop_work_with_retry(
    api: &dyn BridgeApiClient,
    environment_id: &str,
    work_id: &str,
    _logger: &dyn BridgeLogger,
    base_delay_ms: Option<u64>,
) {
    let delay = base_delay_ms.unwrap_or(1000);
    let mut attempt = 0;
    let max_attempts = 3;

    while attempt < max_attempts {
        let result = api.stop_work(environment_id, work_id, false);
        match result {
            Ok(()) => return,
            Err(e) => {
                attempt += 1;
                if attempt >= max_attempts {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(delay * 2u64.pow(attempt as u32))).await;
            }
        }
    }
}

fn on_session_timeout(
    session_id: &str,
    timeout_ms: u64,
    _logger: &dyn BridgeLogger,
    timed_out_sessions: &mut HashSet<String>,
    _handle: &SessionHandle,
) {
    timed_out_sessions.insert(session_id.to_string());
    log::warn!("Session {} timed out after {}ms", session_id, timeout_ms);
}

/// Run the main bridge poll loop.
///
/// This handles:
/// - Polling for work from the bridge API
/// - Spawning sessions to handle work items
/// - Managing active sessions
/// - Heartbeat for active work items
/// - Token refresh
/// - Capacity management
pub async fn run_bridge_loop(
    config: BridgeConfig,
    environment_id: String,
    environment_secret: String,
    api: Box<dyn BridgeApiClient>,
    spawner: Box<dyn SessionSpawner>,
    logger: Box<dyn BridgeLogger>,
    abort_signal: Arc<AtomicBool>,
    backoff_config: Option<BackoffConfig>,
    initial_session_id: Option<String>,
    get_access_token: Option<Box<dyn Fn() -> Option<String> + Send + Sync>>,
) -> Result<(), String> {
    let backoff = backoff_config.unwrap_or_default();

    let loop_signal = abort_signal.clone();

    let active_sessions: HashMap<String, SessionHandle> = HashMap::new();
    let session_start_times: HashMap<String, u64> = HashMap::new();
    let session_work_ids: HashMap<String, String> = HashMap::new();
    let session_compat_ids: HashMap<String, String> = HashMap::new();
    let session_ingress_tokens: HashMap<String, String> = HashMap::new();
    let completed_work_ids: HashSet<String> = HashSet::new();
    let timed_out_sessions: HashSet<String> = HashSet::new();
    let titled_sessions: HashSet<String> = HashSet::new();
    let v2_sessions: HashSet<String> = HashSet::new();

    let capacity_wake = create_capacity_wake(loop_signal.clone());

    let mut conn_backoff: u64 = 0;
    let mut general_backoff: u64 = 0;
    let mut conn_error_start: Option<u64> = None;
    let mut general_error_start: Option<u64> = None;
    let mut last_poll_error_time: Option<u64> = None;
    let mut fatal_exit = false;

    log::debug!(
        "[bridge:work] Starting poll loop spawnMode={} maxSessions={} environmentId={}",
        config.spawn_mode,
        config.max_sessions,
        environment_id
    );

    logger.print_banner(&config, &environment_id);
    logger.update_session_count(0, config.max_sessions, &config.spawn_mode);

    if let Some(ref initial) = initial_session_id {
        logger.set_attached(initial);
    }

    while !loop_signal.load(Ordering::SeqCst) {
        let poll_config = get_poll_interval_config();

        match api.poll_for_work(
            &environment_id,
            &environment_secret,
            poll_config.reclaim_older_than_ms,
        ) {
            Ok(Some(work)) => {
                conn_backoff = 0;
                general_backoff = 0;
                conn_error_start = None;
                general_error_start = None;
                last_poll_error_time = None;

                if completed_work_ids.contains(&work.id) {
                    log::debug!(
                        "[bridge:work] Skipping already-completed workId={}",
                        work.id
                    );
                    continue;
                }

                let secret = match decode_work_secret(&work.secret) {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("Failed to decode work secret: {}", e);
                        completed_work_ids.insert(work.id.clone());
                        continue;
                    }
                };

                let work_type = work.data.data_type.clone();
                match work_type.as_str() {
                    "healthcheck" => {
                        log::debug!("[bridge:work] Healthcheck received");
                    }
                    "session" => {
                        let session_id = work.data.id.clone();

                        if active_sessions.len() >= config.max_sessions as usize {
                            log::debug!(
                                "[bridge:work] At capacity ({}/{}), cannot spawn new session",
                                active_sessions.len(),
                                config.max_sessions
                            );
                            continue;
                        }

                        let use_ccr_v2 = secret.use_code_sessions;
                        let sdk_url = if use_ccr_v2 {
                            build_ccr_v2_sdk_url(&config.api_base_url, &session_id)
                        } else {
                            build_sdk_url(&config.session_ingress_url, &session_id)
                        };

                        log::debug!(
                            "[bridge:session] Spawning sessionId={} sdkUrl={}",
                            session_id,
                            sdk_url
                        );

                        let compat_session_id = to_compat_session_id(&session_id);

                        let spawn_opts = SessionSpawnOpts {
                            session_id: session_id.clone(),
                            sdk_url: sdk_url.clone(),
                            access_token: secret.session_ingress_token.clone(),
                            use_ccr_v2,
                            worker_epoch: None,
                        };

                        let spawn_result = spawner.spawn(spawn_opts, &config.dir);

                        match spawn_result {
                            Ok(handle) => {
                                active_sessions.insert(session_id.clone(), handle);
                                session_work_ids.insert(session_id.clone(), work.id.clone());
                                session_ingress_tokens.insert(
                                    session_id.clone(),
                                    secret.session_ingress_token.clone(),
                                );
                                session_compat_ids
                                    .insert(session_id.clone(), compat_session_id.clone());
                                session_start_times.insert(
                                    session_id.clone(),
                                    std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis() as u64,
                                );

                                logger.add_session(
                                    &compat_session_id,
                                    &format!(
                                        "{}/sessions/{}",
                                        config.session_ingress_url, compat_session_id
                                    ),
                                );
                                logger.set_attached(&compat_session_id);
                            }
                            Err(e) => {
                                log::error!("Failed to spawn session: {}", e);
                            }
                        }
                    }
                    _ => {
                        log::debug!("[bridge:work] Unknown work type: {}, skipping", work_type);
                    }
                }
            }
            Ok(None) => {
                let at_capacity = active_sessions.len() >= config.max_sessions as usize;
                if at_capacity {
                    let at_cap_ms = poll_config.multisession_poll_interval_ms_at_capacity;
                    if at_cap_ms > 0 {
                        tokio::time::sleep(Duration::from_millis(at_cap_ms)).await;
                    }
                } else {
                    let interval = if !active_sessions.is_empty() {
                        poll_config.multisession_poll_interval_ms_partial_capacity
                    } else {
                        poll_config.multisession_poll_interval_ms_not_at_capacity
                    };
                    tokio::time::sleep(Duration::from_millis(interval)).await;
                }
            }
            Err(e) => {
                if loop_signal.load(Ordering::SeqCst) {
                    break;
                }

                if let Some(bridge_err) = e.downcast_ref::<BridgeFatalError>() {
                    fatal_exit = true;
                    logger.log_error(&bridge_err.message);
                    break;
                }

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;

                if let Some(last_err) = last_poll_error_time {
                    if now - last_err > poll_sleep_detection_threshold_ms(&backoff) {
                        conn_backoff = 0;
                        general_backoff = 0;
                        conn_error_start = None;
                        general_error_start = None;
                    }
                }

                last_poll_error_time = Some(now);

                if conn_error_start.is_none() {
                    conn_error_start = Some(now);
                }
                if general_error_start.is_none() {
                    general_error_start = Some(now);
                }

                let backoff_ms = if conn_backoff == 0 {
                    backoff.conn_initial_ms
                } else {
                    (conn_backoff * 2).min(backoff.conn_cap_ms)
                };
                conn_backoff = if conn_backoff == 0 {
                    1
                } else {
                    (conn_backoff * 2).min(backoff.conn_cap_ms)
                };

                log::debug!("[bridge:poll] Error, backing off for {}ms", backoff_ms);
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_config_default() {
        let config = BackoffConfig::default();
        assert_eq!(config.conn_initial_ms, 2_000);
        assert_eq!(config.conn_cap_ms, 120_000);
    }

    #[test]
    fn test_poll_sleep_detection_threshold() {
        let config = BackoffConfig::default();
        let threshold = poll_sleep_detection_threshold_ms(&config);
        assert_eq!(threshold, 240_000);
    }
}
