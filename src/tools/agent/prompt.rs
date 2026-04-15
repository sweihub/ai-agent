// Source: ~/claudecode/openclaudecode/src/tools/AgentTool/prompt.ts
#![allow(dead_code)]
use std::sync::Arc;

use super::constants::{AGENT_TOOL_NAME, FILE_READ_TOOL_NAME, FILE_WRITE_TOOL_NAME,
                       GLOB_TOOL_NAME, LEGACY_AGENT_TOOL_NAME, SEND_MESSAGE_TOOL_NAME};
use super::load_agents_dir::AgentDefinition;

/// Tool names referenced in the prompt
const EXIT_PLAN_MODE_TOOL_NAME: &str = "ExitPlanMode";
const WEB_FETCH_TOOL_NAME: &str = "WebFetch";

/// Format one agent line for the agent_listing_delta attachment message.
pub fn format_agent_line(agent: &AgentDefinition) -> String {
    let tools_description = get_tools_description(agent);
    format!(
        "- {}: {} (Tools: {})",
        agent.agent_type, agent.when_to_use, tools_description
    )
}

/// Get a description of the tools available to an agent.
fn get_tools_description(agent: &AgentDefinition) -> String {
    let has_allowlist = !agent.tools.is_empty() && agent.tools != vec!["*"];
    let has_denylist = !agent.disallowed_tools.is_empty();

    if has_allowlist && has_denylist {
        let deny_set: std::collections::HashSet<&str> = agent
            .disallowed_tools
            .iter()
            .map(|s| s.as_str())
            .collect();
        let effective: Vec<&String> = agent
            .tools
            .iter()
            .filter(|t| !deny_set.contains(t.as_str()))
            .collect();
        if effective.is_empty() {
            return "None".to_string();
        }
        effective.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
    } else if has_allowlist {
        agent.tools.join(", ")
    } else if has_denylist {
        let tools: Vec<&str> = agent.disallowed_tools.iter().map(|s| s.as_str()).collect();
        format!("All tools except {}", tools.join(", "))
    } else {
        "All tools".to_string()
    }
}

/// Whether the agent list should be injected as an attachment message.
pub fn should_inject_agent_list_in_messages() -> bool {
    std::env::var("AI_CODE_AGENT_LIST_IN_MESSAGES")
        .map(|v| v == "1" || v == "true" || v == "yes")
        .unwrap_or(false)
}

/// Whether fork subagent feature is enabled.
pub fn is_fork_subagent_enabled() -> bool {
    std::env::var("AI_CODE_FORK_SUBAGENT")
        .map(|v| v == "1" || v == "true" || v == "yes")
        .unwrap_or(false)
}

/// Whether embedded search tools are available (bfs/ugrep instead of Glob/Grep).
pub fn has_embedded_search_tools() -> bool {
    std::env::var("AI_CODE_EMBEDDED_SEARCH_TOOLS")
        .map(|v| v == "1" || v == "true" || v == "yes")
        .unwrap_or(false)
}

/// Get the subscription type.
fn get_subscription_type() -> &'static str {
    std::env::var("AI_CODE_SUBSCRIPTION_TYPE")
        .ok()
        .map(|s| {
            let s = s.to_lowercase();
            if s == "pro" {
                "pro"
            } else {
                "free"
            }
        })
        .unwrap_or("free")
}

/// Check if background tasks are disabled.
fn is_background_tasks_disabled() -> bool {
    std::env::var("AI_CODE_DISABLE_BACKGROUND_TASKS")
        .map(|v| v == "1" || v == "true" || v == "yes")
        .unwrap_or(false)
}

/// Check if running as a teammate.
fn is_in_process_teammate() -> bool {
    std::env::var("AI_CODE_IN_PROCESS_TEAMMATE")
        .map(|v| v == "1" || v == "true" || v == "yes")
        .unwrap_or(false)
}

/// Check if running as a teammate (general).
fn is_teammate() -> bool {
    std::env::var("AI_CODE_TEAMMATE")
        .map(|v| v == "1" || v == "true" || v == "yes")
        .unwrap_or(false)
}

