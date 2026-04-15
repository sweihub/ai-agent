use super::Command;

pub fn create_ant_trace_command() -> Command {
    Command::local("ant-trace", "Trace ant operations")
        .argument_hint("[start|stop|show] [<trace-id>]")
}
