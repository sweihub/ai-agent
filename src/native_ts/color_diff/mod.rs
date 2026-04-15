use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn distance(&self, other: &Color) -> f64 {
        let dr = self.r as f64 - other.r as f64;
        let dg = self.g as f64 - other.g as f64;
        let db = self.b as f64 - other.b as f64;
        (dr * dr + dg * dg + db * db).sqrt()
    }

    pub fn luminance(&self) -> f64 {
        0.299 * self.r as f64 + 0.587 * self.g as f64 + 0.114 * self.b as f64
    }
}

pub fn color_diff(c1: &Color, c2: &Color) -> f64 {
    c1.distance(c2)
}

pub fn find_closest_color(target: &Color, colors: &[Color]) -> Option<&Color> {
    colors.iter().min_by(|a, b| {
        let diff_a = a.distance(target);
        let diff_b = b.distance(target);
        diff_a
            .partial_cmp(&diff_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}
