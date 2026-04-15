// Source: /data/home/swei/claudecode/openclaudecode/src/utils/env.ts
//! Environment variable name constants for Claude Code / AI Agent SDK.
//!
//! This module consolidates all environment variable names used throughout
//! the codebase that have `CLAUDE_`, `AI_`, or `ANTHROPIC_` prefixes.

/// AI_* environment variables
pub mod ai {
    /// API configuration
    pub const API_BASE_URL: &str = "AI_API_BASE_URL";
    pub const API_PROVIDER: &str = "AI_API_PROVIDER";
    pub const AUTH_TOKEN: &str = "AI_AUTH_TOKEN";
    pub const MODEL: &str = "AI_MODEL";

    /// Model defaults
    pub const SMALL_FAST_MODEL: &str = "AI_SMALL_FAST_MODEL";

    /// Bridge mode
    pub const BRIDGE_MODE: &str = "AI_BRIDGE_MODE";
    pub const CCR_AUTO_CONNECT: &str = "AI_CCR_AUTO_CONNECT";
    pub const CCR_MIRROR: &str = "AI_CCR_MIRROR";
    pub const ORGANIZATION_UUID: &str = "AI_ORGANIZATION_UUID";

    /// Bridge auth
    pub const BRIDGE_OAUTH_TOKEN: &str = "AI_BRIDGE_OAUTH_TOKEN";
    pub const BRIDGE_BASE_URL: &str = "AI_BRIDGE_BASE_URL";
    pub const OAUTH_TOKEN: &str = "AI_OAUTH_TOKEN";

    /// Compact settings
    pub const CONTEXT_WINDOW: &str = "AI_CONTEXT_WINDOW";
    pub const BLOCKING_LIMIT_OVERRIDE: &str = "AI_BLOCKING_LIMIT_OVERRIDE";
    pub const AUTO_COMPACT_WINDOW: &str = "AI_AUTO_COMPACT_WINDOW";
    pub const AUTOCOMPACT_PCT_OVERRIDE: &str = "AI_AUTOCOMPACT_PCT_OVERRIDE";
    pub const DISABLE_COMPACT: &str = "AI_DISABLE_COMPACT";
    pub const DISABLE_BACKGROUND_TASKS: &str = "AI_CODE_DISABLE_BACKGROUND_TASKS";

    /// Upstream proxy
    pub const UPSTREAM_PROXY: &str = "AI_UPSTREAM_PROXY";

    /// Assistant mode
    pub const CODE_ASSISTANT_MODE: &str = "AI_CODE_ASSISTANT_MODE";

    /// Memory
    pub const REMOTE_MEMORY_DIR: &str = "AI_REMOTE_MEMORY_DIR";
    pub const MEMORY_PATH_OVERRIDE: &str = "AI_MEMORY_PATH_OVERRIDE";
    pub const DISABLE_AUTO_MEMORY: &str = "AI_DISABLE_AUTO_MEMORY";
    pub const SIMPLE: &str = "AI_SIMPLE";
    pub const REMOTE: &str = "AI_REMOTE";
    pub const COWORK_MEMORY_PATH_OVERRIDE: &str = "AI_COWORK_MEMORY_PATH_OVERRIDE";

    /// Code memory
    pub const CODE_DISABLE_AUTO_MEMORY: &str = "AI_CODE_DISABLE_AUTO_MEMORY";
    pub const CODE_SIMPLE: &str = "AI_CODE_SIMPLE";
    pub const CODE_REMOTE: &str = "AI_CODE_REMOTE";
    pub const CODE_REMOTE_MEMORY_DIR: &str = "AI_CODE_REMOTE_MEMORY_DIR";

    /// Extract memories
    pub const DISABLE_EXTRACT_MEMORIES: &str = "AI_DISABLE_EXTRACT_MEMORIES";

    /// Config directory
    pub const CONFIG_DIR: &str = "AI_CONFIG_DIR";

    /// Shell
    pub const SHELL_PREFIX: &str = "AI_SHELL_PREFIX";

    /// User identification
    pub const USER_ID: &str = "AI_USER_ID";

    /// Feature flags
    pub const CODE_FEATURE_BRIDGE_MODE: &str = "AI_CODE_FEATURE_BRIDGE_MODE";
    pub const CODE_FEATURE_FORK_SUBAGENT: &str = "AI_CODE_FEATURE_FORK_SUBAGENT";

