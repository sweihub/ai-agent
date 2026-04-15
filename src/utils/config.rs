// Source: /data/home/swei/claudecode/openclaudecode/src/services/autoDream/config.ts
//! Configuration management utilities
//! Translated from /data/home/swei/claudecode/openclaudecode/src/utils/config.ts

use crate::constants::env::{ai, system};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use once_cell::sync::Lazy;

// Re-entrancy guard: prevents get_config -> log_event -> get_global_config -> get_config
// infinite recursion when the config file is corrupted.
static INSIDE_GET_CONFIG: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

// Cache for global config
static GLOBAL_CONFIG_CACHE: Lazy<Mutex<GlobalConfigCache>> =
    Lazy::new(|| Mutex::new(GlobalConfigCache::default()));

#[derive(Default)]
struct GlobalConfigCache {
    config: Option<GlobalConfig>,
    mtime: u64,
}

/// Install method for the application
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallMethod {
    #[default]
    Unknown,
    Local,
    Native,
    Global,
}

/// Release channel
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseChannel {
    #[default]
    Stable,
    Latest,
}

/// Diff tool configuration
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffTool {
    #[default]
    Auto,
    Terminal,
}

/// Editor mode
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EditorMode {
    #[default]
    Normal,
    Emacs,
    Vim,
}

/// Notification channel
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationChannel {
    #[default]
    Auto,
    Terminal,
    Native,
}

/// Theme setting
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeSetting {
    #[default]
    Dark,
    Light,
    System,
}

/// Account info from OAuth
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AccountInfo {
    #[serde(default)]
    pub account_uuid: String,
    #[serde(default)]
    pub email_address: String,
    pub organization_uuid: Option<String>,
    pub organization_name: Option<String>,
    pub organization_role: Option<String>,
    pub workspace_role: Option<String>,
    pub display_name: Option<String>,
    pub has_extra_usage_enabled: Option<bool>,
    pub billing_type: Option<String>,
    pub account_created_at: Option<String>,
    pub subscription_created_at: Option<String>,
}

/// MCP server configuration
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct McpServerConfig {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
}

/// Project-specific configuration (all fields use camelCase in settings.json)
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    #[serde(default)]
    pub mcp_context_uris: Vec<String>,
    pub mcp_servers: Option<HashMap<String, McpServerConfig>>,
    pub last_api_duration: Option<u64>,
    pub last_api_duration_without_retries: Option<u64>,
    pub last_tool_duration: Option<u64>,
    pub last_cost: Option<f64>,
    pub last_duration: Option<u64>,
    pub last_lines_added: Option<u32>,
    pub last_lines_removed: Option<u32>,
    pub last_total_input_tokens: Option<u32>,
    pub last_total_output_tokens: Option<u32>,
    pub last_total_cache_creation_input_tokens: Option<u32>,
    pub last_total_cache_read_input_tokens: Option<u32>,
    pub last_total_web_search_requests: Option<u32>,
    pub last_fps_average: Option<f64>,
    pub last_fps_low_1_pct: Option<f64>,
    pub last_session_id: Option<String>,
    pub last_model_usage: Option<HashMap<String, ModelUsage>>,
    pub last_session_metrics: Option<HashMap<String, f64>>,
    pub example_files: Option<Vec<String>>,
    pub example_files_generated_at: Option<u64>,
    #[serde(default)]
    pub has_trust_dialog_accepted: bool,
    #[serde(default)]
    pub has_completed_project_onboarding: bool,
    #[serde(default)]
    pub project_onboarding_seen_count: u32,
    #[serde(default)]
    pub has_claude_md_external_includes_approved: bool,
    #[serde(default)]
    pub has_claude_md_external_includes_warning_shown: bool,
    pub enabled_mcpjson_servers: Option<Vec<String>>,
    pub disabled_mcpjson_servers: Option<Vec<String>>,
    pub enable_all_project_mcp_servers: Option<bool>,
    pub disabled_mcp_servers: Option<Vec<String>>,
    pub enabled_mcp_servers: Option<Vec<String>>,
    pub active_worktree_session: Option<WorktreeSession>,
    pub remote_control_spawn_mode: Option<String>,
}

/// Model usage statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ModelUsage {
    #[serde(default)]
    pub input_tokens: u32,
    #[serde(default)]
    pub output_tokens: u32,
    #[serde(default)]
    pub cache_read_input_tokens: u32,
    #[serde(default)]
    pub cache_creation_input_tokens: u32,
    #[serde(default)]
    pub web_search_requests: u32,
    #[serde(default)]
    pub cost_usd: f64,
}

/// Worktree session information
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct WorktreeSession {
    #[serde(default)]
    pub original_cwd: String,
    #[serde(default)]
    pub worktree_path: String,
    #[serde(default)]
    pub worktree_name: String,
    pub original_branch: Option<String>,
    pub session_id: String,
    pub hook_based: Option<bool>,
}

