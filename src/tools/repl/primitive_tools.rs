// Source: ~/claudecode/openclaudecode/src/tools/REPLTool/primitiveTools.ts

/// Primitive tools hidden from direct model use when REPL mode is on
/// (REPL_ONLY_TOOLS) but still accessible inside the REPL VM context.
/// Exported so display-side code (collapseReadSearch, renderers) can
/// classify/render virtual messages for these tools even when they're
/// absent from the filtered execution tools list.
///
/// Lazy getter — the import chain collapseReadSearch.ts → primitiveTools.ts
/// → FileReadTool.tsx → ... loops back through the tool registry, so a
/// top-level const hits "Cannot access before initialization". Deferring
/// to call time avoids the TDZ.
///
/// Referenced directly rather than via getAllBaseTools() because that
/// excludes Glob/Grep when hasEmbeddedSearchTools() is true.
#[allow(dead_code)]
pub fn get_repl_primitive_tools() -> Vec<&'static str> {
    vec![
        "FileRead",
        "FileWrite",
        "FileEdit",
        "Glob",
        "Grep",
        "Bash",
        "NotebookEdit",
        "Agent",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_primitive_tools_not_empty() {
        let tools = get_repl_primitive_tools();
        assert!(!tools.is_empty());
    }

    #[test]
    fn test_repl_primitive_tools_contains_expected() {
        let tools = get_repl_primitive_tools();
        assert!(tools.contains(&"FileRead"));
        assert!(tools.contains(&"Bash"));
        assert!(tools.contains(&"Glob"));
    }
}
