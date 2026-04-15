/**
 * Example 18: Plugin System
 *
 * Demonstrates loading and using plugins with commands, skills, and MCP servers.
 *
 * Run: cargo run --example 18_plugin
 *
 * This example shows:
 * 1. Loading plugins from directories
 * 2. Using plugin commands
 * 3. Using plugin skills
 * 4. Managing MCP servers
 */
use ai_agent::{Agent, load_plugin, load_plugins_from_dir, CommandRegistry, PluginMcpServerManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 18: Plugin System ---\n");

    // Load a plugin from examples/plugins directory
    let plugin_path = std::path::Path::new("examples/plugins/hello-plugin");
    match load_plugin(plugin_path).await {
        Ok(plugin) => {
            println!("Loaded plugin: {} v{}",
                plugin.manifest.name,
                plugin.manifest.version.unwrap_or_else(|| "unknown".to_string()));
            println!("  Commands: {:?}", plugin.manifest.commands);
            println!("  Skills: {:?}", plugin.manifest.skills);
        }
        Err(e) => {
            println!("No plugin found at {}, skipping...", plugin_path.display());
            println!("  Error: {:?}", e);
        }
    }

    // Load all plugins from a directory
    println!("\n--- Loading plugins from directory ---");
    let plugins_dir = std::path::Path::new("examples/plugins");
    let plugins = load_plugins_from_dir(plugins_dir).await;
    println!("Loaded {} plugin(s)", plugins.len());
    for plugin in &plugins {
        let cmd_count = plugin.manifest.commands
            .as_ref()
            .and_then(|c| c.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        let skill_count = plugin.manifest.skills
            .as_ref()
            .and_then(|s| s.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        println!("  - {}: {} commands, {} skills",
            plugin.manifest.name,
            cmd_count,
            skill_count);
    }

    // Demonstrate Command Registry
    println!("\n--- Command Registry ---");
    let registry = CommandRegistry::new();
    println!("Created command registry");

    // List all registered commands
    let all_commands = registry.all_commands();
    println!("Total commands registered: {}", all_commands.len());

    // Demonstrate MCP Server Manager
    println!("\n--- MCP Server Manager ---");
    let mcp_manager = PluginMcpServerManager::new();
    println!("Created MCP server manager");

    // List available MCP servers
    let servers = mcp_manager.list_servers().await;
    println!("Total MCP servers: {}", servers.len());

    // Create an agent with plugin support
    println!("\n--- Creating Agent ---");
    let model = std::env::var("AI_MODEL").unwrap_or_else(|_| "MiniMaxAI/MiniMax-M2.5".to_string());
    let mut agent = Agent::new(&model, 10);

    // Use the agent to demonstrate plugin-related prompts
    println!("Using model: {}\n", model);

    // Note: To use plugin commands with the agent, you would typically:
    // 1. Load the plugin
    // 2. Register its commands with CommandRegistry
    // 3. The agent can then invoke /plugin:command

    // For now, just verify the agent works
    let result = agent.prompt("Say 'hello from plugin example' in one sentence.").await?;

    println!("Agent response: {}", result.text);
    println!("\n--- done ---");

    Ok(())
}