// Source: ~/claudecode/openclaudecode/src/utils/xml.rs

/// Escape XML/HTML special characters for safe interpolation into element
/// text content (between tags). Use when untrusted strings (process stdout,
/// user input, external data) go inside `<tag>${here}</tag>`.
pub fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escape for interpolation into a double- or single-quoted attribute value:
/// `<tag attr="${here}">`. Escapes quotes in addition to `& < >`.
pub fn escape_xml_attr(s: &str) -> String {
    escape_xml(s)
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("a < b & c > d"), "a &lt; b &amp; c &gt; d");
    }

    #[test]
    fn test_escape_xml_attr() {
        assert_eq!(
            escape_xml_attr("say \"hello\""),
            "say &quot;hello&quot;"
        );
        assert_eq!(
            escape_xml_attr("it's"),
            "it&apos;s"
        );
        assert_eq!(
            escape_xml_attr("<a href=\"test\">link</a>"),
            "&lt;a href=&quot;test&quot;&gt;link&lt;/a&gt;"
        );
    }

    #[test]
    fn test_escape_empty() {
        assert_eq!(escape_xml(""), "");
        assert_eq!(escape_xml_attr(""), "");
    }
}