    /// Entry point
    pub const CODE_ENTRYPOINT: &str = "AI_CODE_ENTRYPOINT";

    /// Session
    pub const CODE_SESSION_ID: &str = "AI_CODE_SESSION_ID";
    pub const CODE_MODEL: &str = "AI_CODE_MODEL";

    /// Context
    pub const CODE_DISABLE_1M_CONTEXT: &str = "AI_CODE_DISABLE_1M_CONTEXT";
    pub const CODE_MAX_CONTEXT_TOKENS: &str = "AI_CODE_MAX_CONTEXT_TOKENS";

    /// Fullscreen
    pub const FULLSCREEN: &str = "AI_FULLSCREEN";

    /// Bash
    pub const BASH_MAINTAIN_PROJECT_WORKING_DIR: &str = "AI_BASH_MAINTAIN_PROJECT_WORKING_DIR";

    /// AWS region
    pub const AWS_REGION: &str = "AI_AWS_REGION";
    pub const AWS_DEFAULT_REGION: &str = "AI_AWS_DEFAULT_REGION";

    /// Cloud ML
    pub const CLOUD_ML_REGION: &str = "AI_CLOUD_ML_REGION";

    /// Thinking
    pub const ULTRATHINK: &str = "AI_ULTRATHINK";
    pub const MAX_THINKING_TOKENS: &str = "AI_MAX_THINKING_TOKENS";

    /// Windows paths
    pub const CODE_GIT_BASH_PATH: &str = "AI_CODE_GIT_BASH_PATH";

    /// Debug
    pub const CODE_DEBUG_LOG_LEVEL: &str = "AI_CODE_DEBUG_LOG_LEVEL";
    pub const CODE_DEBUG_LOGS_DIR: &str = "AI_CODE_DEBUG_LOGS_DIR";

    /// Plan mode
    pub const CODE_PLAN_MODE_V2: &str = "AI_CODE_PLAN_MODE_V2";
    pub const CODE_PLAN_MODE_AGENT_COUNT: &str = "AI_CODE_PLAN_MODE_AGENT_COUNT";
    pub const CODE_PLAN_MODE_EXPLORE_AGENT_COUNT: &str = "AI_CODE_PLAN_MODE_EXPLORE_AGENT_COUNT";
    pub const CODE_PLAN_MODE_INTERVIEW: &str = "AI_CODE_PLAN_MODE_INTERVIEW";
    pub const CODE_PEWTER_LEDGER_VARIANT: &str = "AI_CODE_PEWTER_LEDGER_VARIANT";

    /// Swarm
    pub const TEAMMATE_COMMAND: &str = "AI_TEAMMATE_COMMAND";
    pub const TEAMMATE_COLOR: &str = "AI_AGENT_COLOR";
    pub const PLAN_MODE_REQUIRED: &str = "AI_PLAN_MODE_REQUIRED";

    /// Ultrareview
    pub const ULTRAREVIEW_ENABLED: &str = "AI_ULTRAREVIEW_ENABLED";

    /// Session ingress auth
    pub const CODE_INGRESS_AUTH_REQUIRED: &str = "AI_CODE_INGRESS_AUTH_REQUIRED";
    pub const CODE_INGRESS_TOKEN: &str = "AI_CODE_INGRESS_TOKEN";

    /// User type
    pub const USER_TYPE: &str = "USER_TYPE";

    /// Worktree mode
    pub const WORKTREE_MODE: &str = "AI_WORKTREE_MODE";
    pub const WORKTREE_ROOT: &str = "AI_WORKTREE_ROOT";
    pub const WORKTREE_SLUG: &str = "AI_WORKTREE_SLUG";
    pub const ORIGINAL_CWD: &str = "AI_ORIGINAL_CWD";

    /// Project path
    pub const PROJECT_PATH: &str = "AI_PROJECT_PATH";

    /// Disable autoupdater
    pub const DISABLE_AUTOUPDATER: &str = "AI_DISABLE_AUTOUPDATER";

    /// Version
    pub const VERSION: &str = "AI_CODE_VERSION";

    /// Claude config home (for fallback)
    pub const CLAUDE_CONFIG_HOME: &str = "CLAUDE_CONFIG_HOME";
    pub const CLAUDE_CONFIG_DIR: &str = "CLAUDE_CONFIG_DIR";
    pub const CLAUDE_PROJECT_PATH: &str = "CLAUDE_PROJECT_PATH";