/// Global application configuration
/// Note: All fields are serialized to camelCase in settings.json
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalConfig {
    pub api_key_helper: Option<String>,
    pub projects: Option<HashMap<String, ProjectConfig>>,
    #[serde(default)]
    pub num_startups: u32,
    pub install_method: Option<InstallMethod>,
    pub auto_updates: Option<bool>,
    pub auto_updates_protected_for_native: Option<bool>,
    pub doctor_shown_at_session: Option<u32>,
    pub user_id: Option<String>,
    #[serde(default)]
    pub theme: Option<ThemeSetting>,
    pub has_completed_onboarding: Option<bool>,
    pub last_onboarding_version: Option<String>,
    pub last_release_notes_seen: Option<String>,
    pub changelog_last_fetched: Option<u64>,
    pub cached_changelog: Option<String>,
    pub mcp_servers: Option<HashMap<String, McpServerConfig>>,
    pub claude_ai_mcp_ever_connected: Option<Vec<String>>,
    #[serde(default)]
    pub preferred_notif_channel: NotificationChannel,
    pub custom_notify_command: Option<String>,
    #[serde(default)]
    pub verbose: Option<bool>,
    pub custom_api_key_responses: Option<CustomApiKeyResponses>,
    pub primary_api_key: Option<String>,
    pub has_acknowledged_cost_threshold: Option<bool>,
    pub has_seen_undercover_auto_notice: Option<bool>,
    pub has_seen_ultraplan_terms: Option<bool>,
    pub has_reset_auto_mode_opt_in_for_default_offer: Option<bool>,
    pub oauth_account: Option<AccountInfo>,
    pub iterm2_key_binding_installed: Option<bool>,
    pub editor_mode: Option<EditorMode>,
    pub bypass_permissions_mode_accepted: Option<bool>,
    pub has_used_backslash_return: Option<bool>,
    #[serde(default)]
    pub auto_compact_enabled: bool,
    #[serde(default)]
    pub show_turn_duration: bool,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub has_seen_tasks_hint: Option<bool>,
    pub has_used_stash: Option<bool>,
    pub has_used_background_task: Option<bool>,
    pub queued_command_up_hint_count: Option<u32>,
    pub diff_tool: Option<DiffTool>,
    pub iterm2_setup_in_progress: Option<bool>,
    pub iterm2_backup_path: Option<String>,
    pub apple_terminal_backup_path: Option<String>,
    pub apple_terminal_setup_in_progress: Option<bool>,
    pub shift_enter_key_binding_installed: Option<bool>,
    pub option_as_meta_key_installed: Option<bool>,
    pub auto_connect_ide: Option<bool>,
    pub auto_install_ide_extension: Option<bool>,
    pub has_ide_onboarding_been_shown: Option<HashMap<String, bool>>,
    pub ide_hint_shown_count: Option<u32>,
    pub has_ide_auto_connect_dialog_been_shown: Option<bool>,
    #[serde(default)]
    pub tips_history: HashMap<String, u32>,
    pub companion: Option<serde_json::Value>,
    pub companion_muted: Option<bool>,
    pub feedback_survey_state: Option<FeedbackSurveyState>,
    pub transcript_share_dismissed: Option<bool>,
    #[serde(default)]
    pub memory_usage_count: u32,
    pub has_shown_s1m_welcome_v2: Option<HashMap<String, bool>>,
    pub s1m_access_cache: Option<HashMap<String, S1mAccessCacheEntry>>,
    pub s1m_non_subscriber_access_cache: Option<HashMap<String, S1mAccessCacheEntry>>,
    pub passes_eligibility_cache: Option<HashMap<String, serde_json::Value>>,
    pub grove_config_cache: Option<HashMap<String, GroveConfigCacheEntry>>,
    pub passes_upsell_seen_count: Option<u32>,
    pub has_visited_passes: Option<bool>,
    pub passes_last_seen_remaining: Option<u32>,
    pub overage_credit_grant_cache: Option<HashMap<String, OverageCreditCacheEntry>>,
    pub overage_credit_upsell_seen_count: Option<u32>,
    pub has_visited_extra_usage: Option<bool>,
    pub voice_notice_seen_count: Option<u32>,
    pub voice_lang_hint_shown_count: Option<u32>,
    pub voice_lang_hint_last_language: Option<String>,
    pub voice_footer_hint_seen_count: Option<u32>,
    pub opus_1m_merge_notice_seen_count: Option<u32>,
    pub experiment_notices_seen_count: Option<HashMap<String, u32>>,
    pub has_shown_opus_plan_welcome: Option<HashMap<String, bool>>,
    #[serde(default)]
    pub prompt_queue_use_count: u32,
    #[serde(default)]
    pub btw_use_count: u32,
    pub last_plan_mode_use: Option<u64>,
    pub subscription_notice_count: Option<u32>,
    pub has_available_subscription: Option<bool>,
    pub subscription_upsell_seen_count: Option<u32>,
    pub recommended_subscription: Option<String>,
    #[serde(default)]
    pub todo_feature_enabled: bool,
    pub show_expanded_todos: Option<bool>,
    pub show_spinner_tree: Option<bool>,
    pub first_start_time: Option<String>,
    #[serde(default)]
    pub message_idle_notif_threshold_ms: u64,
    pub github_action_setup_count: Option<u32>,
    pub slack_app_install_count: Option<u32>,
    #[serde(default)]
    pub file_checkpointing_enabled: bool,
    #[serde(default)]
    pub terminal_progress_bar_enabled: bool,
    pub show_status_in_terminal_tab: Option<bool>,
    pub task_complete_notif_enabled: Option<bool>,
    pub input_needed_notif_enabled: Option<bool>,
    pub agent_push_notif_enabled: Option<bool>,
    pub claude_code_first_token_date: Option<String>,
    pub model_switch_callout_dismissed: Option<bool>,
    pub model_switch_callout_last_shown: Option<u64>,
    pub model_switch_callout_version: Option<String>,
    pub effort_callout_dismissed: Option<bool>,
    pub effort_callout_v2_dismissed: Option<bool>,
    pub remote_dialog_seen: Option<bool>,
    pub bridge_oauth_dead_expires_at: Option<u64>,
    pub bridge_oauth_dead_fail_count: Option<u32>,
    pub desktop_upsell_seen_count: Option<u32>,
    pub desktop_upsell_dismissed: Option<bool>,
    pub idle_return_dismissed: Option<bool>,
    pub opus_pro_migration_complete: Option<bool>,
    pub opus_pro_migration_timestamp: Option<u64>,
    pub sonnet_1m_45_migration_complete: Option<bool>,
    pub legacy_opus_migration_timestamp: Option<u64>,
    pub sonnet_45_to_46_migration_timestamp: Option<u64>,
    #[serde(default)]
    pub cached_statsig_gates: HashMap<String, bool>,
    pub cached_dynamic_configs: Option<HashMap<String, serde_json::Value>>,
    pub cached_growth_book_features: Option<HashMap<String, serde_json::Value>>,
    pub growth_book_overrides: Option<HashMap<String, serde_json::Value>>,
    pub last_shown_emergency_tip: Option<String>,
    #[serde(default)]
    pub respect_gitignore: bool,
    #[serde(default)]
    pub copy_full_response: bool,
    pub copy_on_select: Option<bool>,
    pub github_repo_paths: Option<HashMap<String, Vec<String>>>,
    pub deep_link_terminal: Option<String>,
    pub iterm2_it2_setup_complete: Option<bool>,
    pub prefer_tmux_over_iterm2: Option<bool>,
    pub skill_usage: Option<HashMap<String, SkillUsageEntry>>,
    pub official_marketplace_auto_install_attempted: Option<bool>,
    pub official_marketplace_auto_installed: Option<bool>,
    pub official_marketplace_auto_install_fail_reason: Option<String>,
    pub official_marketplace_auto_install_retry_count: Option<u32>,
    pub official_marketplace_auto_install_last_attempt_time: Option<u64>,
    pub official_marketplace_auto_install_next_retry_time: Option<u64>,
    pub has_completed_claude_in_chrome_onboarding: Option<bool>,
    pub claude_in_chrome_default_enabled: Option<bool>,
    pub cached_chrome_extension_installed: Option<bool>,
    pub chrome_extension: Option<ChromeExtensionState>,
    pub lsp_recommendation_disabled: Option<bool>,
    pub lsp_recommendation_never_plugins: Option<Vec<String>>,
    pub lsp_recommendation_ignored_count: Option<u32>,
    pub claude_code_hints: Option<ClaudeCodeHints>,
    pub permission_explainer_enabled: Option<bool>,
    pub teammate_mode: Option<String>,
    pub teammate_default_model: Option<String>,
    pub pr_status_footer_enabled: Option<bool>,
    pub tungsten_panel_visible: Option<bool>,
    pub penguin_mode_org_enabled: Option<bool>,
    pub startup_prefetched_at: Option<u64>,
    pub remote_control_at_startup: Option<bool>,
    pub cached_extra_usage_disabled_reason: Option<String>,
    pub auto_permissions_notification_count: Option<u32>,
    pub speculation_enabled: Option<bool>,
    pub client_data_cache: Option<serde_json::Value>,
    pub additional_model_options_cache: Option<Vec<serde_json::Value>>,
    pub metrics_status_cache: Option<MetricsStatusCache>,
    pub migration_version: Option<u32>,
}

