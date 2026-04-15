//! Session title utilities.

/// Generate a session title from user input
pub fn generate_session_title(user_input: &str) -> String {
    let input = user_input.trim();

    if input.is_empty() {
        return "New Session".to_string();
    }

    // Get first line or first sentence
    let first_line = input.lines().next().unwrap_or(input);
    let title = first_line
        .split(&['.', '!', '?'][..])
        .next()
        .unwrap_or(first_line);

    // Truncate to reasonable length
    let title = title.trim();
    if title.len() > 50 {
        format!("{}...", &title[..47])
    } else {
        title.to_string()
    }
}

/// Clean a session title for use in filenames
pub fn clean_title_for_filename(title: &str) -> String {
    title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
}

/// Get a short description of the session
pub fn get_short_description(user_input: &str) -> String {
    let words: Vec<&str> = user_input.split_whitespace().take(5).collect();
    words.join(" ")
}
