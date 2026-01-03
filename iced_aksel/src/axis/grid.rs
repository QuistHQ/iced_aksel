use iced_core::{Color, Pixels};

/// Defines the visual styling of a grid line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridLine {
    /// The visual thickness (stroke width) of the grid line.
    pub thickness: Pixels,

    /// Whether the line should be dashed.
    pub dashed: bool,

    /// The color of the grid line.
    pub color: Color,
}

impl Default for GridLine {
    fn default() -> Self {
        Self {
            thickness: Pixels(1.0),
            dashed: false,
            color: Color::from_rgb(0.8, 0.8, 0.8),
        }
    }
}