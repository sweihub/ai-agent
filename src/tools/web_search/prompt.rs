// Source: ~/claudecode/openclaudecode/src/tools/WebSearchTool/prompt.ts
use chrono::Local;

pub const WEB_SEARCH_TOOL_NAME: &str = "WebSearch";

#[allow(dead_code)]
pub fn get_web_search_prompt() -> String {
    let current_month_year = get_local_month_year();
    format!(
        r#"
- Allows Claude to search the web and use the results to inform responses
- Provides up-to-date information for current events and recent data
- Returns search result information formatted as search result blocks, including links as markdown hyperlinks
- Use this tool for accessing information beyond Claude's knowledge cutoff
- Searches are performed automatically within a single API call

CRITICAL REQUIREMENT - You MUST follow this:
  - After answering the user's question, you MUST include a "Sources:" section at the end of your response
  - In the Sources section, list all relevant URLs from the search results as markdown hyperlinks: [Title](URL)
  - This is MANDATORY - never skip including sources in your response
  - Example format:

    [Your answer here]

    Sources:
    - [Source Title 1](https://example.com/1)
    - [Source Title 2](https://example.com/2)

Usage notes:
  - Domain filtering is supported to include or block specific websites
  - Web search is only available in the US

IMPORTANT - Use the correct year in search queries:
  - The current month is {}. You MUST use this year when searching for recent information, documentation, or current events.
  - Example: If the user asks for "latest React docs", search for "React documentation" with the current year, NOT last year
"#,
        current_month_year
    )
}

fn get_local_month_year() -> String {
    let now = Local::now();
    now.format("%B %Y").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_search_tool_name() {
        assert_eq!(WEB_SEARCH_TOOL_NAME, "WebSearch");
    }

    #[test]
    fn test_web_search_prompt_not_empty() {
        let prompt = get_web_search_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("Sources:"));
    }

    #[test]
    fn test_local_month_year_format() {
        let month_year = get_local_month_year();
        // Should contain a month name and a 4-digit year
        assert!(!month_year.is_empty());
        assert!(month_year.chars().any(|c| c.is_ascii_digit()));
    }
}
