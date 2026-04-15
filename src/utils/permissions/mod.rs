// Auto-generated mod.rs for permissions module
#![allow(dead_code)]

//! Permission system utilities — ported from TypeScript openclaudecode.

pub mod auto_mode_state;
pub mod bash_classifier;
pub mod bypass_permissions_killswitch;
pub mod classifier_decision;
pub mod classifier_shared;
pub mod dangerous_patterns;
pub mod denial_tracking;
pub mod filesystem;
pub mod get_next_permission_mode;
pub mod path_validation;
pub mod permission_explainer;
pub mod permission_mode;
pub mod permission_prompt_tool_result_schema;
pub mod permission_result;
pub mod permission_rule;
pub mod permission_rule_parser;
pub mod permissions;
pub mod permission_setup;
pub mod permissions_loader;
pub mod permission_update;
pub mod permission_update_schema;
pub mod shadowed_rule_detection;
pub mod shell_rule_matching;
pub mod yolo_classifier;

// Re-export all public items

pub use auto_mode_state::*;
pub use bash_classifier::*;
pub use bypass_permissions_killswitch::*;
pub use classifier_decision::*;
pub use classifier_shared::*;
pub use dangerous_patterns::*;
pub use denial_tracking::*;
pub use filesystem::*;
pub use get_next_permission_mode::*;
pub use path_validation::*;
pub use permission_explainer::*;
pub use permission_mode::*;
pub use permission_prompt_tool_result_schema::*;
pub use permission_result::*;
pub use permission_rule::*;
pub use permission_rule_parser::*;
pub use permissions::*;
pub use permission_setup::*;
pub use permissions_loader::*;
pub use permission_update::*;
pub use permission_update_schema::*;
pub use shadowed_rule_detection::*;
pub use shell_rule_matching::*;
pub use yolo_classifier::*;
