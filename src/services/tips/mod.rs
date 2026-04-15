pub mod tip_history;
pub mod tip_registry;
pub mod tip_scheduler;
pub mod types;

pub use tip_history::{get_sessions_since_last_shown, record_tip_shown};
pub use tip_scheduler::{
    get_tip_to_show_on_spinner, record_shown_tip, select_tip_with_longest_time_since_shown,
};
pub use types::{Tip, TipContext};
