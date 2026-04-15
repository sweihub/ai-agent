// Source: /data/home/swei/claudecode/openclaudecode/src/commands/remote-setup/api.ts
#![allow(dead_code)]

use crate::constants::env::ai_code;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Extended tool type with strict mode and defer_loading support
#[derive(Debug, Clone)]
pub struct BetaToolWithExtras {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub strict: Option<bool>,
    pub defer_loading: Option<bool>,
    pub cache_control: Option<CacheControl>,
    pub eager_input_streaming: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct CacheControl {
    pub r#type: String,
    pub scope: Option<String>,
    pub ttl: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CacheScope {
    Global,
    Org,
}

#[derive(Debug, Clone)]
pub struct SystemPromptBlock {
    pub text: String,
    pub cache_scope: Option<CacheScope>,
}

// Fields to filter from tool schemas when swarms are not enabled
static SWARM_FIELDS_BY_TOOL: Lazy<HashMap<&'static str, Vec<&'static str>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("ExitPlanModeV2Tool", vec!["launchSwarm", "teammateCount"]);
    m.insert("AgentTool", vec!["name", "team_name", "mode"]);
    m
});

static LOGGED_STRIP: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

/// Filter swarm-related fields from a tool's input schema.
/// Called at runtime when is_agent_swarms_enabled() returns false.
pub fn filter_swarm_fields_from_schema(
    tool_name: &str,
    mut schema: serde_json::Value,
) -> serde_json::Value {
    let fields_to_remove = SWARM_FIELDS_BY_TOOL.get(tool_name);
    if fields_to_remove.is_none() || fields_to_remove.unwrap().is_empty() {
        return schema;
    }

    // Clone the schema to avoid mutating the original
    if let Some(obj) = schema.as_object_mut() {
        if let Some(props) = obj.get_mut("properties") {
            if let Some(props_obj) = props.as_object_mut() {
                for field in fields_to_remove.unwrap() {
                    props_obj.remove(*field);
                }
            }
        }
    }

    schema
}

/// Convert tool to API schema
pub async fn tool_to_api_schema(
    tool: &dyn Tool,
    options: ToolToApiSchemaOptions,
) -> Result<BetaToolWithExtras, String> {
    // Session-stable base schema: name, description, input_schema, strict,
    // eager_input_streaming. These are computed once per session and cached.
    let cache_key = if let Some(ref json_schema) = tool.get_input_json_schema() {
        format!("{}:{}", tool.get_name(), serde_json::to_string(json_schema).unwrap_or_default())
    } else {
        tool.get_name().to_string()
    };

    // Get or create cached base schema
    let base = get_tool_schema_cache().get(&cache_key).cloned();

    let base = match base {
        Some(b) => b,
        None => {
            let strict_tools_enabled = check_statsig_feature_gate("tengu_tool_pear");
            
            // Use tool's JSON schema directly if provided, otherwise convert Zod schema
            let input_schema = if let Some(json_schema) = tool.get_input_json_schema() {
                json_schema.clone()
            } else {
                // Would convert from Zod schema
                serde_json::json!({})
            };

            // Filter out swarm-related fields when swarms are not enabled
            let input_schema = if !is_agent_swarms_enabled() {
                filter_swarm_fields_from_schema(tool.get_name(), input_schema)
            } else {
                input_schema
            };

            let base_schema = BaseToolSchema {
                name: tool.get_name().to_string(),
                description: tool.get_prompt().await.unwrap_or_default(),
                input_schema,
                strict: None,
                eager_input_streaming: None,
            };

            // Only add strict if:
            // 1. Feature flag is enabled
            // 2. Tool has strict: true
            // 3. Model is provided and supports it
            if strict_tools_enabled && tool.is_strict() {
                if let Some(ref model) = options.model {
                    if model_supports_structured_outputs(model) {
                        // Would set strict
                    }
                }
            }

            // Enable fine-grained tool streaming
            // Matches TypeScript: sets eager_input_streaming when flag is enabled
            if get_api_provider() == "firstParty" && is_first_party_anthropic_base_url() {
                if is_env_truthy("AI_CODE_ENABLE_FINE_GRAINED_TOOL_STREAMING") {
                    base_schema.eager_input_streaming = Some(true);
                }
            }

            get_tool_schema_cache().insert(cache_key, base_schema.clone());
            base_schema
        }
    };

    // Per-request overlay: defer_loading and cache_control vary by call
    let mut schema = BetaToolWithExtras {
        name: base.name,
        description: base.description,
        input_schema: base.input_schema,
        strict: base.strict,
        defer_loading: None,
        cache_control: None,
        eager_input_streaming: base.eager_input_streaming,
    };

    // Add defer_loading if requested
    if options.defer_loading {
        schema.defer_loading = Some(true);
    }

    if let Some(ref cache_control) = options.cache_control {
        schema.cache_control = Some(CacheControl {
            r#type: cache_control.get("type").cloned().unwrap_or_else(|| "ephemeral".to_string()),
            scope: cache_control.get("scope").cloned(),
            ttl: cache_control.get("ttl").cloned(),
        });
    }

    // AI_CODE_DISABLE_EXPERIMENTAL_BETAS is the kill switch for beta API shapes
    if is_env_truthy("AI_CODE_DISABLE_EXPERIMENTAL_BETAS") {
        let allowed: HashSet<&str> = ["name", "description", "input_schema", "cache_control"]
            .iter()
            .cloned()
            .collect();
        
        let stripped: Vec<String> = vec![]; // Would determine stripped fields
        
        if !stripped.is_empty() {
            log_strip_once(stripped);
            return BetaToolWithExtras {
                name: schema.name,
                description: schema.description,
                input_schema: schema.input_schema,
                strict: None,
                defer_loading: None,
                cache_control: schema.cache_control,
                eager_input_streaming: None,
            };
        }
    }

    Ok(schema)
}

