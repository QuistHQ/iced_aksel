use iced_core::{Color, Pixels};

use crate::style::{DashStyle, GridLineStyle};

/// Configuration for a single grid line on the chart.
///
/// Grid lines are drawn perpendicular to the axis at each tick position.
///
/// # Example
///
/// ```rust
/// use iced_aksel::axis::GridLine;
/// use iced::Pixels;
///
/// // Create a grid line with 2-pixel thickness
/// let grid_line = GridLine {
///     thickness: Pixels(2.0),
/// };
///
/// // Or use the default (1-pixel thickness)
/// let default_grid = GridLine::default();
/// ```
#[derive(Debug, Clone, Copy)]
pub struct GridLine {
    /// The thickness of the grid line in pixels.
    pub width: Pixels,
    /// The color of the gridline
    pub color: Color,
    /// Whether the gridline should be dashed
    pub dashed: Option<DashStyle>,
}

impl From<GridLineStyle> for GridLine {
    fn from(value: GridLineStyle) -> Self {
        Self {
            width: value.width,
            color: value.color,
            dashed: value.dashed,
        }
    }
}