    /// AI Agent SDK
    pub const AGENT_SDK_VERSION: &str = "AI_AGENT_SDK_VERSION";
    pub const AGENT_SDK_CLIENT_APP: &str = "AI_AGENT_SDK_CLIENT_APP";

    /// Claude user ID
    pub const CLAUDE_USER_ID: &str = "CLAUDE_USER_ID";

    /// Claude trusted device token
    pub const CLAUDE_TRUSTED_DEVICE_TOKEN: &str = "CLAUDE_TRUSTED_DEVICE_TOKEN";

    /// Claude local OAuth
    pub const USE_LOCAL_OAUTH: &str = "USE_LOCAL_OAUTH";
    pub const CLAUDE_LOCAL_OAUTH_API_BASE: &str = "CLAUDE_LOCAL_OAUTH_API_BASE";
    pub const USE_STAGING_OAUTH: &str = "USE_STAGING_OAUTH";

    /// Mock headerless 429
    pub const CLAUDE_MOCK_HEADERLESS_429: &str = "CLAUDE_MOCK_HEADERLESS_429";

    /// Disable explore plan agents
    pub const DISABLE_EXPLORE_PLAN_AGENTS: &str = "DISABLE_EXPLORE_PLAN_AGENTS";

    /// Max tool use concurrency
    pub const MAX_TOOL_USE_CONCURRENCY: &str = "AI_CODE_MAX_TOOL_USE_CONCURRENCY";

    /// Dynamic env
    pub fn dynamic_key(key: &str) -> String {
        format!("AI_DYNAMIC_{}", key)
    }

    /// Vertex region mapping
    pub mod vertex {
        pub const REGION_CLAUDE_HAIKU_4_5: &str = "AI_VERTEX_REGION_CLAUDE_HAIKU_4_5";
        pub const REGION_CLAUDE_3_5_HAIKU: &str = "AI_VERTEX_REGION_CLAUDE_3_5_HAIKU";
        pub const REGION_CLAUDE_3_5_SONNET: &str = "AI_VERTEX_REGION_CLAUDE_3_5_SONNET";
        pub const REGION_CLAUDE_3_7_SONNET: &str = "AI_VERTEX_REGION_CLAUDE_3_7_SONNET";
        pub const REGION_CLAUDE_4_1_OPUS: &str = "AI_VERTEX_REGION_CLAUDE_4_1_OPUS";
        pub const REGION_CLAUDE_4_0_OPUS: &str = "AI_VERTEX_REGION_CLAUDE_4_0_OPUS";
        pub const REGION_CLAUDE_4_6_SONNET: &str = "AI_VERTEX_REGION_CLAUDE_4_6_SONNET";
        pub const REGION_CLAUDE_4_5_SONNET: &str = "AI_VERTEX_REGION_CLAUDE_4_5_SONNET";
        pub const REGION_CLAUDE_4_0_SONNET: &str = "AI_VERTEX_REGION_CLAUDE_4_0_SONNET";
    }

    /// Config home
    pub const CONFIG_HOME: &str = "AI_CONFIG_HOME";

