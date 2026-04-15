use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JustifyContent {
    FlexStart,
    Center,
    FlexEnd,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlignItems {
    FlexStart,
    Center,
    FlexEnd,
    Stretch,
    Baseline,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PositionType {
    Relative,
    Absolute,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YogaNode {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub position_type: PositionType,
    pub padding: [f32; 4],
    pub margin: [f32; 4],
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Option<f32>,
}

impl Default for YogaNode {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
            position_type: PositionType::Relative,
            padding: [0.0; 4],
            margin: [0.0; 4],
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: None,
        }
    }
}

impl YogaNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_flex_direction(mut self, direction: FlexDirection) -> Self {
        self.flex_direction = direction;
        self
    }

    pub fn with_justify_content(mut self, justify: JustifyContent) -> Self {
        self.justify_content = justify;
        self
    }

    pub fn with_align_items(mut self, align: AlignItems) -> Self {
        self.align_items = align;
        self
    }
}
