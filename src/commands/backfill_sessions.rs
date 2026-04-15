use super::Command;

pub fn create_backfill_sessions_command() -> Command {
    Command::local("backfill-sessions", "Backfill session data")
}