    // Anthropic API (merged from anthropic module)
    // Missing constants from the original anthropic module
    pub const API_KEY: &str = "AI_API_KEY";
    pub const BASE_URL: &str = "AI_BASE_URL";
    pub const UNIX_SOCKET: &str = "AI_UNIX_SOCKET";
    pub const DEFAULT_OPUS_MODEL: &str = "AI_DEFAULT_OPUS_MODEL";
    pub const DEFAULT_SONNET_MODEL: &str = "AI_DEFAULT_SONNET_MODEL";
    pub const DEFAULT_HAIKU_MODEL: &str = "AI_DEFAULT_HAIKU_MODEL";
    pub const CUSTOM_MODEL_OPTION: &str = "AI_CUSTOM_MODEL_OPTION";
    pub const BEDROCK_BASE_URL: &str = "AI_BEDROCK_BASE_URL";
    pub const VERTEX_BASE_URL: &str = "AI_VERTEX_BASE_URL";
    pub const VERTEX_PROJECT_ID: &str = "AI_VERTEX_PROJECT_ID";
    pub const FOUNDRY_BASE_URL: &str = "AI_FOUNDRY_BASE_URL";
    pub const FOUNDRY_RESOURCE: &str = "AI_FOUNDRY_RESOURCE";
    pub const FOUNDRY_API_KEY: &str = "AI_FOUNDRY_API_KEY";
    // Aliases with ANTHROPIC_ prefix for backward compatibility
    pub const ANTHROPIC_BASE_URL: &str = "AI_BASE_URL";
    pub const ANTHROPIC_API_KEY: &str = "AI_API_KEY";
    pub const ANTHROPIC_AUTH_TOKEN: &str = "AI_AUTH_TOKEN";
    pub const ANTHROPIC_MODEL: &str = "AI_MODEL";
    pub const ANTHROPIC_UNIX_SOCKET: &str = "AI_UNIX_SOCKET";
    pub const ANTHROPIC_DEFAULT_OPUS_MODEL: &str = "AI_DEFAULT_OPUS_MODEL";
    pub const ANTHROPIC_DEFAULT_SONNET_MODEL: &str = "AI_DEFAULT_SONNET_MODEL";
    pub const ANTHROPIC_DEFAULT_HAIKU_MODEL: &str = "AI_DEFAULT_HAIKU_MODEL";
    pub const ANTHROPIC_CUSTOM_MODEL_OPTION: &str = "AI_CUSTOM_MODEL_OPTION";
    pub const ANTHROPIC_BEDROCK_BASE_URL: &str = "AI_BEDROCK_BASE_URL";
    pub const ANTHROPIC_VERTEX_BASE_URL: &str = "AI_VERTEX_BASE_URL";
    pub const ANTHROPIC_VERTEX_PROJECT_ID: &str = "AI_VERTEX_PROJECT_ID";
    pub const ANTHROPIC_FOUNDRY_BASE_URL: &str = "AI_FOUNDRY_BASE_URL";
    pub const ANTHROPIC_FOUNDRY_RESOURCE: &str = "AI_FOUNDRY_RESOURCE";
    pub const ANTHROPIC_FOUNDRY_API_KEY: &str = "AI_FOUNDRY_API_KEY";
}

/// AI_CODE_* environment variables (merged from claude_code module)
pub mod ai_code {
    // OAuth and auth
    pub const OAUTH_TOKEN: &str = "AI_CODE_OAUTH_TOKEN";
    pub const ACCESS_TOKEN: &str = "AI_CODE_ACCESS_TOKEN";
    pub const ORG_UUID: &str = "AI_CODE_ORG_UUID";
    pub const CUSTOM_OAUTH_URL: &str = "AI_CODE_CUSTOM_OAUTH_URL";
    pub const OAUTH_CLIENT_ID: &str = "AI_CODE_OAUTH_CLIENT_ID";
    pub const LOCAL_OAUTH_API_BASE: &str = "AI_LOCAL_OAUTH_API_BASE";
    pub const LOCAL_OAUTH_APPS_BASE: &str = "AI_LOCAL_OAUTH_APPS_BASE";
    pub const LOCAL_OAUTH_CONSOLE_BASE: &str = "AI_LOCAL_OAUTH_CONSOLE_BASE";

    // Subscriber
    pub const SUBSCRIBER: &str = "AI_CODE_SUBSCRIBER";

    // Coordinator mode
    pub const COORDINATOR_MODE: &str = "AI_CODE_COORDINATOR_MODE";

    // Assistant mode - use ai::CODE_ASSISTANT_MODE instead
    // pub const ASSISTANT_MODE: &str = "AI_CODE_ASSISTANT_MODE";

    // Entrypoint - use ai::CODE_ENTRYPOINT instead

    // Date override
    pub const OVERRIDE_DATE: &str = "AI_CODE_OVERRIDE_DATE";

    // Session - use ai::CODE_SESSION_ID instead
    // pub const SESSION_ID: &str = "AI_CODE_SESSION_ID";
    pub const SESSION_ACCESS_TOKEN: &str = "AI_CODE_SESSION_ACCESS_TOKEN";

    // Config - use ai::CONFIG_DIR and ai::CONFIG_HOME instead
    // pub const CONFIG_DIR: &str = "AI_CONFIG_DIR";
    // pub const CONFIG_HOME: &str = "AI_CONFIG_HOME";

