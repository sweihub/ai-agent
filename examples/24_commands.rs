/**
 * Example 24: Custom Commands
 *
 * Demonstrates command registration and execution:
 * - Register custom slash commands
 * - Define command arguments
 * - Execute commands with the agent
 *
 * Run: cargo run --example 24_commands
 *
 * Environment variables from .env:
 * - AI_BASE_URL: LLM server URL
 * - AI_AUTH_TOKEN: API authentication token
 * - AI_MODEL: Model name (defaults to claude-sonnet-4-6)
 */
use ai_agent::{
    Agent, AgentOptions, CommandResult, CommandSource,
    CommandAvailability, CommandResultDisplay,
    PluginCommand, CommandRegistry, EnvConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 24: Custom Commands ---\n");

    let config = EnvConfig::load();
    let model = config.model.clone().unwrap_or_else(|| "claude-sonnet-4-6".to_string());

    println!("Using model: {}\n", model);

    // Create agent
    let mut agent = Agent::create(AgentOptions {
        model: Some(model.clone()),
        max_turns: Some(5),
        ..Default::default()
    });

    // Register a custom command
    let cmd = create_greet_command();
    println!("Registered command: /{}", cmd.name);
    println!("Description: {}", cmd.description);
    println!("Allowed tools: {:?}\n", cmd.allowed_tools);

    // Demonstrate command execution via agent
    println!("=== Asking agent to greet ===");
    let result = agent.prompt(
        "Say hello using the /greet command with argument 'World'"
    ).await?;
    println!("{}", result.text);
    println!();

    // Demonstrate command types
    println!("=== Command Types ===");
    println!("CommandAvailability: {:?} or {:?}", CommandAvailability::ClaudeAi, CommandAvailability::Console);
    println!("CommandResultDisplay: {:?}, {:?}, {:?}", CommandResultDisplay::Skip, CommandResultDisplay::System, CommandResultDisplay::User);
    println!("CommandSource: {:?}", CommandSource::Plugin);
    println!();

    // Demonstrate CommandResult
    println!("=== Command Results ===");
    let text_result = CommandResult::Text {
        value: "Hello from command!".to_string(),
    };
    println!("Text result: {:?}", text_result);

    let skip_result = CommandResult::Skip;
    println!("Skip result: {:?}", skip_result);

    let compact_result = CommandResult::Compact {
        display_text: Some("Compacted".to_string()),
    };
    println!("Compact result: {:?}", compact_result);
    println!();

    // Register command with registry
    println!("=== Using Command Registry ===");
    let registry = CommandRegistry::global();
    registry.register(cmd);

    // List registered commands
    let commands = registry.all_commands();
    println!("Registered commands: {}", commands.len());
    for name in &commands {
        println!("  - {}", name);
    }

    println!("\n=== done ===");
    Ok(())
}

/// Create a greet command
fn create_greet_command() -> PluginCommand {
    PluginCommand {
        name: "greet".to_string(),
        description: "Greet with a custom message".to_string(),
        allowed_tools: vec!["Read".to_string()],
        argument_hint: Some("<name>".to_string()),
        arg_names: vec!["name".to_string()],
        when_to_use: Some("When you want to greet someone".to_string()),
        version: Some("1.0.0".to_string()),
        model: None,
        effort: None,
        disable_model_invocation: false,
        user_invocable: true,
        content: "Hello, {name}! Welcome to AI Agent SDK.".to_string(),
        content_length: 0,
        source_path: None,
        plugin_name: "example".to_string(),
        plugin_source: "local".to_string(),
        is_skill: false,
    }
}