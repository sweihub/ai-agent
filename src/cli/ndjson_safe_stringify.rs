//! NDJSON safe stringification

const JS_LINE_TERMINATORS: &str = "\u{2028}\u{2029}";

fn escape_js_line_terminators(json: &str) -> String {
    json.replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

/// JSON.stringify for one-message-per-line transports. Escapes U+2028
/// LINE SEPARATOR and U+2029 PARAGRAPH SEPARATOR so the serialized output
/// cannot be broken by a line-splitting receiver.
pub fn ndjson_safe_stringify<T: serde::Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let json = serde_json::to_string(value)?;
    Ok(escape_js_line_terminators(&json))
}