    // Subprocess env
    pub const SUBPROCESS_ENV_SCRUB: &str = "AI_CODE_SUBPROCESS_ENV_SCRUB";

    // Fast mode
    pub const DISABLE_FAST_MODE: &str = "AI_CODE_DISABLE_FAST_MODE";

    // Legacy model remap
    pub const DISABLE_LEGACY_MODEL_REMAP: &str = "AI_CODE_DISABLE_LEGACY_MODEL_REMAP";

    // Simple mode
    pub const SIMPLE: &str = "AI_CODE_SIMPLE";

    // Privacy / traffic
    pub const DISABLE_NONESSENTIAL_TRAFFIC: &str = "AI_CODE_DISABLE_NONESSENTIAL_TRAFFIC";

    // Trusted device
    pub const TRUSTED_DEVICE_TOKEN: &str = "AI_TRUSTED_DEVICE_TOKEN";

    // Bridge - use ai::BRIDGE_OAUTH_TOKEN and ai::BRIDGE_BASE_URL instead
    // pub const BRIDGE_OAUTH_TOKEN: &str = "AI_BRIDGE_OAUTH_TOKEN";
    // pub const BRIDGE_BASE_URL: &str = "AI_BRIDGE_BASE_URL";
    pub const CCR_MIRROR: &str = "AI_CODE_CCR_MIRROR";

    // Session runner
    pub const ENVIRONMENT_KIND: &str = "AI_CODE_ENVIRONMENT_KIND";
    pub const FORCE_SANDBOX: &str = "AI_CODE_FORCE_SANDBOX";
    pub const POST_FOR_SESSION_INGRESS_V2: &str = "AI_CODE_POST_FOR_SESSION_INGRESS_V2";
    pub const USE_CCR_V2: &str = "AI_CODE_USE_CCR_V2";
    pub const WORKER_EPOCH: &str = "AI_CODE_WORKER_EPOCH";

    // VCR testing
    pub const TEST_FIXTURES_ROOT: &str = "AI_CODE_TEST_FIXTURES_ROOT";

    // Auth
    pub const API_KEY_FILE_DESCRIPTOR: &str = "AI_CODE_API_KEY_FILE_DESCRIPTOR";
    pub const OAUTH_TOKEN_FILE_DESCRIPTOR: &str = "AI_CODE_OAUTH_TOKEN_FILE_DESCRIPTOR";
    pub const BARE: &str = "AI_CODE_BARE";
    pub const HOMESPACE: &str = "AI_CODE_HOMESPACE";
    pub const PREFER_THIRD_PARTY: &str = "AI_CODE_PREFER_THIRD_PARTY";
    pub const NON_INTERACTIVE: &str = "AI_CODE_NON_INTERACTIVE";

    // Attachments
    pub const DISABLE_ATTACHMENTS: &str = "AI_CODE_DISABLE_ATTACHMENTS";
    pub const PLAN_MODE_EXITED: &str = "AI_CODE_PLAN_MODE_EXITED";

    // API features
    pub const ENABLE_FINE_GRAINED_TOOL_STREAMING: &str = "AI_CODE_ENABLE_FINE_GRAINED_TOOL_STREAMING";
    pub const ENABLE_AGENT_SWARMS: &str = "AI_CODE_ENABLE_AGENT_SWARMS";
    pub const USE_BEDROCK: &str = "AI_CODE_USE_BEDROCK";
    pub const USE_VERTEX: &str = "AI_CODE_USE_VERTEX";
    pub const USE_FOUNDRY: &str = "AI_CODE_USE_FOUNDRY";

    // Streaming
    pub const ENABLE_STREAM_WATCHDOG: &str = "AI_CODE_ENABLE_STREAM_WATCHDOG";
    pub const STREAM_IDLE_TIMEOUT_MS: &str = "AI_CODE_STREAM_IDLE_TIMEOUT_MS";
    pub const DISABLE_NONSTREAMING_FALLBACK: &str = "AI_CODE_DISABLE_NONSTREAMING_FALLBACK";
    pub const API_TIMEOUT_MS: &str = "AI_CODE_API_TIMEOUT_MS";

    // Terminal - use ai::ORIGINAL_CWD instead (no ai equivalent for TERMINAL_RECORDING)
    pub const TERMINAL_RECORDING: &str = "AI_CODE_TERMINAL_RECORDING";
    // pub const ORIGINAL_CWD: &str = "AI_CODE_ORIGINAL_CWD";

