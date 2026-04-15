use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayContext {
    pub is_visible: bool,
    pub overlay_type: Option<OverlayType>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OverlayType {
    Help,
    Menu,
    Search,
    CommandPalette,
}

impl Default for OverlayContext {
    fn default() -> Self {
        Self {
            is_visible: false,
            overlay_type: None,
            content: None,
        }
    }
}

impl OverlayContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, overlay_type: OverlayType) {
        self.is_visible = true;
        self.overlay_type = Some(overlay_type);
    }

    pub fn hide(&mut self) {
        self.is_visible = false;
        self.overlay_type = None;
        self.content = None;
    }

    pub fn toggle(&mut self, overlay_type: OverlayType) {
        if self.is_visible && self.overlay_type == Some(overlay_type) {
            self.hide();
        } else {
            self.show(overlay_type);
        }
    }
}