fn log_strip_once(stripped: Vec<String>) {
    if let Ok(mut logged) = LOGGED_STRIP.lock() {
        if *logged {
            return;
        }
        *logged = true;
        log_for_debugging(&format!(
            "[betas] Stripped from tool schemas: [{}] (AI_CODE_DISABLE_EXPERIMENTAL_BETAS=1)",
            stripped.join(", ")
        ));
    }
}

/// Split system prompt blocks by content type for API matching and cache control.
pub fn split_sys_prompt_prefix(
    system_prompt: Vec<String>,
    options: Option<SplitSysPromptOptions>,
) -> Vec<SystemPromptBlock> {
    let use_global_cache_feature = should_use_global_cache_scope();
    
    if use_global_cache_feature && options.as_ref().map(|o| o.skip_global_cache_for_system_prompt).unwrap_or(false) {
        // Filter out boundary marker, return blocks without global scope
        let mut attribution_header: Option<String> = None;
        let mut system_prompt_prefix: Option<String> = None;
        let mut rest: Vec<String> = vec![];

        for prompt in &system_prompt {
            if prompt.is_empty() {
                continue;
            }
            if prompt == "x-anthropic-billing-header" {
                continue; // Skip boundary
            }
            if prompt.starts_with("x-anthropic-billing-header") {
                attribution_header = Some(prompt.clone());
            } else if CLI_SYSPROMPT_PREFIXES.contains(prompt) {
                system_prompt_prefix = Some(prompt.clone());
            } else {
                rest.push(prompt.clone());
            }
        }

        let mut result = vec![];
        if let Some(header) = attribution_header {
            result.push(SystemPromptBlock { text: header, cache_scope: None });
        }
        if let Some(prefix) = system_prompt_prefix {
            result.push(SystemPromptBlock { text: prefix, cache_scope: Some(CacheScope::Org) });
        }
        let rest_joined = rest.join("\n\n");
        if !rest_joined.is_empty() {
            result.push(SystemPromptBlock { text: rest_joined, cache_scope: Some(CacheScope::Org) });
        }
        return result;
    }

    if use_global_cache_feature {
        // Look for boundary marker
        if let Some(boundary_index) = system_prompt.iter().position(|s| s == "x-anthropic-billing-header") {
            let mut attribution_header: Option<String> = None;
            let mut system_prompt_prefix: Option<String> = None;
            let mut static_blocks: Vec<String> = vec![];
            let mut dynamic_blocks: Vec<String> = vec![];

            for (i, block) in system_prompt.iter().enumerate() {
                if block.is_empty() || block == "x-anthropic-billing-header" {
                    continue;
                }

                if block.starts_with("x-anthropic-billing-header") {
                    attribution_header = Some(block.clone());
                } else if CLI_SYSPROMPT_PREFIXES.contains(block) {
                    system_prompt_prefix = Some(block.clone());
                } else if i < boundary_index {
                    static_blocks.push(block.clone());
                } else {
                    dynamic_blocks.push(block.clone());
                }
            }

            let mut result = vec![];
            if let Some(header) = attribution_header {
                result.push(SystemPromptBlock { text: header, cache_scope: None });
            }
            if let Some(prefix) = system_prompt_prefix {
                result.push(SystemPromptBlock { text: prefix, cache_scope: None });
            }
            let static_joined = static_blocks.join("\n\n");
            if !static_joined.is_empty() {
                result.push(SystemPromptBlock { text: static_joined, cache_scope: Some(CacheScope::Global) });
            }
            let dynamic_joined = dynamic_blocks.join("\n\n");
            if !dynamic_joined.is_empty() {
                result.push(SystemPromptBlock { text: dynamic_joined, cache_scope: None });
            }

            return result;
        }
    }

    // Default mode
    let mut attribution_header: Option<String> = None;
    let mut system_prompt_prefix: Option<String> = None;
    let mut rest: Vec<String> = vec![];

    for block in &system_prompt {
        if block.is_empty() {
            continue;
        }

        if block.starts_with("x-anthropic-billing-header") {
            attribution_header = Some(block.clone());
        } else if CLI_SYSPROMPT_PREFIXES.contains(block) {
            system_prompt_prefix = Some(block.clone());
        } else {
            rest.push(block.clone());
        }
    }

    let mut result = vec![];
    if let Some(header) = attribution_header {
        result.push(SystemPromptBlock { text: header, cache_scope: None });
    }
    if let Some(prefix) = system_prompt_prefix {
        result.push(SystemPromptBlock { text: prefix, cache_scope: Some(CacheScope::Org) });
    }
    let rest_joined = rest.join("\n\n");
    if !rest_joined.is_empty() {
        result.push(SystemPromptBlock { text: rest_joined, cache_scope: Some(CacheScope::Org) });
    }
    result
}

