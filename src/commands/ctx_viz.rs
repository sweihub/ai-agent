use super::Command;

pub fn create_ctx_viz_command() -> Command {
    Command::local("ctx-viz", "Visualize context")
}
