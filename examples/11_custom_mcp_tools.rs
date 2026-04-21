/**
 * Example 11: Custom Tools with create_tool() + annotations
 *
 * Shows how to define custom tools with annotations for
 * read-only hints and concurrency safety.
 *
 * Run: cargo run --example 11_custom_mcp_tools
 */
use ai_agent::{create_tool_with_annotations, sdk_tool_to_tool_definition, Agent, AgentOptions, ToolAnnotations};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 11: Custom Tools ---\n");

    // Define tools using JSON schema (similar to Zod in TypeScript)
    let get_temperature = create_tool_with_annotations(
        "get_temperature",
        "Get the current temperature at a location",
        serde_json::json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "City name"
                },
                "unit": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"],
                    "default": "celsius",
                    "description": "Temperature unit"
                }
            },
            "required": ["city"]
        }),
        ToolAnnotations {
            read_only_hint: Some(true),
            ..Default::default()
        },
    );
    let get_temperature: ai_agent::ToolDefinition = sdk_tool_to_tool_definition(get_temperature);

    let convert_units = create_tool_with_annotations(
        "convert_units",
        "Convert between measurement units",
        serde_json::json!({
            "type": "object",
            "properties": {
                "value": {
                    "type": "number",
                    "description": "Value to convert"
                },
                "from_unit": {
                    "type": "string",
                    "description": "Source unit"
                },
                "to_unit": {
                    "type": "string",
                    "description": "Target unit"
                }
            },
            "required": ["value", "from_unit", "to_unit"]
        }),
        ToolAnnotations::default(),
    );
    let convert_units: ai_agent::ToolDefinition = sdk_tool_to_tool_definition(convert_units);

    let builtin_tools = ai_agent::get_all_tools();
    let mut all_tools = builtin_tools;
    all_tools.push(get_temperature);
    all_tools.push(convert_units);

    let mut agent = Agent::create(AgentOptions {
        model: Some(std::env::var("AI_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".to_string())),
        tools: all_tools,
        ..Default::default()
    });

    println!("Loaded {} tools\n", agent.get_tools().len());

    let result = agent.query(
        "What is the temperature in Tokyo and Paris? Also convert 10 km to miles. Be brief."
    ).await?;

    println!("Answer: {}", result.text);

    Ok(())
}