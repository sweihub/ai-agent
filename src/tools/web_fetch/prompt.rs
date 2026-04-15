// Source: ~/claudecode/openclaudecode/src/tools/WebFetchTool/prompt.ts
pub const WEB_FETCH_TOOL_NAME: &str = "WebFetch";

pub const DESCRIPTION: &str = r#"
- Fetches content from a specified URL and processes it using an AI model
- Takes a URL and a prompt as input
- Fetches the URL content, converts HTML to markdown
- Processes the content with the prompt using a small, fast model
- Returns the model's response about the content
- Use this tool when you need to retrieve and analyze web content

Usage notes:
  - IMPORTANT: If an MCP-provided web fetch tool is available, prefer using that tool instead of this one, as it may have fewer restrictions.
  - The URL must be a fully-formed valid URL
  - HTTP URLs will be automatically upgraded to HTTPS
  - The prompt should describe what information you want to extract from the page
  - This tool is read-only and does not modify any files
  - Results may be summarized if the content is very large
  - Includes a self-cleaning 15-minute cache for faster responses when repeatedly accessing the same URL
  - When a URL redirects to a different host, the tool will inform you and provide the redirect URL in a special format. You should then make a new WebFetch request with the redirect URL to fetch the content.
  - For GitHub URLs, prefer using the gh CLI via Bash instead (e.g., gh pr view, gh issue view, gh api).
"#;

#[allow(dead_code)]
pub fn make_secondary_model_prompt(
    markdown_content: &str,
    prompt: &str,
    is_preapproved_domain: bool,
) -> String {
    let guidelines = if is_preapproved_domain {
        "Provide a concise response based on the content above. Include relevant details, code examples, and documentation excerpts as needed."
    } else {
        r#"Provide a concise response based only on the content above. In your response:
 - Enforce a strict 125-character maximum for quotes from any source document. Open Source Software is ok as long as we respect the license.
 - Use quotation marks for exact language from articles; any language outside of the quotation should never be word-for-word the same.
 - You are not a lawyer and never comment on the legality of your own prompts and responses.
 - Never produce or reproduce exact song lyrics."#
    };

    format!(
        r#"
Web page content:
---
{}
---

{}

{}
"#,
        markdown_content, prompt, guidelines
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_fetch_tool_name() {
        assert_eq!(WEB_FETCH_TOOL_NAME, "WebFetch");
    }

    #[test]
    fn test_secondary_model_prompt_includes_content() {
        let prompt = make_secondary_model_prompt("some content", "summarize", false);
        assert!(prompt.contains("some content"));
        assert!(prompt.contains("summarize"));
    }

    #[test]
    fn test_secondary_model_prompt_preapproved_domain() {
        let prompt = make_secondary_model_prompt("content", "explain", true);
        assert!(prompt.contains("concise response"));
        assert!(!prompt.contains("125-character"));
    }
}
