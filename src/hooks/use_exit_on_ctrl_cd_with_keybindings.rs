// Source: ~/claudecode/openclaudecode/src/hooks/useExitOnCtrlCDWithKeybindings.ts
#![allow(dead_code)]

use crate::hooks::use_exit_on_ctrl_cd::{ExitOnCtrlCD, ExitState};

pub use crate::hooks::use_exit_on_ctrl_cd::ExitState as ExitStateType;

/// Convenience function that wires up `ExitOnCtrlCD` with keybindings.
///
/// This is the standard way to use `useExitOnCtrlCD` in components.
/// The separation exists to avoid import cycles -- `useExitOnCtrlCD`
/// doesn't import from the keybindings module directly.
///
/// Translation of the React `useExitOnCtrlCDWithKeybindings` hook.
/// In Rust this is a plain constructor function.
///
/// - `exit_fn`: the function to call on exit
/// - `on_exit`: optional custom exit handler
/// - `on_interrupt`: optional callback for features to handle interrupt (ctrl+c).
///   Return true if handled, false to fall through to double-press exit.
/// - `is_active`: whether the keybinding is active
pub fn exit_on_ctrl_cd_with_keybindings(
    exit_fn: impl Fn() + Send + Sync + 'static,
    on_exit: Option<impl Fn() + Send + Sync + 'static>,
    on_interrupt: Option<impl Fn() -> bool + Send + Sync + 'static>,
    is_active: bool,
) -> ExitOnCtrlCD {
    ExitOnCtrlCD::new(exit_fn, on_interrupt, on_exit, is_active)
}
