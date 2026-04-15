// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/normalization.ts
const CLAUDEAI_SERVER_PREFIX: &str = "claude.ai ";

pub fn normalize_name_for_mcp(name: &str) -> String {
    let normalized: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();

    if name.starts_with(CLAUDEAI_SERVER_PREFIX) {
        let re = regex::Regex::new(r"_+").unwrap();
        let result = re.replace_all(&normalized, "_");
        result.trim_matches('_').to_string()
    } else {
        normalized
    }
}