/// Feedback survey state
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FeedbackSurveyState {
    pub last_shown_time: Option<u64>,
}

/// S1M access cache entry
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct S1mAccessCacheEntry {
    pub has_access: bool,
    pub has_access_not_as_default: Option<bool>,
    pub timestamp: u64,
}

/// Grove config cache entry
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GroveConfigCacheEntry {
    pub grove_enabled: bool,
    pub timestamp: u64,
}

/// Overage credit cache entry
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OverageCreditCacheEntry {
    pub info: OverageCreditInfo,
    pub timestamp: u64,
}

/// Overage credit info
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct OverageCreditInfo {
    pub available: bool,
    pub eligible: bool,
    pub granted: bool,
    pub amount_minor_units: Option<i64>,
    pub currency: Option<String>,
}

/// Skill usage entry
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SkillUsageEntry {
    pub usage_count: u32,
    pub last_used_at: u64,
}

/// Chrome extension state
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ChromeExtensionState {
    pub paired_device_id: Option<String>,
    pub paired_device_name: Option<String>,
}

/// Claude code hints
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ClaudeCodeHints {
    pub plugin: Option<Vec<String>>,
    pub disabled: Option<bool>,
}

/// Metrics status cache
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MetricsStatusCache {
    pub enabled: bool,
    pub timestamp: u64,
}

