pub fn scan_for_secrets(_content: &str) -> Vec<SecretFound> {
    vec![]
}

#[derive(Debug, Clone)]
pub struct SecretFound {
    pub secret_type: String,
    pub line: usize,
}
