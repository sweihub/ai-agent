use crate::constants::env::ai_code;

pub fn is_billed_as_extra_usage(
    model: Option<&str>,
    is_fast_mode: bool,
    is_opus1m_merged: bool,
) -> bool {
    if !is_claude_ai_subscriber() {
        return false;
    }
    if is_fast_mode {
        return true;
    }

    let model = match model {
        Some(m) => m,
        None => return false,
    };

    if !has_1m_context(model) {
        return false;
    }

    let m = model.to_lowercase().replace("[1m]", "").trim().to_string();

    let is_opus46 = m == "opus" || m.contains("opus-4-6");
    let is_sonnet46 = m == "sonnet" || m.contains("sonnet-4-6");

    if is_opus46 && is_opus1m_merged {
        return false;
    }

    is_opus46 || is_sonnet46
}

fn is_claude_ai_subscriber() -> bool {
    std::env::var(ai_code::SUBSCRIBER)
        .map(|v| v == "true")
        .unwrap_or(false)
}

fn has_1m_context(model: &str) -> bool {
    model.contains("1m") || model.contains("opus-1") || model.contains("sonnet-1")
}