/// Custom API key responses
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CustomApiKeyResponses {
    pub approved: Option<Vec<String>>,
    pub rejected: Option<Vec<String>>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        GlobalConfig {
            num_startups: 0,
            install_method: None,
            auto_updates: None,
            theme: Some(ThemeSetting::Dark),
            preferred_notif_channel: NotificationChannel::Auto,
            verbose: Some(false),
            editor_mode: Some(EditorMode::Normal),
            auto_compact_enabled: true,
            show_turn_duration: true,
            queued_command_up_hint_count: Some(0),
            diff_tool: Some(DiffTool::Auto),
            custom_api_key_responses: Some(CustomApiKeyResponses {
                approved: Some(vec![]),
                rejected: Some(vec![]),
            }),
            env: HashMap::new(),
            tips_history: HashMap::new(),
            memory_usage_count: 0,
            prompt_queue_use_count: 0,
            btw_use_count: 0,
            todo_feature_enabled: true,
            show_expanded_todos: Some(false),
            message_idle_notif_threshold_ms: 60000,
            auto_connect_ide: Some(false),
            auto_install_ide_extension: Some(true),
            file_checkpointing_enabled: true,
            terminal_progress_bar_enabled: true,
            cached_statsig_gates: HashMap::new(),
            cached_dynamic_configs: Some(HashMap::new()),
            cached_growth_book_features: Some(HashMap::new()),
            respect_gitignore: true,
            copy_full_response: false,
            // All other fields are None/false/empty by default
            api_key_helper: None,
            projects: None,
            auto_updates_protected_for_native: None,
            doctor_shown_at_session: None,
            user_id: None,
            has_completed_onboarding: None,
            last_onboarding_version: None,
            last_release_notes_seen: None,
            changelog_last_fetched: None,
            cached_changelog: None,
            mcp_servers: None,
            claude_ai_mcp_ever_connected: None,
            custom_notify_command: None,
            primary_api_key: None,
            has_acknowledged_cost_threshold: None,
            has_seen_undercover_auto_notice: None,
            has_seen_ultraplan_terms: None,
            has_reset_auto_mode_opt_in_for_default_offer: None,
            oauth_account: None,
            iterm2_key_binding_installed: None,
            bypass_permissions_mode_accepted: None,
            has_used_backslash_return: None,
            has_seen_tasks_hint: None,
            has_used_stash: None,
            has_used_background_task: None,
            iterm2_setup_in_progress: None,
            iterm2_backup_path: None,
            apple_terminal_backup_path: None,
            apple_terminal_setup_in_progress: None,
            shift_enter_key_binding_installed: None,
            option_as_meta_key_installed: None,
            has_ide_onboarding_been_shown: None,
            ide_hint_shown_count: None,
            has_ide_auto_connect_dialog_been_shown: None,
            companion: None,
            companion_muted: None,
            feedback_survey_state: None,
            transcript_share_dismissed: None,
            has_shown_s1m_welcome_v2: None,
            s1m_access_cache: None,
            s1m_non_subscriber_access_cache: None,
            passes_eligibility_cache: None,
            grove_config_cache: None,
            passes_upsell_seen_count: None,
            has_visited_passes: None,
            passes_last_seen_remaining: None,
            overage_credit_grant_cache: None,
            overage_credit_upsell_seen_count: None,
            has_visited_extra_usage: None,
            voice_notice_seen_count: None,
            voice_lang_hint_shown_count: None,
            voice_lang_hint_last_language: None,
            voice_footer_hint_seen_count: None,
            opus_1m_merge_notice_seen_count: None,
            experiment_notices_seen_count: None,
            has_shown_opus_plan_welcome: None,
            last_plan_mode_use: None,
            subscription_notice_count: None,
            has_available_subscription: None,
            subscription_upsell_seen_count: None,
            recommended_subscription: None,
            show_spinner_tree: None,
            first_start_time: None,
            github_action_setup_count: None,
            slack_app_install_count: None,
            show_status_in_terminal_tab: None,
            task_complete_notif_enabled: None,
            input_needed_notif_enabled: None,
            agent_push_notif_enabled: None,
            claude_code_first_token_date: None,
            model_switch_callout_dismissed: None,
            model_switch_callout_last_shown: None,
            model_switch_callout_version: None,
            effort_callout_dismissed: None,
            effort_callout_v2_dismissed: None,
            remote_dialog_seen: None,
            bridge_oauth_dead_expires_at: None,
            bridge_oauth_dead_fail_count: None,
            desktop_upsell_seen_count: None,
            desktop_upsell_dismissed: None,
            idle_return_dismissed: None,
            opus_pro_migration_complete: None,
            opus_pro_migration_timestamp: None,
            sonnet_1m_45_migration_complete: None,
            legacy_opus_migration_timestamp: None,
            sonnet_45_to_46_migration_timestamp: None,
            growth_book_overrides: None,
            last_shown_emergency_tip: None,
            copy_on_select: None,
            github_repo_paths: None,
            deep_link_terminal: None,
            iterm2_it2_setup_complete: None,
            prefer_tmux_over_iterm2: None,
            skill_usage: None,
            official_marketplace_auto_install_attempted: None,
            official_marketplace_auto_installed: None,
            official_marketplace_auto_install_fail_reason: None,
            official_marketplace_auto_install_retry_count: None,
            official_marketplace_auto_install_last_attempt_time: None,
            official_marketplace_auto_install_next_retry_time: None,
            has_completed_claude_in_chrome_onboarding: None,
            claude_in_chrome_default_enabled: None,
            cached_chrome_extension_installed: None,
            chrome_extension: None,
            lsp_recommendation_disabled: None,
            lsp_recommendation_never_plugins: None,
            lsp_recommendation_ignored_count: None,
            claude_code_hints: None,
            permission_explainer_enabled: None,
            teammate_mode: None,
            teammate_default_model: None,
            pr_status_footer_enabled: None,
            tungsten_panel_visible: None,
            penguin_mode_org_enabled: None,
            startup_prefetched_at: None,
            remote_control_at_startup: None,
            cached_extra_usage_disabled_reason: None,
            auto_permissions_notification_count: None,
            speculation_enabled: None,
            client_data_cache: None,
            additional_model_options_cache: None,
            metrics_status_cache: None,
            migration_version: None,
        }
    }
}

