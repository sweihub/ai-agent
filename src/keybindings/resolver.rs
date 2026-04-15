// Source: /data/home/swei/claudecode/openclaudecode/src/keybindings/resolver.ts
//! Keybinding resolver

use std::collections::HashMap;

pub fn resolve_keybinding(
    context: &str,
    action: &str,
    bindings: &HashMap<String, HashMap<String, String>>,
) -> Option<String> {
    bindings
        .get(context)
        .and_then(|ctx| ctx.get(action).cloned())
}
