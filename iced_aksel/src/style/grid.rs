use iced_core::{Color, Pixels};

/// Style of the grid lines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridStyle {
    /// The color of the grid lines.
    pub color: Color,
    /// The thickness of the grid lines in pixels.
    pub width: Pixels,
    /// Whether the grid lines should be dashed.
    pub dashed: bool,
}