/// Get the global config file path
pub fn get_global_config_path() -> PathBuf {
    // Use AI_ prefix for localization (AI_CONFIG_DIR or CLAUDE_CONFIG_DIR)
    let config_dir = std::env::var(ai::CONFIG_DIR)
        .or_else(|_| std::env::var(ai::CLAUDE_CONFIG_DIR))
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .map(|h| h.join(".ai").to_string_lossy().to_string())
                .unwrap_or_else(|| "~/.ai".to_string())
        });

    dirs::home_dir()
    .map(|h| h.join(".ai.json"))
    .unwrap_or_else(|| PathBuf::from(".ai.json"))
}

/// Load global config from file
pub fn get_global_config() -> GlobalConfig {
    let path = get_global_config_path();

    if !path.exists() {
        return GlobalConfig::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<GlobalConfig>(&content) {
            Ok(config) => {
                // Merge with defaults to ensure all fields are present
                let mut default_config = GlobalConfig::default();
                merge_config(&mut default_config, config);
                default_config
            }
            Err(e) => {
                eprintln!("Failed to parse config: {}", e);
                GlobalConfig::default()
            }
        },
        Err(e) => {
            eprintln!("Failed to read config file: {}", e);
            GlobalConfig::default()
        }
    }
}

/// Normalize a path for use as a JSON config key.
/// Converts backslashes to forward slashes for consistent JSON serialization
/// (Windows paths can be C:\path or C:/path depending on source).
fn normalize_path_for_config_key(path: &str) -> String {
    path.replace('\\', "/")
}

/// Find the git root for a given path (searches parents for .git directory)
fn find_git_root(path: &str) -> Option<String> {
    let mut current = std::path::Path::new(path);
    loop {
        if current.join(".git").exists() {
            return Some(current.to_string_lossy().to_string());
        }
        match current.parent() {
            Some(p) => current = p,
            None => return None,
        }
    }
}

/// Resolve a worktree .git file to the main repo root (follows gitdir: pointing to common dir)
fn resolve_canonical_git_root(git_root: &str) -> Option<String> {
    let git_dir = std::path::Path::new(git_root).join(".git");
    let git_dir_contents = std::fs::read_to_string(&git_dir).ok()?;
    for line in git_dir_contents.lines() {
        if line.starts_with("gitdir: ") {
            let common_dir = &line[8..].trim_end_matches('/');
            // The common dir is <main-repo>/.git/worktrees/<worktree-name>
            // We want <main-repo>
            if let Some(worktrees_idx) = common_dir.find("/worktrees/") {
                return Some(common_dir[..worktrees_idx].to_string());
            }
            // Fallback: strip trailing /.git or /git-dir
            let normalized = std::path::Path::new(common_dir)
                .parent()?
                .to_string_lossy()
                .to_string();
            return Some(normalized);
        }
    }
    Some(git_root.to_string())
}

/// Find the canonical git repository root, resolving through worktrees.
/// Unlike find_git_root (which returns the worktree directory), this returns
/// the main repository root so all worktrees of the same repo map to the same project.
fn find_canonical_git_root(start_path: &str) -> Option<String> {
    let root = find_git_root(start_path)?;
    Some(resolve_canonical_git_root(&root).unwrap_or(root))
}

/// Get the project path for config lookup (git root or cwd)
fn get_project_path_for_config() -> String {
    use crate::utils::cwd::get_original_cwd;

    let original_cwd = get_original_cwd();
    let original_cwd_str = original_cwd.to_string_lossy();

    // Try git root first
    if let Some(canonical) = find_canonical_git_root(&original_cwd_str) {
        return normalize_path_for_config_key(&canonical);
    }

    // Fall back to original cwd
    normalize_path_for_config_key(&original_cwd_str)
}