    // Swarm - use ai::TEAMMATE_COMMAND, ai::TEAMMATE_COLOR, ai::PLAN_MODE_REQUIRED instead
    // pub const TEAMMATE_COMMAND: &str = "AI_CODE_TEAMMATE_COMMAND";
    // pub const TEAMMATE_COLOR: &str = "AI_CODE_AGENT_COLOR";
    // pub const PLAN_MODE_REQUIRED: &str = "AI_CODE_PLAN_MODE_REQUIRED";

    // Mock rate limits - use ai::CLAUDE_MOCK_HEADERLESS_429 instead
    // pub const MOCK_HEADERLESS_429: &str = "AI_MOCK_HEADERLESS_429";

    // Diagnostics
    pub const DIAGNOSTICS_FILE: &str = "AI_CODE_DIAGNOSTICS_FILE";

    // User ID - use ai::USER_ID instead
    // pub const USER_ID: &str = "AI_USER_ID";

    // Existing constants
    pub const PROVIDER_MANAGED_BY_HOST: &str = "AI_CODE_PROVIDER_MANAGED_BY_HOST";
    pub const SUBAGENT_MODEL: &str = "AI_CODE_SUBAGENT_MODEL";
    pub const SKIP_BEDROCK_AUTH: &str = "AI_CODE_SKIP_BEDROCK_AUTH";
    pub const SKIP_VERTEX_AUTH: &str = "AI_CODE_SKIP_VERTEX_AUTH";
    pub const SKIP_FOUNDRY_AUTH: &str = "AI_CODE_SKIP_FOUNDRY_AUTH";
    // pub const BASH_MAINTAIN_PROJECT_WORKING_DIR: &str = "AI_CODE_BASH_MAINTAIN_PROJECT_WORKING_DIR"; // use ai::BASH_MAINTAIN_PROJECT_WORKING_DIR
    pub const API_KEY_HELPER_TTL_MS: &str = "AI_CODE_API_KEY_HELPER_TTL_MS";
    pub const DISABLE_EXPERIMENTAL_BETAS: &str = "AI_CODE_DISABLE_EXPERIMENTAL_BETAS";
    pub const DISABLE_TERMINAL_TITLE: &str = "AI_CODE_DISABLE_TERMINAL_TITLE";
    pub const ENABLE_TELEMETRY: &str = "AI_CODE_ENABLE_TELEMETRY";
    pub const EXPERIMENTAL_AGENT_TEAMS: &str = "AI_CODE_EXPERIMENTAL_AGENT_TEAMS";
    pub const IDE_SKIP_AUTO_INSTALL: &str = "AI_CODE_IDE_SKIP_AUTO_INSTALL";
    pub const MAX_OUTPUT_TOKENS: &str = "AI_CODE_MAX_OUTPUT_TOKENS";
    pub const ATTRIBUTION_HEADER: &str = "AI_CODE_ATTRIBUTION_HEADER";
    pub const SANDBOX_ENABLED: &str = "AI_CODE_SANDBOX_ENABLED";
    pub const SANDBOX_DIR: &str = "AI_CODE_SANDBOX_DIR";
    pub const EXIT_AFTER_STOP_DELAY: &str = "AI_CODE_EXIT_AFTER_STOP_DELAY";
    pub const CLIENT_CERT: &str = "AI_CODE_CLIENT_CERT";
    pub const CLIENT_KEY: &str = "AI_CODE_CLIENT_KEY";
    pub const CLIENT_KEY_PASSPHRASE: &str = "AI_CODE_CLIENT_KEY_PASSPHRASE";

    /// Compact
    pub const BLOCKING_LIMIT_OVERRIDE: &str = "AI_CODE_BLOCKING_LIMIT_OVERRIDE";
}

