//! Bundled skills - ported from openclaudecode/src/skills/bundled/
//!
//! Individual skill implementations that ship with the SDK.

pub mod batch;
pub mod claude_api;
pub mod claude_in_chrome;
pub mod debug;
pub mod dream;
pub mod hunter;
pub mod keybindings;
pub mod loop_skill; // Note: "loop" is a Rust keyword, using loop_skill
pub mod lorem_ipsum;
pub mod remember;
pub mod run_skill_generator;
pub mod schedule_remote_agents;
pub mod simplify;
pub mod skillify;
pub mod stuck;
pub mod update_config;
pub mod verify;

/// Initialize all bundled skills.
/// Called at startup to register skills that ship with the SDK.
pub fn init_bundled_skills() {
    update_config::register_update_config_skill();
    keybindings::register_keybindings_skill();
    verify::register_verify_skill();
    debug::register_debug_skill();
    lorem_ipsum::register_lorem_ipsum_skill();
    skillify::register_skillify_skill();
    remember::register_remember_skill();
    simplify::register_simplify_skill();
    batch::register_batch_skill();
    stuck::register_stuck_skill();
    loop_skill::register_loop_skill();

    // Feature-gated in TypeScript (KAIROS, REVIEW_ARTIFACT, AGENT_TRIGGERS,
    // BUILDING_CLAUDE_APPS, RUN_SKILL_GENERATOR) — always enabled in Rust per SDK policy
    dream::register_dream_skill();
    hunter::register_hunter_skill();
    schedule_remote_agents::register_schedule_remote_agents_skill();
    claude_api::register_claude_api_skill();
    claude_in_chrome::register_claude_in_chrome_skill();
    run_skill_generator::register_run_skill_generator_skill();
}
