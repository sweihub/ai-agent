#![allow(dead_code)]

/// Logs a tengu_skill_loaded event for each skill available at session startup.
/// This enables analytics on which skills are available across sessions.
pub async fn log_skills_loaded(
    _cwd: String,
    _context_window_tokens: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
