/**
 * Example 13: Agent Tool - Subagent Spawning
 *
 * Demonstrates using the Agent tool to spawn subagents for complex
 * multi-step tasks. When the Agent tool is called, it spawns a new
 * agent that runs independently to complete the sub-task.
 *
 * Run: cargo run --example 13_agent_tool
 *
 * Environment variables from .env:
 * - AI_BASE_URL: LLM server URL
 * - AI_AUTH_TOKEN: API authentication token
 * - AI_MODEL: Model name (defaults to claude-sonnet-4-6)
 */
use ai_agent::{get_all_tools, Agent, AgentOptions, EnvConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 13: Agent Tool ---\n");

    // Load config from .env
    let config = EnvConfig::load();
    let model = config.model.unwrap_or_else(|| "claude-sonnet-4-6".to_string());

    println!("Using model: {}\n", model);

    // Create agent with all tools including Agent tool
    let tools = get_all_tools();
    println!("Available tools: {:?}\n", tools.iter().map(|t| &t.name).collect::<Vec<_>>());

    let mut agent = Agent::create(AgentOptions {
        model: Some(model.to_string()),
        max_turns: Some(10),
        tools,
        ..Default::default()
    });

    // Set system prompt to encourage tool use
    agent.set_system_prompt("You have access to tools. When asked to spawn a subagent, \
use the 'Agent' tool with the appropriate description and prompt.");

    // The main agent will use the Agent tool to spawn a subagent
    let result = agent.query(
        "Use the 'Agent' tool to spawn a subagent. Description: 'count-numbers'. \
        Prompt: 'Count from 1 to 3, one number per line.'"
    ).await?;

    println!("{}", result.text);
    println!("\n=== done ===");

    Ok(())
}