#![allow(dead_code)]

const UTF8_BOM: &str = "\u{FEFF}";

pub fn strip_bom(content: &str) -> String {
    if content.starts_with(UTF8_BOM) {
        content[UTF8_BOM.len()..].to_string()
    } else {
        content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_bom_with_bom() {
        let content = "\u{FEFF}{\"key\": \"value\"}";
        assert_eq!(strip_bom(content), "{\"key\": \"value\"}");
    }

    #[test]
    fn test_strip_bom_without_bom() {
        let content = "{\"key\": \"value\"}";
        assert_eq!(strip_bom(content), "{\"key\": \"value\"}");
    }
}
