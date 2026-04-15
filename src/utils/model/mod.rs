//! Model utilities
//!
//! Translated from openclaudecode/src/utils/model/

pub mod model;
pub mod model_1m_access;
pub mod model_validate;

pub use model::{
    first_party_name_to_canonical, get_best_model, get_canonical_name,
    get_claude_ai_user_default_model_description, get_default_haiku_model,
    get_default_main_loop_model, get_default_opus_model, get_default_sonnet_model,
    get_main_loop_model, get_public_model_display_name, get_public_model_name,
    get_small_fast_model, get_user_specified_model_setting, is_legacy_model_remap_enabled,
    is_opus_1m_merge_enabled, model_display_string, normalize_model_string_for_api,
    parse_user_specified_model, render_default_model_setting, render_model_name,
    render_model_setting, resolve_skill_model_override, ModelName, ModelSetting, ModelShortName,
};

pub use model_1m_access::{check_opus_1m_access, check_sonnet_1m_access};

pub use model_validate::{validate_model, ModelValidationResult};
