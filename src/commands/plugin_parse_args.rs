// Parse plugin subcommand arguments into structured commands

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ParsedCommand {
    #[serde(rename = "menu")]
    Menu,
    #[serde(rename = "help")]
    Help,
    #[serde(rename = "install")]
    Install {
        #[serde(skip_serializing_if = "Option::is_none")]
        marketplace: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
    },
    #[serde(rename = "manage")]
    Manage,
    #[serde(rename = "uninstall")]
    Uninstall {
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
    },
    #[serde(rename = "enable")]
    Enable {
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
    },
    #[serde(rename = "disable")]
    Disable {
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
    },
    #[serde(rename = "validate")]
    Validate {
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    #[serde(rename = "marketplace")]
    Marketplace {
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
    },
}

pub fn parse_plugin_args(args: Option<&str>) -> ParsedCommand {
    let args = match args {
        Some(a) => a.trim(),
        None => "",
    };
    if args.is_empty() {
        return ParsedCommand::Menu;
    }

    let parts: Vec<&str> = args.split_whitespace().collect();
    let command = parts.first().map(|s| s.to_lowercase());

    match command.as_deref() {
        Some("help") | Some("--help") | Some("-h") => ParsedCommand::Help,

        Some("install") | Some("i") => {
            let target = parts.get(1).map(|s| s.to_string());

            if let Some(t) = target {
                // Check if it's in format plugin@marketplace
                if t.contains('@') {
                    let parts: Vec<&str> = t.split('@').collect();
                    if parts.len() == 2 {
                        return ParsedCommand::Install {
                            plugin: Some(parts[0].to_string()),
                            marketplace: Some(parts[1].to_string()),
                        };
                    }
                }

                // Check if the target looks like a marketplace (URL or path)
                let is_marketplace = t.starts_with("http://")
                    || t.starts_with("https://")
                    || t.starts_with("file://")
                    || t.contains('/')
                    || t.contains('\\');

                if is_marketplace {
                    return ParsedCommand::Install {
                        marketplace: Some(t),
                        plugin: None,
                    };
                }

                // Otherwise treat it as a plugin name
                return ParsedCommand::Install {
                    plugin: Some(t),
                    marketplace: None,
                };
            }

            ParsedCommand::Install {
                marketplace: None,
                plugin: None,
            }
        }

        Some("manage") => ParsedCommand::Manage,

        Some("uninstall") => ParsedCommand::Uninstall {
            plugin: parts.get(1).map(|s| s.to_string()),
        },

        Some("enable") => ParsedCommand::Enable {
            plugin: parts.get(1).map(|s| s.to_string()),
        },

        Some("disable") => ParsedCommand::Disable {
            plugin: parts.get(1).map(|s| s.to_string()),
        },

        Some("validate") => {
            let target = parts[1..].join(" ").trim().to_string();
            ParsedCommand::Validate {
                path: if target.is_empty() {
                    None
                } else {
                    Some(target)
                },
            }
        }

        Some("marketplace") | Some("market") => {
            let action = parts.get(1).map(|s| s.to_lowercase());
            let target = parts[2..].join(" ");

            match action.as_deref() {
                Some("add") => ParsedCommand::Marketplace {
                    action: Some("add".to_string()),
                    target: if target.is_empty() {
                        None
                    } else {
                        Some(target)
                    },
                },
                Some("remove") | Some("rm") => ParsedCommand::Marketplace {
                    action: Some("remove".to_string()),
                    target: if target.is_empty() {
                        None
                    } else {
                        Some(target)
                    },
                },
                Some("update") => ParsedCommand::Marketplace {
                    action: Some("update".to_string()),
                    target: if target.is_empty() {
                        None
                    } else {
                        Some(target)
                    },
                },
                Some("list") => ParsedCommand::Marketplace {
                    action: Some("list".to_string()),
                    target: None,
                },
                _ => ParsedCommand::Marketplace {
                    action: None,
                    target: None,
                },
            }
        }

        // Unknown command, show menu
        _ => ParsedCommand::Menu,
    }
}
