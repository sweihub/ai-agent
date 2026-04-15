//! Hook utilities for agent tool permission checking, hook execution, and related helpers.
//!
//! This module provides:
//! - `can_use_tool`: The CanUseToolFn type and related types for permission checking
//! - `helpers`: Helper functions for structured output and argument substitution
//! - `hook_helpers`: Additional hook helpers for structured output enforcement
//! - `api_query_hook_helper`: API query hook creation and execution
//! - `async_hook_registry`: Registry for async hooks with progress tracking
//! - `exec_agent_hook`: Execute agent-based hooks using multi-turn LLM queries
//! - `exec_http_hook`: Execute HTTP hooks by POSTing to configured URLs
//! - `exec_prompt_hook`: Execute prompt-based hooks using single LLM queries
//! - `file_changed_watcher`: File system watcher for CwdChanged/FileChanged hooks
//! - `hook_events`: Event system for broadcasting hook execution events
//! - `hooks_config_manager`: Hook configuration management with event metadata
//! - `hooks_config_snapshot`: Hooks configuration snapshot with policy enforcement
//! - `hooks_settings`: Hook settings parsing and source management
//! - `post_sampling_hooks`: Post-sampling hook registration and execution
//! - `register_frontmatter_hooks`: Register hooks from agent/skill frontmatter
//! - `register_skill_hooks`: Register hooks from skill frontmatter with once support
//! - `session_hooks`: Session-scoped hook storage and management
//! - `skill_improvement`: Automatic skill improvement detection and application
//! - `ssrf_guard`: SSRF guard for HTTP hook DNS resolution

pub mod can_use_tool;
pub mod helpers;
pub mod hook_helpers;
pub mod api_query_hook_helper;
pub mod async_hook_registry;
pub mod exec_agent_hook;
pub mod exec_http_hook;
pub mod exec_prompt_hook;
pub mod file_changed_watcher;
pub mod hook_events;
pub mod hooks_config_manager;
pub mod hooks_config_snapshot;
pub mod hooks_settings;
pub mod post_sampling_hooks;
pub mod register_frontmatter_hooks;
pub mod register_skill_hooks;
pub mod session_hooks;
pub mod skill_improvement;
pub mod ssrf_guard;

// Re-export core types from existing modules
pub use can_use_tool::*;
pub use helpers::*;

// Re-export from new modules with explicit exports to avoid ambiguity
pub use hook_helpers::{HookResponse, hook_response_schema, create_structured_output_tool,
    register_structured_output_enforcement, has_successful_tool_call, SYNTHETIC_OUTPUT_TOOL_NAME};
pub use api_query_hook_helper::{ReplHookContext, ApiQueryHookConfig, ApiQueryResult,
    create_api_query_hook, SystemPrompt};
pub use async_hook_registry::{HookEvent as AsyncHookEvent, HookExecutionEvent, HookOutcome,
    PendingAsyncHook, register_pending_async_hook, get_pending_async_hooks,
    check_for_async_hook_responses, emit_hook_started, emit_hook_progress,
    emit_hook_response, start_hook_progress_interval, register_hook_event_handler,
    set_all_hook_events_enabled, clear_hook_event_state};
pub use exec_agent_hook::{exec_agent_hook, HookResult as ExecAgentHookResult};
pub use exec_http_hook::{exec_http_hook, HttpHook, HttpHookResult};
pub use exec_prompt_hook::{exec_prompt_hook, HookResult as ExecPromptHookResult, PromptHook};
pub use file_changed_watcher::{initialize_file_changed_watcher, on_cwd_changed_for_hooks,
    update_watch_paths, set_env_hook_notifier, FileEvent, HookOutsideReplResult};
pub use hook_events::{HookStartedEvent, HookProgressEvent, HookResponseEvent,
    HookExecutionEvent as HookEventsExecution, HookOutcome as HookEventsOutcome,
    HookEventHandler, ProgressOutput, StartHookProgressParams as HookEventsStartHookProgressParams,
    EmitHookResponseParams};
pub use hooks_config_manager::{HookEvent, HookEventMetadata, MatcherMetadata,
    IndividualHookConfig, HookCommand as HookConfigCommand, HookSource as HookConfigSource,
    HOOK_EVENTS, get_hook_event_metadata, group_hooks_by_event_and_matcher,
    sort_matchers_by_priority as sort_matchers_config, get_hooks_for_matcher as get_hooks_config,
    get_matcher_metadata, get_hook_display_text, is_hook_equal as is_hook_equal_config,
    hook_source_description_display_string, hook_source_header_display_string,
    hook_source_inline_display_string};
pub use hooks_config_snapshot::{HooksSettings, HookMatcher, capture_hooks_config_snapshot,
    update_hooks_config_snapshot, get_hooks_config_from_snapshot, reset_hooks_config_snapshot,
    should_allow_managed_hooks_only, should_disable_all_hooks_including_managed};
pub use hooks_settings::{HookEvent as HooksSettingsEvent, HOOK_EVENTS as HOOK_EVENTS_SETTINGS,
    EditableSettingSource, SOURCES, HookSource as HooksSettingsSource,
    IndividualHookConfig as HooksSettingsIndividualHookConfig,
    HookCommand as HooksSettingsHookCommand, DEFAULT_HOOK_SHELL, is_hook_equal,
    get_hook_display_text as get_hook_display_text_settings, get_all_hooks, get_hooks_for_event,
    hook_source_description_display_string as hook_source_description_display_string_settings,
    hook_source_header_display_string as hook_source_header_display_string_settings,
    hook_source_inline_display_string as hook_source_inline_display_string_settings,
    sort_matchers_by_priority as sort_matchers_by_priority_settings};
pub use post_sampling_hooks::{ReplHookContext as PostSamplingReplHookContext,
    PostSamplingHook, register_post_sampling_hook, clear_post_sampling_hooks,
    execute_post_sampling_hooks};
pub use register_frontmatter_hooks::{register_frontmatter_hooks, HooksSettings as FrontmatterHooksSettings,
    HookMatcher as FrontmatterHookMatcher};
pub use register_skill_hooks::{register_skill_hooks, HooksSettings as SkillHooksSettings,
    HookMatcher as SkillHookMatcher};
pub use session_hooks::{FunctionHook, FunctionHookCallback, OnHookSuccess, AggregatedHookResult,
    SessionHookMatcher, SessionHookEntry, SessionStore, SessionDerivedHookMatcher,
    FunctionHookMatcher, SessionHookCommand, add_session_hook, add_function_hook,
    remove_function_hook, remove_session_hook, get_session_hooks, get_session_function_hooks,
    get_session_hook_callback, clear_session_hooks};
pub use skill_improvement::{SkillUpdate, SkillImprovementSuggestion, init_skill_improvement,
    apply_skill_improvement};
pub use ssrf_guard::{is_blocked_address, ssrf_guarded_lookup, ssrf_guarded_lookup_async,
    LookupAddress, DnsLookupResult, SsrfError, create_ssrf_protected_connector};
