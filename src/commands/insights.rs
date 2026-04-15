// Source: /data/home/swei/claudecode/openclaudecode/src/commands/insights.ts
use super::Command;

pub fn create_insights_command() -> Command {
    Command::prompt(
        "insights",
        "Analyze session history and provide usage insights",
    )
    .argument_hint("[--refresh]")
}

const EXTENSION_TO_LANGUAGE: &[(&str, &str)] = &[
    (".ts", "TypeScript"),
    (".tsx", "TypeScript"),
    (".js", "JavaScript"),
    (".jsx", "JavaScript"),
    (".py", "Python"),
    (".rb", "Ruby"),
    (".go", "Go"),
    (".rs", "Rust"),
    (".java", "Java"),
    (".md", "Markdown"),
    (".json", "JSON"),
    (".yaml", "YAML"),
    (".yml", "YAML"),
    (".sh", "Shell"),
    (".css", "CSS"),
    (".html", "HTML"),
];

pub fn get_language_from_extension(ext: &str) -> Option<&'static str> {
    EXTENSION_TO_LANGUAGE
        .iter()
        .find(|(e, _)| *e == ext)
        .map(|(_, lang)| *lang)
}
