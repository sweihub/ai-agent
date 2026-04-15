/**
 * Example 17: Use External Skill with Agent
 *
 * Demonstrates:
 * 1. Load skills from SKILL.md files
 * 2. Register skill with agent
 * 3. Agent invokes Skill tool to get skill content
 * 4. Agent reads skill and executes the commands using other tools
 *
 * Run: cargo run --example 17_use_skill
 */
use ai_agent::{Agent, tools::skill::register_skills_from_dir};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 17: Use External Skill with Agent ---\n");

    // Step 1: Register skills from examples/skills directory
    register_skills_from_dir(Path::new("examples/skills"));
    println!("Skills registered from examples/skills/");

    // Verify skill is loaded
    use ai_agent::tools::skill::get_skill;
    if let Some(skill) = get_skill("singer") {
        println!("Skill 'singer' loaded: {}", skill.metadata.description);
    }

    // Step 2: Create agent and prompt it to use the skill
    println!("\n--- Agent using Skill tool ---\n");
    let mut agent = Agent::new(
        &std::env::var("AI_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".to_string()),
        10,
    );
    agent.set_system_prompt(
        "You have access to a Skill tool. When user asks to use a skill, invoke it with Skill tool. \
        The Skill tool will return the skill content. Read the skill content and use appropriate tools (like Bash) to execute the commands in the skill."
    );

    // Prompt the agent to use the singer skill - it should invoke Skill tool,
    // then read the skill content and use Bash to execute the echo command
    let result = agent.prompt("Please sing a song using the 'singer' skill. Execute the skill's commands.").await?;

    println!("--- Agent Response ---\n");
    println!("{}", result.text.trim());

    println!("\n=== done ===");
    Ok(())
}