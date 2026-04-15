//! Bridge module for Remote Control functionality.
//!
//! Translated from openclaudecode/src/bridge/

pub mod bridge_api;
pub mod bridge_config;
pub mod bridge_debug;
pub mod bridge_enabled;
pub mod bridge_messaging;
pub mod bridge_permission_callbacks;
pub mod bridge_pointer;
pub mod bridge_status_util;
pub mod bridge_types;
pub mod bridge_ui;
pub mod capacity_wake;
pub mod code_session_api;
pub mod create_session;
pub mod debug_utils;
pub mod env_less_bridge_config;
pub mod flush_gate;
pub mod inbound_attachments;
pub mod inbound_messages;
pub mod jwt_utils;
pub mod peer_sessions;
pub mod poll_config;
pub mod poll_config_defaults;
pub mod remote_bridge_core;
pub mod repl_bridge;
pub mod repl_bridge_core;
pub mod repl_bridge_handle;
pub mod repl_bridge_transport;
pub mod session_id_compat;
pub mod session_runner;
pub mod trusted_device;
pub mod webhook_sanitizer;
pub mod work_secret;

pub use bridge_api::{
    is_expired_error_type, is_suppressible_403, validate_bridge_id, BridgeFatalError,
    HeartbeatResponse, PermissionResponseEvent, RegistrationResponse, SyncBridgeApiClient,
    BRIDGE_LOGIN_ERROR, BRIDGE_LOGIN_INSTRUCTION,
};
pub use bridge_config::{
    get_bridge_access_token, get_bridge_base_url, get_bridge_base_url_override,
    get_bridge_token_override,
};
pub use bridge_debug::{
    clear_bridge_debug_handle, get_bridge_debug_handle, inject_bridge_fault,
    register_bridge_debug_handle, BridgeDebugHandle, BridgeFault, BridgeFaultKind,
    BridgeFaultMethod,
};
pub use bridge_enabled::{
    check_bridge_min_version, get_bridge_disabled_reason, get_ccr_auto_connect_default,
    is_bridge_enabled, is_bridge_enabled_blocking, is_ccr_mirror_enabled, is_cse_shim_enabled,
    is_env_less_bridge_enabled, register_cse_shim_gate, OAuthAccountInfo,
};
pub use bridge_messaging::{
    extract_title_text, handle_ingress_message, handle_server_control_request,
    is_eligible_bridge_message, is_sdk_control_request, is_sdk_control_response, is_sdk_message,
    make_result_message, BoundedUuidSet, SDKControlRequest, SDKControlRequestPayload,
    SDKControlResponse, SDKControlResponsePayload, SDKResultSuccess, ServerControlRequestHandlers,
};
pub use bridge_permission_callbacks::{
    is_bridge_permission_response, BridgePermissionBehavior, BridgePermissionCallbacks,
    BridgePermissionResponse, InMemoryBridgePermissionCallbacks, PermissionUpdate,
};
pub use bridge_pointer::{
    clear_bridge_pointer, get_bridge_pointer_path, read_bridge_pointer,
    read_bridge_pointer_across_worktrees, write_bridge_pointer, BridgePointer, BridgePointerSource,
    BridgePointerWithAge, BRIDGE_POINTER_TTL_MS,
};
pub use bridge_status_util::{
    abbreviate_activity, build_active_footer_text, build_bridge_connect_url,
    build_bridge_session_url, build_idle_footer_text, compute_glimmer_index,
    compute_shimmer_segments, format_duration, get_bridge_status, timestamp, truncate_to_width,
    wrap_with_osc8_link, BridgeStatusColor, BridgeStatusInfo, BridgeStatusLabel, StatusState,
    FAILED_FOOTER_TEXT, SHIMMER_INTERVAL_MS, TOOL_DISPLAY_EXPIRY_MS,
};
pub use bridge_types::{
    BridgeApiClient, BridgeConfig as TypesBridgeConfig, BridgeLogger, BridgeWorkerType, GitInfo,
    HeartbeatResponse as TypesHeartbeatResponse,
    PermissionResponseEvent as TypesPermissionResponseEvent, PermissionResponseInner,
    SessionActivity, SessionActivityType, SessionDoneStatus, SessionHandle, SessionSpawnOpts,
    SessionSpawner, SpawnMode, WorkAuth, WorkSecret, WorkSource,
    BRIDGE_LOGIN_ERROR as TYPES_BRIDGE_LOGIN_ERROR,
    BRIDGE_LOGIN_INSTRUCTION as TYPES_BRIDGE_LOGIN_INSTRUCTION, DEFAULT_SESSION_TIMEOUT_MS,
    REMOTE_CONTROL_DISCONNECTED_MSG,
};
pub use bridge_ui::{create_bridge_logger, BridgeLoggerImpl};
pub use capacity_wake::{create_capacity_wake, CapacitySignal, CapacityWake};
pub use code_session_api::{create_code_session, fetch_remote_credentials, RemoteCredentials};
pub use create_session::{
    archive_bridge_session, create_bridge_session, get_bridge_session, update_bridge_session_title,
    BridgeSessionInfo, GitInfo as SessionGitInfo, GitOutcome, GitSource, SessionContext,
    SessionEvent,
};
pub use debug_utils::*;
pub use env_less_bridge_config::*;
pub use flush_gate::FlushGate;
pub use inbound_attachments::*;
pub use inbound_messages::{ContentBlock, SDKMessage, UserMessageContent};
pub use jwt_utils::*;
pub use peer_sessions::list_peer_sessions;
pub use poll_config::get_poll_interval_config;
pub use poll_config_defaults::{PollIntervalConfig, DEFAULT_POLL_CONFIG};
pub use remote_bridge_core::{
    init_env_less_bridge_core, EnvLessBridgeHandle, EnvLessBridgeParams,
    RemoteCredentials as EnvLessRemoteCredentials,
};
pub use repl_bridge::{init_repl_bridge, ReplBridge, ReplBridgeOptions};
pub use repl_bridge_core::{init_bridge_core, BridgeCoreHandle, BridgeCoreParams};
pub use repl_bridge_handle::{
    BridgeControlRequest, BridgeControlResponse, BridgeState, SessionState,
};
pub use repl_bridge_transport::{DeliveryStatus, ReplBridgeTransport, ReplBridgeTransportBuilder};
pub use session_id_compat::{set_cse_shim_gate, to_compat_session_id, to_infra_session_id};
pub use session_runner::{
    create_session_spawner, safe_filename_id, PermissionRequest,
    SessionActivity as RunnerSessionActivity, SessionActivityType as RunnerSessionActivityType,
    SessionDoneStatus as RunnerSessionDoneStatus, SessionHandle as RunnerSessionHandle,
    SessionSpawnOpts as RunnerSessionSpawnOpts, SessionSpawnerDeps,
};
pub use trusted_device::{
    clear_trusted_device_token, clear_trusted_device_token_cache, enroll_trusted_device,
    get_trusted_device_token, SecureStorage, StorageData,
};
pub use webhook_sanitizer::sanitize_webhook_payload;
pub use work_secret::{
    build_ccr_v2_sdk_url, build_sdk_url, decode_work_secret, register_worker, same_session_id,
    GitInfo as SecretGitInfo, WorkAuth as SecretWorkAuth, WorkSource as SecretWorkSource,
};
