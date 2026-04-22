pub mod keyword;

pub use keyword::{
    TriggerPosition, find_ultraplan_trigger_positions, find_ultrareview_trigger_positions,
    has_ultraplan_keyword, has_ultrareview_keyword, replace_ultraplan_keyword,
};