// Session-level trust cache: trust only transitions false->true during a session
static SESSION_TRUST_ACCEPTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Check if trust dialog has been accepted for this session.
/// Uses session-level cache (latched true) and checks global config paths.
pub fn check_has_trust_dialog_accepted() -> bool {
    // If session trust was already accepted, latch to true
    if SESSION_TRUST_ACCEPTED.load(std::sync::atomic::Ordering::SeqCst) {
        return true;
    }

    // Check session-level trust (for home directory case where trust is not persisted)
    // Note: get_session_trust_accepted() requires bootstrap module which isn't available in SDK.
    // SDK users set AI_CODE_SESSION_TRUST_ACCEPTED=1 to indicate trust was accepted.
    if std::env::var("AI_CODE_SESSION_TRUST_ACCEPTED").as_deref() == Ok("1") {
        SESSION_TRUST_ACCEPTED.store(true, std::sync::atomic::Ordering::SeqCst);
        return true;
    }

    let config = get_global_config();

    // Always check where trust would be saved (git root or original cwd)
    // This is the primary location where trust is persisted by save_current_project_config
    let project_path = get_project_path_for_config();
    if let Some(projects) = &config.projects {
        if let Some(project_config) = projects.get(&project_path) {
            if project_config.has_trust_dialog_accepted {
                SESSION_TRUST_ACCEPTED.store(true, std::sync::atomic::Ordering::SeqCst);
                return true;
            }
        }
    }

    // Now check from current working directory and its parents
    let cwd = crate::utils::cwd::get_cwd();
    let cwd_str = cwd.to_string_lossy();
    let mut current_path = normalize_path_for_config_key(&cwd_str);

    loop {
        if let Some(projects) = &config.projects {
            if let Some(project_config) = projects.get(&current_path) {
                if project_config.has_trust_dialog_accepted {
                    SESSION_TRUST_ACCEPTED.store(true, std::sync::atomic::Ordering::SeqCst);
                    return true;
                }
            }
        }

        // Move to parent directory
        let parent_path = std::path::Path::new(&current_path)
            .parent()
            .map(|p| normalize_path_for_config_key(&p.to_string_lossy()));

        match parent_path {
            Some(parent) if parent != current_path => current_path = parent,
            _ => break,
        }
    }

    false
}

