use super::Command;

pub fn create_skills_command() -> Command {
    Command::local("skills", "Manage custom skills")
        .argument_hint("[list|add|remove] [<skill-name>]")
}

pub fn create_plugins_command() -> Command {
    Command::local("plugins", "Manage plugins").argument_hint("[list|add|remove] [<plugin-name>]")
}

pub fn create_hooks_command() -> Command {
    Command::local("hooks", "Manage hooks").argument_hint("[list|add|remove] [<hook-name>]")
}

pub fn create_memory_command() -> Command {
    Command::local("memory", "Manage memory").argument_hint("[on|off|clear]")
}

pub fn create_ide_command() -> Command {
    Command::local("ide", "Open files in IDE").argument_hint("<file>")
}
