// Source: ~/claudecode/openclaudecode/src/tools/AgentTool/agentColorManager.ts
use crate::tools::agent::{AgentDefinition, get_agent_color_map};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentColorName {
    Red,
    Blue,
    Green,
    Yellow,
    Purple,
    Orange,
    Pink,
    Cyan,
}

impl AgentColorName {
    pub fn all() -> &'static [AgentColorName] {
        &[
            AgentColorName::Red,
            AgentColorName::Blue,
            AgentColorName::Green,
            AgentColorName::Yellow,
            AgentColorName::Purple,
            AgentColorName::Orange,
            AgentColorName::Pink,
            AgentColorName::Cyan,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AgentColorName::Red => "red",
            AgentColorName::Blue => "blue",
            AgentColorName::Green => "green",
            AgentColorName::Yellow => "yellow",
            AgentColorName::Purple => "purple",
            AgentColorName::Orange => "orange",
            AgentColorName::Pink => "pink",
            AgentColorName::Cyan => "cyan",
        }
    }

    pub fn to_theme_color(&self) -> &'static str {
        match self {
            AgentColorName::Red => "red_FOR_SUBAGENTS_ONLY",
            AgentColorName::Blue => "blue_FOR_SUBAGENTS_ONLY",
            AgentColorName::Green => "green_FOR_SUBAGENTS_ONLY",
            AgentColorName::Yellow => "yellow_FOR_SUBAGENTS_ONLY",
            AgentColorName::Purple => "purple_FOR_SUBAGENTS_ONLY",
            AgentColorName::Orange => "orange_FOR_SUBAGENTS_ONLY",
            AgentColorName::Pink => "pink_FOR_SUBAGENTS_ONLY",
            AgentColorName::Cyan => "cyan_FOR_SUBAGENTS_ONLY",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "red" => Some(AgentColorName::Red),
            "blue" => Some(AgentColorName::Blue),
            "green" => Some(AgentColorName::Green),
            "yellow" => Some(AgentColorName::Yellow),
            "purple" => Some(AgentColorName::Purple),
            "orange" => Some(AgentColorName::Orange),
            "pink" => Some(AgentColorName::Pink),
            "cyan" => Some(AgentColorName::Cyan),
            _ => None,
        }
    }
}

const AGENT_COLORS: &[AgentColorName] = &[
    AgentColorName::Red,
    AgentColorName::Blue,
    AgentColorName::Green,
    AgentColorName::Yellow,
    AgentColorName::Purple,
    AgentColorName::Orange,
    AgentColorName::Pink,
    AgentColorName::Cyan,
];

/// Get the theme color for an agent type.
/// Returns `None` for the general-purpose agent or if no color is assigned.
pub fn get_agent_color(agent_type: &str) -> Option<&'static str> {
    if agent_type == "general-purpose" {
        return None;
    }

    let map = get_agent_color_map().lock().unwrap();
    map.get(agent_type)
        .and_then(|color_str| AgentColorName::from_str(color_str).map(|c| c.to_theme_color()))
}

/// Set or clear the color for an agent type.
pub fn set_agent_color(agent_type: &str, color: Option<AgentColorName>) {
    let mut map = get_agent_color_map().lock().unwrap();
    match color {
        None => {
            map.remove(agent_type);
        }
        Some(c) => {
            if AGENT_COLORS.contains(&c) {
                map.insert(agent_type.to_string(), c.as_str().to_string());
            }
        }
    }
}