/// Merge loaded config with defaults
fn merge_config(default: &mut GlobalConfig, loaded: GlobalConfig) {
    // This manually merges fields, preferring loaded values over defaults
    // for fields that are Some in loaded

    macro_rules! merge_option {
        ($field:ident) => {
            if let Some(v) = loaded.$field {
                default.$field = Some(v);
            }
        };
    }

    macro_rules! merge_hashmap {
        ($field:ident) => {
            if let Some(v) = loaded.$field {
                default.$field = v;
            }
        };
    }

    merge_option!(api_key_helper);
    merge_option!(projects);
    default.num_startups = loaded.num_startups;
    merge_option!(install_method);
    merge_option!(auto_updates);
    merge_option!(auto_updates_protected_for_native);
    merge_option!(doctor_shown_at_session);
    merge_option!(user_id);
    default.theme = loaded.theme;
    merge_option!(has_completed_onboarding);
    merge_option!(last_onboarding_version);
    merge_option!(last_release_notes_seen);
    merge_option!(changelog_last_fetched);
    merge_option!(cached_changelog);
    merge_option!(mcp_servers);
    merge_option!(claude_ai_mcp_ever_connected);
    default.preferred_notif_channel = loaded.preferred_notif_channel;
    merge_option!(custom_notify_command);
    merge_option!(verbose);
    merge_option!(custom_api_key_responses);
    merge_option!(primary_api_key);
    merge_option!(has_acknowledged_cost_threshold);
    merge_option!(has_seen_undercover_auto_notice);
    merge_option!(has_seen_ultraplan_terms);
    merge_option!(has_reset_auto_mode_opt_in_for_default_offer);
    merge_option!(oauth_account);
    merge_option!(iterm2_key_binding_installed);
    merge_option!(editor_mode);
    merge_option!(bypass_permissions_mode_accepted);
    merge_option!(has_used_backslash_return);
    default.auto_compact_enabled = loaded.auto_compact_enabled;
    default.show_turn_duration = loaded.show_turn_duration;
    default.env = loaded.env;
    merge_option!(has_seen_tasks_hint);
    merge_option!(has_used_stash);
    merge_option!(has_used_background_task);
    merge_option!(queued_command_up_hint_count);
    merge_option!(diff_tool);
    merge_option!(iterm2_setup_in_progress);
    merge_option!(iterm2_backup_path);
    merge_option!(apple_terminal_backup_path);
    merge_option!(apple_terminal_setup_in_progress);
    merge_option!(shift_enter_key_binding_installed);
    merge_option!(option_as_meta_key_installed);
    merge_option!(auto_connect_ide);
    merge_option!(auto_install_ide_extension);
    merge_option!(has_ide_onboarding_been_shown);
    merge_option!(ide_hint_shown_count);
    merge_option!(has_ide_auto_connect_dialog_been_shown);
    default.tips_history = loaded.tips_history;
    merge_option!(companion);
    merge_option!(companion_muted);
    merge_option!(feedback_survey_state);
    merge_option!(transcript_share_dismissed);
    default.memory_usage_count = loaded.memory_usage_count;
    merge_option!(has_shown_s1m_welcome_v2);
    merge_option!(s1m_access_cache);
    merge_option!(s1m_non_subscriber_access_cache);
    merge_option!(passes_eligibility_cache);
    merge_option!(grove_config_cache);
    merge_option!(passes_upsell_seen_count);
    merge_option!(has_visited_passes);
    merge_option!(passes_last_seen_remaining);
    merge_option!(overage_credit_grant_cache);
    merge_option!(overage_credit_upsell_seen_count);
    merge_option!(has_visited_extra_usage);
    merge_option!(voice_notice_seen_count);
    merge_option!(voice_lang_hint_shown_count);
    merge_option!(voice_lang_hint_last_language);
    merge_option!(voice_footer_hint_seen_count);
    merge_option!(opus_1m_merge_notice_seen_count);
    merge_option!(experiment_notices_seen_count);
    merge_option!(has_shown_opus_plan_welcome);
    default.prompt_queue_use_count = loaded.prompt_queue_use_count;
    default.btw_use_count = loaded.btw_use_count;
    merge_option!(last_plan_mode_use);
    merge_option!(subscription_notice_count);
    merge_option!(has_available_subscription);
    merge_option!(subscription_upsell_seen_count);
    merge_option!(recommended_subscription);
    default.todo_feature_enabled = loaded.todo_feature_enabled;
    merge_option!(show_expanded_todos);
    merge_option!(show_spinner_tree);
    merge_option!(first_start_time);
    default.message_idle_notif_threshold_ms = loaded.message_idle_notif_threshold_ms;
    merge_option!(github_action_setup_count);
    merge_option!(slack_app_install_count);
    default.file_checkpointing_enabled = loaded.file_checkpointing_enabled;
    default.terminal_progress_bar_enabled = loaded.terminal_progress_bar_enabled;
    merge_option!(show_status_in_terminal_tab);
    merge_option!(task_complete_notif_enabled);
    merge_option!(input_needed_notif_enabled);
    merge_option!(agent_push_notif_enabled);
    merge_option!(claude_code_first_token_date);
    merge_option!(model_switch_callout_dismissed);
    merge_option!(model_switch_callout_last_shown);
    merge_option!(model_switch_callout_version);
    merge_option!(effort_callout_dismissed);
    merge_option!(effort_callout_v2_dismissed);
    merge_option!(remote_dialog_seen);
    merge_option!(bridge_oauth_dead_expires_at);
    merge_option!(bridge_oauth_dead_fail_count);
    merge_option!(desktop_upsell_seen_count);
    merge_option!(desktop_upsell_dismissed);
    merge_option!(idle_return_dismissed);
    merge_option!(opus_pro_migration_complete);
    merge_option!(opus_pro_migration_timestamp);
    merge_option!(sonnet_1m_45_migration_complete);
    merge_option!(legacy_opus_migration_timestamp);
    merge_option!(sonnet_45_to_46_migration_timestamp);
    default.cached_statsig_gates = loaded.cached_statsig_gates;
    merge_option!(cached_dynamic_configs);
    merge_option!(cached_growth_book_features);
    merge_option!(growth_book_overrides);
    merge_option!(last_shown_emergency_tip);
    default.respect_gitignore = loaded.respect_gitignore;
    default.copy_full_response = loaded.copy_full_response;
    merge_option!(copy_on_select);
    merge_option!(github_repo_paths);
    merge_option!(deep_link_terminal);
    merge_option!(iterm2_it2_setup_complete);
    merge_option!(prefer_tmux_over_iterm2);
    merge_option!(skill_usage);
    merge_option!(official_marketplace_auto_install_attempted);
    merge_option!(official_marketplace_auto_installed);
    merge_option!(official_marketplace_auto_install_fail_reason);
    merge_option!(official_marketplace_auto_install_retry_count);
    merge_option!(official_marketplace_auto_install_last_attempt_time);
    merge_option!(official_marketplace_auto_install_next_retry_time);
    merge_option!(has_completed_claude_in_chrome_onboarding);
    merge_option!(claude_in_chrome_default_enabled);
    merge_option!(cached_chrome_extension_installed);
    merge_option!(chrome_extension);
    merge_option!(lsp_recommendation_disabled);
    merge_option!(lsp_recommendation_never_plugins);
    merge_option!(lsp_recommendation_ignored_count);
    merge_option!(claude_code_hints);
    merge_option!(permission_explainer_enabled);
    merge_option!(teammate_mode);
    merge_option!(teammate_default_model);
    merge_option!(pr_status_footer_enabled);
    merge_option!(tungsten_panel_visible);
    merge_option!(penguin_mode_org_enabled);
    merge_option!(startup_prefetched_at);
    merge_option!(remote_control_at_startup);
    merge_option!(cached_extra_usage_disabled_reason);
    merge_option!(auto_permissions_notification_count);
    merge_option!(speculation_enabled);
    merge_option!(client_data_cache);
    merge_option!(additional_model_options_cache);
    merge_option!(metrics_status_cache);
    merge_option!(migration_version);
}

/// Remove null and default values from JSON string (for .ai.json cleanliness)
fn remove_nulls(json: &str) -> String {
    let value: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return json.to_string(),
    };
    let cleaned = remove_defaults_impl(&value);
    serde_json::to_string_pretty(&cleaned)
        .map(|s| s.to_string())
        .unwrap_or_else(|_| json.to_string())
}

fn remove_defaults_impl(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let filtered: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .filter(|(_, v)| !is_default_value(v))
                .map(|(k, v)| (k.clone(), remove_defaults_impl(v)))
                .collect();
            serde_json::Value::Object(filtered)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(remove_defaults_impl).collect())
        }
        _ => value.clone(),
    }
}

/// Check if a value is a default that should be stripped from settings.json
fn is_default_value(value: &serde_json::Value) -> bool {
    match value {
        // Null values
        serde_json::Value::Null => true,
        // Boolean false
        serde_json::Value::Bool(b) if !b => true,
        // Number zero
        serde_json::Value::Number(n) if n.as_i64() == Some(0) => true,
        // Empty strings
        serde_json::Value::String(s) if s.is_empty() => true,
        // Empty arrays
        serde_json::Value::Array(arr) if arr.is_empty() => true,
        // Empty objects
        serde_json::Value::Object(obj) if obj.is_empty() => true,
        _ => false,
    }
}