pub fn append_system_context(
    system_prompt: Vec<String>,
    context: HashMap<String, String>,
) -> Vec<String> {
    let context_str = context
        .iter()
        .map(|(key, value)| format!("{}: {}", key, value))
        .collect::<Vec<_>>()
        .join("\n");
    
    let mut result = system_prompt;
    if !context_str.is_empty() {
        result.push(context_str);
    }
    result.into_iter().filter(|s| !s.is_empty()).collect()
}

// ============ Helper Functions ============

static TOOL_SCHEMA_CACHE: Lazy<Mutex<HashMap<String, BaseToolSchema>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn get_tool_schema_cache() -> std::sync::MutexGuard<'static, HashMap<String, BaseToolSchema>> {
    TOOL_SCHEMA_CACHE.lock().unwrap()
}

#[derive(Debug, Clone)]
struct BaseToolSchema {
    name: String,
    description: String,
    input_schema: serde_json::Value,
    strict: Option<bool>,
    eager_input_streaming: Option<bool>,
}

trait Tool {
    fn get_name(&self) -> &str;
    fn get_prompt(&self) -> impl std::future::Future<Output = Result<String, String>> + '_;
    fn get_input_json_schema(&self) -> Option<&serde_json::Value>;
    fn is_strict(&self) -> bool;
}

struct ToolToApiSchemaOptions {
    get_tool_permission_context: Option<Box<dyn Fn() -> Box<dyn ToolPermissionContext>>>,
    tools: Vec<Box<dyn Tool>>,
    agents: Vec<AgentDefinition>,
    allowed_agent_types: Option<Vec<String>>,
    model: Option<String>,
    defer_loading: Option<bool>,
    cache_control: Option<HashMap<String, String>>,
}

trait ToolPermissionContext: Send + Sync {
    fn get_mode(&self) -> &str;
}

struct AgentDefinition {
    agent_type: String,
    // ... other fields
}

fn check_statsig_feature_gate(_gate: &str) -> bool {
    // Would check with GrowthBook
    false
}

fn is_agent_swarms_enabled() -> bool {
    std::env::var(ai_code::ENABLE_AGENT_SWARMS).is_ok()
}

fn model_supports_structured_outputs(_model: &str) -> bool {
    // Would check model capabilities
    true
}

fn get_api_provider() -> String {
    if is_env_truthy("AI_CODE_USE_BEDROCK") {
        "bedrock".to_string()
    } else if is_env_truthy("AI_CODE_USE_VERTEX") {
        "vertex".to_string()
    } else if is_env_truthy("AI_CODE_USE_FOUNDRY") {
        "foundry".to_string()
    } else {
        "firstParty".to_string()
    }
}

fn is_first_party_anthropic_base_url() -> bool {
    // Would check ANTHROPIC_BASE_URL
    true
}

fn should_use_global_cache_scope() -> bool {
    // Would check feature flag
    false
}

fn is_env_truthy(var: &str) -> bool {
    std::env::var(var).map(|v| v == "1" || v == "true").unwrap_or(false)
}

static CLI_SYSPROMPT_PREFIXES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut s = HashSet::new();
    // Would populate with actual prefixes
    s
});

struct SplitSysPromptOptions {
    skip_global_cache_for_system_prompt: bool,
}

fn log_for_debugging(message: &str) {
    eprintln!("[DEBUG] {}", message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_swarm_fields_from_schema() {
        let schema = serde_json::json!({
            "properties": {
                "command": { "type": "string" },
                "launchSwarm": { "type": "boolean" },
                "teammateCount": { "type": "number" }
            }
        });

        let filtered = filter_swarm_fields_from_schema("ExitPlanModeV2Tool", schema);
        let props = filtered.get("properties").unwrap().as_object().unwrap();
        assert!(props.contains_key("command"));
        assert!(!props.contains_key("launchSwarm"));
        assert!(!props.contains_key("teammateCount"));
    }

    #[test]
    fn test_split_sys_prompt_prefix() {
        let prompts = vec![
            "x-anthropic-billing-header: test".to_string(),
            "System prefix".to_string(),
            "Main content".to_string(),
        ];

        let result = split_sys_prompt_prefix(prompts, None);
        assert!(!result.is_empty());
    }
}
