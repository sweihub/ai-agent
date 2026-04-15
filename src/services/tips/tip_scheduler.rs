use crate::services::tips::tip_history::{get_sessions_since_last_shown, record_tip_shown};
use crate::services::tips::types::Tip;

pub fn select_tip_with_longest_time_since_shown(available_tips: &[Tip]) -> Option<&Tip> {
    if available_tips.is_empty() {
        return None;
    }

    if available_tips.len() == 1 {
        return Some(&available_tips[0]);
    }

    let mut tips_with_sessions: Vec<(&Tip, i64)> = available_tips
        .iter()
        .map(|tip| {
            let id = tip.get("id").and_then(|v| v.as_str()).unwrap_or("");
            (tip, get_sessions_since_last_shown(id))
        })
        .collect();

    tips_with_sessions.sort_by(|a, b| b.1.cmp(&a.1));
    tips_with_sessions.first().map(|(tip, _)| *tip)
}

pub fn get_tip_to_show_on_spinner() -> Option<Tip> {
    let tips = crate::services::tips::tip_registry::get_relevant_tips();

    if tips.is_empty() {
        return None;
    }

    select_tip_with_longest_time_since_shown(&tips).cloned()
}

pub fn record_shown_tip(tip: &Tip) {
    if let Some(id) = tip.get("id").and_then(|v| v.as_str()) {
        record_tip_shown(id);
    }
}
