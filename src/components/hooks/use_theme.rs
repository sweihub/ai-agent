use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub is_dark: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeColors {
    pub background: String,
    pub foreground: String,
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub error: String,
    pub warning: String,
    pub success: String,
}

impl Theme {
    pub fn default_light() -> Self {
        Self {
            name: "light".to_string(),
            colors: ThemeColors {
                background: "#ffffff".to_string(),
                foreground: "#000000".to_string(),
                primary: "#0066cc".to_string(),
                secondary: "#666666".to_string(),
                accent: "#00cc66".to_string(),
                error: "#cc0000".to_string(),
                warning: "#cc8800".to_string(),
                success: "#00cc00".to_string(),
            },
            is_dark: false,
        }
    }

    pub fn default_dark() -> Self {
        Self {
            name: "dark".to_string(),
            colors: ThemeColors {
                background: "#1e1e1e".to_string(),
                foreground: "#d4d4d4".to_string(),
                primary: "#4fc3f7".to_string(),
                secondary: "#9e9e9e".to_string(),
                accent: "#81c784".to_string(),
                error: "#f44336".to_string(),
                warning: "#ffb74d".to_string(),
                success: "#66bb6a".to_string(),
            },
            is_dark: true,
        }
    }
}