/// AI_CODE_* model defaults
pub mod ai_code_model {
    pub const DEFAULT_HAIKU_MODEL: &str = "AI_DEFAULT_HAIKU_MODEL";
    pub const DEFAULT_HAIKU_MODEL_DESCRIPTION: &str = "AI_DEFAULT_HAIKU_MODEL_DESCRIPTION";
    pub const DEFAULT_HAIKU_MODEL_NAME: &str = "AI_DEFAULT_HAIKU_MODEL_NAME";
    pub const DEFAULT_HAIKU_MODEL_SUPPORTED_CAPABILITIES: &str = "AI_DEFAULT_HAIKU_MODEL_SUPPORTED_CAPABILITIES";
    pub const DEFAULT_OPUS_MODEL: &str = "AI_DEFAULT_OPUS_MODEL";
    pub const DEFAULT_OPUS_MODEL_DESCRIPTION: &str = "AI_DEFAULT_OPUS_MODEL_DESCRIPTION";
    pub const DEFAULT_OPUS_MODEL_NAME: &str = "AI_DEFAULT_OPUS_MODEL_NAME";
    pub const DEFAULT_OPUS_MODEL_SUPPORTED_CAPABILITIES: &str = "AI_DEFAULT_OPUS_MODEL_SUPPORTED_CAPABILITIES";
    pub const DEFAULT_SONNET_MODEL: &str = "AI_DEFAULT_SONNET_MODEL";
    pub const DEFAULT_SONNET_MODEL_DESCRIPTION: &str = "AI_DEFAULT_SONNET_MODEL_DESCRIPTION";
    pub const DEFAULT_SONNET_MODEL_NAME: &str = "AI_DEFAULT_SONNET_MODEL_NAME";
    pub const DEFAULT_SONNET_MODEL_SUPPORTED_CAPABILITIES: &str = "AI_DEFAULT_SONNET_MODEL_SUPPORTED_CAPABILITIES";
}

/// Standard system environment variables (OS-level, not AI_* prefixed)
pub mod system {
    /// Shell
    pub const SHELL: &str = "SHELL";
    pub const HOME: &str = "HOME";
    pub const USERPROFILE: &str = "USERPROFILE";

    /// Path
    pub const PATH: &str = "PATH";

    /// Editor
    pub const EDITOR: &str = "EDITOR";
    pub const VISUAL: &str = "VISUAL";

    /// Terminal
    pub const TERM: &str = "TERM";

    /// Localization
    pub const LANG: &str = "LANG";
    pub const LC_ALL: &str = "LC_ALL";

    /// XDG directories
    pub const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
    pub const XDG_DATA_HOME: &str = "XDG_DATA_HOME";
    pub const XDG_CACHE_HOME: &str = "XDG_CACHE_HOME";

    /// Windows paths
    pub const APPDATA: &str = "APPDATA";
    pub const LOCALAPPDATA: &str = "LOCALAPPDATA";

    /// Proxy (lowercase variants)
    pub const HTTP_PROXY: &str = "HTTP_PROXY";
    pub const HTTPS_PROXY: &str = "HTTPS_PROXY";
    pub const NO_PROXY: &str = "NO_PROXY";
    pub const HTTP_PROXY_LOWER: &str = "http_proxy";
    pub const HTTPS_PROXY_LOWER: &str = "https_proxy";
    pub const NO_PROXY_LOWER: &str = "no_proxy";

    /// CI/CD
    pub const CI: &str = "CI";
    pub const NODE_ENV: &str = "NODE_ENV";

    /// Debug
    pub const DEBUG: &str = "DEBUG";
    pub const DEBUG_SDK: &str = "DEBUG_SDK";

    /// Docker/Kubernetes
    pub const DOCKER_CONTAINER: &str = "DOCKER_CONTAINER";
    pub const KUBERNETES_SERVICE_HOST: &str = "KUBERNETES_SERVICE_HOST";

    /// WSL
    pub const WSL_DISTRO_NAME: &str = "WSL_DISTRO_NAME";

    /// Bun
    pub const BUN_VERSION: &str = "BUN_VERSION";
    pub const BUN_EMBEDDED: &str = "BUN_EMBEDDED";

    /// Compact
    pub const DISABLE_COMPACT: &str = "DISABLE_COMPACT";

    /// Autoupdater
    pub const DISABLE_AUTOUPDATER: &str = "DISABLE_AUTOUPDATER";

    /// GrowthBook
    pub const ENABLE_GROWTHBOOK_DEV: &str = "ENABLE_GROWTHBOOK_DEV";

    /// Process
    pub const PPID: &str = "PPID";

    /// VCR
    pub const VCR_RECORD: &str = "VCR_RECORD";

    /// Native installer
    pub const ENABLE_PID_BASED_VERSION_LOCKING: &str = "ENABLE_PID_BASED_VERSION_LOCKING";
}