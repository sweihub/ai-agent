/**
 * Example 20: AI.md Instruction Files
 *
 * Demonstrates loading AI.md/AI.local.md instruction files.
 *
 * Run: cargo run --example 20_ai_md
 *
 * This example shows:
 * 1. Loading AI.md files from various locations
 * 2. Understanding memory types (Managed, User, Project, Local)
 * 3. Processing @include directives
 * 4. Using frontmatter for conditional rules
 */
use ai_agent::{AiMdType, get_ai_md_files, load_ai_md};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 20: AI.md Instruction Files ---\n");

    let cwd = std::env::current_dir()?;

    // Get all AI.md files
    println!("--- Loading AI.md files ---");
    match get_ai_md_files(&cwd) {
        Ok(files) => {
            println!("Found {} AI.md file(s)\n", files.len());

            for file in &files {
                let type_str = match file.md_type {
                    AiMdType::Managed => "Managed",
                    AiMdType::User => "User",
                    AiMdType::Project => "Project",
                    AiMdType::Local => "Local",
                };
                println!("  [{}] {}", type_str, file.path);
                println!("      Content: {} chars", file.content.len());
                if let Some(parent) = &file.parent {
                    println!("      Included from: {}", parent);
                }
                if let Some(globs) = &file.globs {
                    println!("      Globs: {:?}", globs);
                }
                println!();
            }
        }
        Err(e) => {
            println!("Error loading AI.md files: {:?}", e);
        }
    }

    // Load AI.md for system prompt
    println!("\n--- Loading AI.md for system prompt ---");
    match load_ai_md(&cwd) {
        Ok(Some(prompt)) => {
            println!("AI.md prompt loaded: {} chars", prompt.len());
            // Show first 500 chars
            let preview = prompt.chars().take(500).collect::<String>();
            println!("\nPreview:\n{}", preview);
        }
        Ok(None) => {
            println!("No AI.md files found");
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }

    // Explain the AI.md system
    println!("\n--- AI.md System ---");
    println!("Files are loaded in this order (later = higher priority):");
    println!("  1. Managed: /etc/ai-code/AI.md (global policy)");
    println!("  2. User: ~/.ai/AI.md (private global)");
    println!("  3. Project: ./AI.md, .ai/rules/*.md (checked in)");
    println!("  4. Local: ./AI.local.md (private project)");
    println!();
    println!("Features:");
    println!("  - @include directive for other files");
    println!("  - Frontmatter paths for conditional rules");
    println!("  - 80+ file extensions supported for @include");

    println!("\n--- done ---");
    Ok(())
}
