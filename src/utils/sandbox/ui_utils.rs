pub fn remove_sandbox_violation_tags(text: &str) -> String {
    let re = regex::Regex::new(r"<sandbox_violations>[\s\S]*?</sandbox_violations>").unwrap();
    re.replace_all(text, "").to_string()
}
