pub fn create_heapdump_command() -> super::Command {
    super::Command::local("heapdump", "Capture heap dump for debugging")
}
