pub fn parse_frontmatter(content: &str) -> Option<Frontmatter> {
    if !content.starts_with("---") {
        return None;
    }

    let end_marker = content[3..].find("---")?;
    let frontmatter_content = &content[3..3 + end_marker];

    let mut fm = Frontmatter::default();
    for line in frontmatter_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            let value = line[colon_pos + 1..].trim();
            fm.properties.insert(key.to_string(), value.to_string());
        }
    }

    Some(fm)
}

#[derive(Clone, Debug, Default)]
pub struct Frontmatter {
    pub properties: std::collections::HashMap<String, String>,
}

impl Frontmatter {
    pub fn get(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }
}