/// Save global config to file
pub fn save_global_config(config: &GlobalConfig) -> Result<(), String> {
    let path = get_global_config_path();

    // Ensure the directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    // Remove null values from JSON
    let json = remove_nulls(&json);

    fs::write(&path, json).map_err(|e| format!("Failed to write config file: {}", e))?;

    // Update cache
    if let Ok(mut cache) = GLOBAL_CONFIG_CACHE.lock() {
        cache.config = Some(config.clone());
        cache.mtime = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
    }

    Ok(())
}

/// Get project config for current directory
pub fn get_current_project_config() -> ProjectConfig {
    let global_config = get_global_config();

    // Try to get project path from environment or use current directory
    let project_path = std::env::var(ai::PROJECT_PATH)
        .or_else(|_| std::env::var(ai::CLAUDE_PROJECT_PATH))
        .unwrap_or_else(|_err| {
            std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        });

    global_config
        .projects
        .and_then(|p| p.get(&project_path).cloned())
        .unwrap_or_default()
}

/// Save project config for current directory
pub fn save_current_project_config(config: ProjectConfig) -> Result<(), String> {
    let project_path = std::env::var(ai::PROJECT_PATH)
        .or_else(|_| std::env::var(ai::CLAUDE_PROJECT_PATH))
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        });

    let mut global_config = get_global_config();

    let projects = global_config.projects.get_or_insert_with(HashMap::new);
    projects.insert(project_path, config);

    save_global_config(&global_config)
}

/// Get or create user ID
pub fn get_or_create_user_id() -> String {
    let mut config = get_global_config();

    if let Some(user_id) = &config.user_id {
        return user_id.clone();
    }

    // Generate new user ID
    let user_id = uuid::Uuid::new_v4().to_string();
    config.user_id = Some(user_id.clone());

    let _ = save_global_config(&config);

    user_id
}

/// Auto-updater disabled reason
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "envVar")]
pub enum AutoUpdaterDisabledReason {
    Development,
    Env { env_var: String },
    Config,
}

/// Get auto-updater disabled reason
pub fn get_auto_updater_disabled_reason() -> Option<AutoUpdaterDisabledReason> {
    // Check for development mode
    if std::env::var(system::NODE_ENV)
        .map(|v| v == "development")
        .unwrap_or(false)
    {
        return Some(AutoUpdaterDisabledReason::Development);
    }

    // Check for DISABLE_AUTOUPDATER env var (with AI_ prefix support)
    if std::env::var(ai::DISABLE_AUTOUPDATER)
        .or_else(|_| std::env::var(system::DISABLE_AUTOUPDATER))
        .map(|v| !v.is_empty() && v != "false")
        .unwrap_or(false)
    {
        return Some(AutoUpdaterDisabledReason::Env {
            env_var: "DISABLE_AUTOUPDATER".to_string(),
        });
    }

    // Check config
    let config = get_global_config();
    if config.auto_updates == Some(false)
        && (config.install_method != Some(InstallMethod::Native)
            || config.auto_updates_protected_for_native != Some(true))
    {
        return Some(AutoUpdaterDisabledReason::Config);
    }

    None
}

/// Check if auto-updater is disabled
pub fn is_auto_updater_disabled() -> bool {
    get_auto_updater_disabled_reason().is_some()
}

/// Get custom API key status
pub fn get_custom_api_key_status(truncated_api_key: &str) -> &'static str {
    let config = get_global_config();

    if let Some(responses) = &config.custom_api_key_responses {
        if let Some(approved) = &responses.approved {
            if approved.contains(&truncated_api_key.to_string()) {
                return "approved";
            }
        }
        if let Some(rejected) = &responses.rejected {
            if rejected.contains(&truncated_api_key.to_string()) {
                return "rejected";
            }
        }
    }

    "new"
}

/// Record first start time
pub fn record_first_start_time() {
    let mut config = get_global_config();

    if config.first_start_time.is_none() {
        config.first_start_time = Some(chrono::Utc::now().to_rfc3339());
        let _ = save_global_config(&config);
    }
}

/// Complete the onboarding flow - marks onboarding as done
pub fn complete_onboarding(version: &str) {
    let mut config = get_global_config();
    config.has_completed_onboarding = Some(true);
    config.last_onboarding_version = Some(version.to_string());
    let _ = save_global_config(&config);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GlobalConfig::default();
        assert_eq!(config.num_startups, 0);
        assert_eq!(config.theme, Some(ThemeSetting::Dark));
        assert_eq!(config.verbose, Some(false));
        assert!(config.auto_compact_enabled);
    }

    #[test]
    fn test_get_global_config_path() {
        let path = get_global_config_path();
        assert!(path.to_string_lossy().contains(".ai"));
    }

    #[test]
    fn test_is_auto_updater_disabled() {
        // Without env vars set, should not be disabled
        let _ = is_auto_updater_disabled();
    }

    #[test]
    fn test_get_custom_api_key_status() {
        assert_eq!(get_custom_api_key_status("test-key"), "new");
    }
}
