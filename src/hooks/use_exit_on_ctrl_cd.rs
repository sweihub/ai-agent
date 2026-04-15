// Source: ~/claudecode/openclaudecode/src/hooks/useExitOnCtrlCD.ts
#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::hooks::use_double_press::{DoublePress, DOUBLE_PRESS_TIMEOUT_MS};

/// State of the exit prompt.
#[derive(Debug, Clone, Default)]
pub struct ExitState {
    pub pending: bool,
    pub key_name: Option<ExitKeyName>,
}

/// Which key triggered the exit prompt.
#[derive(Debug, Clone, Copy)]
pub enum ExitKeyName {
    CtrlC,
    CtrlD,
}

/// Options for a keybinding handler.
pub struct KeybindingOptions {
    pub context: Option<String>,
    pub is_active: bool,
}

/// Type for a keybinding registration function.
pub type UseKeybindingsFn =
    Arc<dyn Fn(serde_json::Value, Option<KeybindingOptions>) + Send + Sync>;

/// Handle ctrl+c and ctrl+d for exiting the application.
///
/// Uses a time-based double-press mechanism:
/// - First press: Shows "Press X again to exit" message
/// - Second press within timeout: Exits the application
///
/// Note: We use time-based double-press rather than the chord system because
/// we want the first ctrl+c to also trigger interrupt (handled elsewhere).
/// The chord system would prevent the first press from firing any action.
///
/// These keys are hardcoded and cannot be rebound via keybindings.json.
///
/// Translation of the React `useExitOnCtrlCD` hook.
/// In Rust this is a struct that manages exit state and provides handlers.
pub struct ExitOnCtrlCD {
    exit_state: Arc<Mutex<ExitState>>,
    exit_fn: Arc<dyn Fn() + Send + Sync>,
    on_interrupt: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
    ctrl_c_handler: Arc<DoublePress>,
    ctrl_d_handler: Arc<DoublePress>,
}

impl ExitOnCtrlCD {
    /// Create a new exit handler.
    ///
    /// - `exit_fn`: custom exit handler (or uses default)
    /// - `on_interrupt`: optional callback for features to handle interrupt (ctrl+c).
    ///   Return true if handled, false to fall through to double-press exit.
    /// - `on_exit`: optional custom exit handler
    /// - `is_active`: whether the keybinding is active
    pub fn new(
        exit_fn: impl Fn() + Send + Sync + 'static,
        on_interrupt: Option<impl Fn() -> bool + Send + Sync + 'static>,
        on_exit: Option<impl Fn() + Send + Sync + 'static>,
        is_active: bool,
    ) -> Self {
        let exit_state = Arc::new(Mutex::new(ExitState::default()));
        let exit_fn: Arc<dyn Fn() + Send + Sync> = if let Some(on_exit) = on_exit {
            Arc::new(on_exit)
        } else {
            Arc::new(exit_fn)
        };

        let exit_state_for_ctrl_c = Arc::clone(&exit_state);
        let exit_fn_for_ctrl_c = Arc::clone(&exit_fn);
        let ctrl_c_handler = Arc::new(DoublePress::new(
            move |pending| {
                let key_name = if pending {
                    Some(ExitKeyName::CtrlC)
                } else {
                    None
                };
                // Can't use tokio::spawn here; use a simple approach.
                // In a real app this would be called from an async context.
                // We store the state for later retrieval.
                let state = Arc::clone(&exit_state_for_ctrl_c);
                let exit_fn = Arc::clone(&exit_fn_for_ctrl_c);
                tokio::spawn(async move {
                    let mut guard = state.lock().await;
                    guard.pending = pending;
                    guard.key_name = key_name;
                    if !pending {
                        exit_fn();
                    }
                });
            },
            move || {
                exit_fn_for_ctrl_c();
            },
            None::<fn()>,
        ));

        let exit_state_for_ctrl_d = Arc::clone(&exit_state);
        let exit_fn_for_ctrl_d = Arc::clone(&exit_fn);
        let ctrl_d_handler = Arc::new(DoublePress::new(
            move |pending| {
                let key_name = if pending {
                    Some(ExitKeyName::CtrlD)
                } else {
                    None
                };
                let state = Arc::clone(&exit_state_for_ctrl_d);
                let exit_fn = Arc::clone(&exit_fn_for_ctrl_d);
                tokio::spawn(async move {
                    let mut guard = state.lock().await;
                    guard.pending = pending;
                    guard.key_name = key_name;
                    if !pending {
                        exit_fn();
                    }
                });
            },
            move || {
                exit_fn_for_ctrl_d();
            },
            None::<fn()>,
        ));

        Self {
            exit_state,
            exit_fn,
            on_interrupt: on_interrupt.map(|f| Arc::new(f) as Arc<dyn Fn() -> bool + Send + Sync>),
            ctrl_c_handler,
            ctrl_d_handler,
        }
    }

    /// Handle the interrupt signal (ctrl+c).
    ///
    /// Lets features handle interrupt first via callback.
    pub async fn handle_interrupt(&self) {
        if let Some(ref on_interrupt) = self.on_interrupt {
            if on_interrupt() {
                return; // Feature handled it
            }
        }
        self.ctrl_c_handler.press().await;
    }

    /// Handle the exit signal (ctrl+d).
    pub async fn handle_exit(&self) {
        self.ctrl_d_handler.press().await;
    }

    /// Get the current exit state.
    pub async fn get_exit_state(&self) -> ExitState {
        self.exit_state.lock().await.clone()
    }

    /// Register the keybinding handlers with the given keybindings function.
    pub fn register_keybindings(&self, use_keybindings: &UseKeybindingsFn) {
        let interrupt_handler = Arc::clone(&self.ctrl_c_handler);
        let exit_handler = Arc::clone(&self.ctrl_d_handler);
        let on_interrupt = self.on_interrupt.clone();
        let exit_state = Arc::clone(&self.exit_state);

        // Build handler map equivalent to the useMemo in TypeScript
        use_keybindings(
            serde_json::json!({
                "app:interrupt": "interrupt_handler",
                "app:exit": "exit_handler",
            }),
            Some(KeybindingOptions {
                context: Some("Global".to_string()),
                is_active: true,
            }),
        );
    }
}
