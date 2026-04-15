use regex::Regex;

lazy_static::lazy_static! {
    static ref XML_TAG_BLOCK_PATTERN: Regex = Regex::new(r"<([a-z][\w-]*)(?:\s[^>]*)?>[\s\S]*?<\/\1>\n?").unwrap();
    static ref IDE_CONTEXT_TAGS_PATTERN: Regex = Regex::new(r"<(ide_opened_file|ide_selection)(?:\s[^>]*)?>[\s\S]*?<\/\1>\n?").unwrap();
}

pub fn strip_display_tags(text: &str) -> String {
    let result = XML_TAG_BLOCK_PATTERN
        .replace_all(text, "")
        .trim()
        .to_string();
    if result.is_empty() {
        text.to_string()
    } else {
        result
    }
}

pub fn strip_display_tags_allow_empty(text: &str) -> String {
    XML_TAG_BLOCK_PATTERN
        .replace_all(text, "")
        .trim()
        .to_string()
}

pub fn strip_ide_context_tags(text: &str) -> String {
    IDE_CONTEXT_TAGS_PATTERN
        .replace_all(text, "")
        .trim()
        .to_string()
}