/// Generate the Agent tool prompt.
pub async fn get_prompt(
    agent_definitions: &[AgentDefinition],
    is_coordinator: bool,
    allowed_agent_types: Option<&[String]>,
) -> String {
    // Filter agents by allowed types when Agent(x,y) restricts which agents can be spawned
    let effective_agents: Vec<&AgentDefinition> = if let Some(types) = allowed_agent_types {
        agent_definitions
            .iter()
            .filter(|a| types.iter().any(|t| t == &a.agent_type))
            .collect()
    } else {
        agent_definitions.iter().collect()
    };

    let fork_enabled = is_fork_subagent_enabled();
    let list_via_attachment = should_inject_agent_list_in_messages();
    let embedded = has_embedded_search_tools();

    // Agent list section
    let agent_list_section = if list_via_attachment {
        "Available agent types are listed in <system-reminder> messages in the conversation."
            .to_string()
    } else {
        format!(
            "Available agent types and the tools they have access to:\n{}",
            effective_agents
                .iter()
                .map(|a| format_agent_line(a))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    // Fork-related sections
    let when_to_fork_section = if fork_enabled {
        r#"

## When to fork

Fork yourself (omit `subagent_type`) when the intermediate tool output isn't worth keeping in your context. The criterion is qualitative — "will I need this output again" — not task size.
- **Research**: fork open-ended questions. If research can be broken into independent questions, launch parallel forks in one message. A fork beats a fresh subagent for this — it inherits context and shares your cache.
- **Implementation**: prefer to fork implementation work that requires more than a couple of edits. Do research before jumping to implementation.

Forks are cheap because they share your prompt cache. Don't set `model` on a fork — a different model can't reuse the parent's cache. Pass a short `name` (one or two words, lowercase) so the user can see the fork in the teams panel and steer it mid-run.

**Don't peek.** The tool result includes an `output_file` path — do not Read or tail it unless the user explicitly asks for a progress check. You get a completion notification; trust it. Reading the transcript mid-flight pulls the fork's tool noise into your context, which defeats the point of forking.

**Don't race.** After launching, you know nothing about what the fork found. Never fabricate or predict fork results in any format — not as prose, summary, or structured output. The notification arrives as a user-role message in a later turn; it is never something you write yourself. If the user asks a follow-up before the notification lands, tell them the fork is still running — give status, not a guess.

**Writing a fork prompt.** Since the fork inherits your context, the prompt is a *directive* — what to do, not what the situation is. Be specific about scope: what's in, what's out, what another agent is handling. Don't re-explain background."#
            .to_string()
    } else {
        String::new()
    };

    let writing_the_prompt_section = format!(
        r#"

## Writing the prompt

{context_note}Brief the agent like a smart colleague who just walked into the room — it hasn't seen this conversation, doesn't know what you've tried, doesn't understand why this task matters.
- Explain what you're trying to accomplish and why.
- Describe what you've already learned or ruled out.
- Give enough context about the surrounding problem that the agent can make judgment calls rather than just following a narrow instruction.
- If you need a short response, say so ("report in under 200 words").
- Lookups: hand over the exact command. Investigations: hand over the question — prescribed steps become dead weight when the premise is wrong.

{style} command-style prompts produce shallow, generic work.

**Never delegate understanding.** Don't write "based on your findings, fix the bug" or "based on the research, implement it." Those phrases push synthesis onto the agent instead of doing it yourself. Write prompts that prove you understood: include file paths, line numbers, what specifically to change."#,
        context_note = if fork_enabled {
            "When spawning a fresh agent (with a `subagent_type`), it starts with zero context. "
        } else {
            ""
        },
        style = if fork_enabled { "For fresh agents, terse" } else { "Terse" },
    );

    // Shared core prompt
    let shared = format!(
        r#"Launch a new agent to handle complex, multi-step tasks autonomously.

The {agent_tool} tool launches specialized agents (subprocesses) that autonomously handle complex tasks. Each agent type has specific capabilities and tools available to it.

{agent_list}

{when_to_use}"#,
        agent_tool = AGENT_TOOL_NAME,
        agent_list = agent_list_section,
        when_to_use = if fork_enabled {
            format!(
                "When using the {} tool, specify a subagent_type to use a specialized agent, or omit it to fork yourself — a fork inherits your full conversation context.",
                AGENT_TOOL_NAME
            )
        } else {
            format!(
                "When using the {} tool, specify a subagent_type parameter to select which agent type to use. If omitted, the general-purpose agent is used.",
                AGENT_TOOL_NAME
            )
        }
    );

    // Coordinator mode gets the slim prompt
    if is_coordinator {
        return shared;
    }

    // When NOT to use section
    let file_search_hint = if embedded {
        "`find` via the Bash tool"
    } else {
        GLOB_TOOL_NAME
    };
    let content_search_hint = if embedded {
        "`grep` via the Bash tool"
    } else {
        GLOB_TOOL_NAME
    };

    let when_not_to_use = if fork_enabled {
        String::new()
    } else {
        format!(
            r#"
When NOT to use the {agent_tool} tool:
- If you want to read a specific file path, use the {file_read} tool or {file_search} instead of the {agent_tool} tool, to find the match more quickly
- If you are searching for a specific class definition like "class Foo", use {content_search} instead, to find the match more quickly
- If you are searching for code within a specific file or set of 2-3 files, use the {file_read} tool instead of the {agent_tool} tool, to find the match more quickly
- Other tasks that are not related to the agent descriptions above
"#,
            agent_tool = AGENT_TOOL_NAME,
            file_read = FILE_READ_TOOL_NAME,
            file_search = file_search_hint,
            content_search = content_search_hint,
        )
    };

    // Usage notes
    let concurrency_note = if !list_via_attachment && get_subscription_type() != "pro" {
        "\n- Launch multiple agents concurrently whenever possible, to maximize performance; to do that, use a single message with multiple tool uses"
    } else {
        ""
    };

    let background_note = if !is_background_tasks_disabled()
        && !is_in_process_teammate()
        && !fork_enabled
    {
        r#"
- You can optionally run agents in the background using the run_in_background parameter. When an agent runs in the background, you will be automatically notified when it completes — do NOT sleep, poll, or proactively check on its progress. Continue with other work or respond to the user instead.
- **Foreground vs background**: Use foreground (default) when you need the agent's results before you can proceed — e.g., research agents whose findings inform your next steps. Use background when you have genuinely independent work to do in parallel."#
    } else {
        ""
    };

    let resume_note = if fork_enabled {
        "Each fresh Agent invocation with a subagent_type starts without context — provide a complete task description."
    } else {
        "Each Agent invocation starts fresh — provide a complete task description."
    };

    let clearly_tell_note = if fork_enabled { "" } else { ", since it is not aware of the user's intent" };

    let isolation_note = if is_teammate() {
        "\n- The name, team_name, and mode parameters are not available in this context — teammates cannot spawn other teammates. Omit them to spawn a subagent."
    } else {
        ""
    };

    // Examples
    let examples = if fork_enabled {
        get_fork_examples()
    } else {
        get_current_examples()
    };

    format!(
        r#"{shared}
{when_not_to_use}
Usage notes:
- Always include a short description (3-5 words) summarizing what the agent will do{concurrency_note}
- When the agent is done, it will return a single message back to you. The result returned by the agent is not visible to the user. To show the user the result, you should send a text message back to the user with a concise summary of the result.{background_note}
- To continue a previously spawned agent, use {send_message} with the agent's ID or name as the `to` field. The agent resumes with its full context preserved. {resume_note}
- The agent's outputs should generally be trusted
- Clearly tell the agent whether you expect it to write code or just to do research (search, file reads, web fetches, etc.){clearly_tell_note}
- If the agent description mentions that it should be used proactively, then you should try your best to use it without the user having to ask for it first. Use your judgement.
- If the user specifies that they want you to run agents "in parallel", you MUST send a single message with multiple {agent_tool} tool use content blocks. For example, if you need to launch both a build-validator agent and a test-runner agent in parallel, send a single message with both tool calls.{isolation_note}
{writing_the_prompt_section}
{when_to_fork_section}
{examples}"#,
        send_message = SEND_MESSAGE_TOOL_NAME,
        agent_tool = AGENT_TOOL_NAME,
    )
}

/// Get fork-aware examples.
fn get_fork_examples() -> String {
    format!(
        r#"Example usage:

<example>
user: "What's left on this branch before we can ship?"
assistant: <thinking>Forking this — it's a survey question. I want the punch list, not the git output in my context.</thinking>
{agent_tool}({{
  name: "ship-audit",
  description: "Branch ship-readiness audit",
  prompt: "Audit what's left before this branch can ship. Check: uncommitted changes, commits ahead of main, whether tests exist, whether the GrowthBook gate is wired up, whether CI-relevant files changed. Report a punch list — done vs. missing. Under 200 words."
}})
assistant: Ship-readiness audit running.
<commentary>
Turn ends here. The coordinator knows nothing about the findings yet. What follows is a SEPARATE turn — the notification arrives from outside, as a user-role message. It is not something the coordinator writes.
</commentary>
[later turn — notification arrives as user message]
assistant: Audit's back. Three blockers: no tests for the new prompt path, GrowthBook gate wired but not in build_flags.yaml, and one uncommitted file.
</example>

<example>
user: "Can you get a second opinion on whether this migration is safe?"
assistant: <thinking>I'll ask the code-reviewer agent — it won't see my analysis, so it can give an independent read.</thinking>
<commentary>
A subagent_type is specified, so the agent starts fresh. It needs full context in the prompt. The briefing explains what to assess and why.
</commentary>
{agent_tool}({{
  name: "migration-review",
  description: "Independent migration review",
  subagent_type: "code-reviewer",
  prompt: "Review migration 0042_user_schema.sql for safety. Context: we're adding a NOT NULL column to a 50M-row table. Existing rows get a backfill default. I want a second opinion on whether the backfill approach is safe under concurrent writes — I've checked locking behavior but want independent verification. Report: is this safe, and if not, what specifically breaks?"
}})
</example>"#,
        agent_tool = AGENT_TOOL_NAME
    )
}

/// Get current (non-fork) examples.
fn get_current_examples() -> String {
    format!(
        r#"Example usage:

<example_agent_descriptions>
"test-runner": use this agent after you are done writing code to run tests
"greeting-responder": use this agent to respond to user greetings with a friendly joke
</example_agent_descriptions>

<example>
user: "Please write a function that checks if a number is prime"
assistant: I'm going to use the {file_write} tool to write the following code:
<code>
function isPrime(n) {{
  if (n <= 1) return false
  for (let i = 2; i * i <= n; i++) {{
    if (n % i === 0) return false
  }}
  return true
}}
</code>
<commentary>
Since a significant piece of code was written and the task was completed, now use the test-runner agent to run the tests
</commentary>
assistant: Uses the {agent_tool} tool to launch the test-runner agent
</example>

<example>
user: "Hello"
<commentary>
Since the user is greeting, use the greeting-responder agent to respond with a friendly joke
</commentary>
assistant: "I'm going to use the {agent_tool} tool to launch the greeting-responder agent"
</example>"#,
        file_write = FILE_WRITE_TOOL_NAME,
        agent_tool = AGENT_TOOL_NAME,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_agent_line() {
        let agent = AgentDefinition {
            agent_type: "test".to_string(),
            when_to_use: "A test agent".to_string(),
            tools: vec!["Bash".to_string(), "Read".to_string()],
            disallowed_tools: vec![],
            source: "built-in".to_string(),
            base_dir: "built-in".to_string(),
            get_system_prompt: Arc::new(|| String::new()),
            model: None,
            max_turns: None,
            permission_mode: None,
            effort: None,
            color: None,
            mcp_servers: vec![],
            hooks: None,
            skills: vec![],
            background: false,
            initial_prompt: None,
            memory: None,
            isolation: None,
            required_mcp_servers: vec![],
            omit_claude_md: false,
            critical_system_reminder_experimental: None,
        };
        let line = format_agent_line(&agent);
        assert!(line.contains("test"));
        assert!(line.contains("A test agent"));
        assert!(line.contains("Bash, Read"));
    }

    #[test]
    fn test_get_tools_description_wildcard() {
        let agent = AgentDefinition {
            agent_type: "test".to_string(),
            when_to_use: "".to_string(),
            tools: vec!["*".to_string()],
            disallowed_tools: vec![],
            source: "built-in".to_string(),
            base_dir: "built-in".to_string(),
            get_system_prompt: Arc::new(|| String::new()),
            model: None,
            max_turns: None,
            permission_mode: None,
            effort: None,
            color: None,
            mcp_servers: vec![],
            hooks: None,
            skills: vec![],
            background: false,
            initial_prompt: None,
            memory: None,
            isolation: None,
            required_mcp_servers: vec![],
            omit_claude_md: false,
            critical_system_reminder_experimental: None,
        };
        assert_eq!(get_tools_description(&agent), "All tools");
    }

    #[test]
    fn test_get_tools_description_with_denylist() {
        let agent = AgentDefinition {
            agent_type: "test".to_string(),
            when_to_use: "".to_string(),
            tools: vec!["*".to_string()],
            disallowed_tools: vec!["Write".to_string(), "Edit".to_string()],
            source: "built-in".to_string(),
            base_dir: "built-in".to_string(),
            get_system_prompt: Arc::new(|| String::new()),
            model: None,
            max_turns: None,
            permission_mode: None,
            effort: None,
            color: None,
            mcp_servers: vec![],
            hooks: None,
            skills: vec![],
            background: false,
            initial_prompt: None,
            memory: None,
            isolation: None,
            required_mcp_servers: vec![],
            omit_claude_md: false,
            critical_system_reminder_experimental: None,
        };
        let desc = get_tools_description(&agent);
        assert!(desc.contains("All tools except"));
    }
}
