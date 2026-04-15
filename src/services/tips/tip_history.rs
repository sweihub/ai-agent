use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

static TIPS_HISTORY: Lazy<Mutex<HashMap<String, i64>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static NUM_STARTUPS: Lazy<Mutex<i64>> = Lazy::new(|| Mutex::new(0));

pub fn record_tip_shown(tip_id: &str) {
    let num_startups = *NUM_STARTUPS.lock().unwrap();
    let mut history = TIPS_HISTORY.lock().unwrap();
    history.insert(tip_id.to_string(), num_startups);
}

pub fn get_sessions_since_last_shown(tip_id: &str) -> i64 {
    let num_startups = *NUM_STARTUPS.lock().unwrap();
    let history = TIPS_HISTORY.lock().unwrap();
    match history.get(tip_id) {
        Some(&last) => num_startups - last,
        None => i64::MAX,
    }
}
