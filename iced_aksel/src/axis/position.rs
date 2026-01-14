/// The orientation of an axis (horizontal or vertical).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// Axis runs left-to-right across the chart.
    Horizontal,
    /// Axis runs top-to-bottom along the chart.
    Vertical,
}

/// The position where an axis is rendered on the chart.
///
/// # Example
///
/// ```rust
/// use iced_aksel::{Axis, axis::Position, scale::Linear};
///
/// // Create an axis at the bottom of the chart
/// let x_axis: Axis<f64>  = Axis::new(Linear::new(0.0, 100.0), Position::Bottom);
///
/// // Create an axis at the left side of the chart
/// let y_axis: Axis<f64> = Axis::new(Linear::new(0.0, 50.0), Position::Left);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    /// Axis at the top edge of the chart (horizontal).
    Top,
    /// Axis at the bottom edge of the chart (horizontal).
    Bottom,
    /// Axis at the left edge of the chart (vertical).
    Left,
    /// Axis at the right edge of the chart (vertical).
    Right,
}

impl From<Position> for Orientation {
    fn from(value: Position) -> Self {
        match value {
            Position::Top | Position::Bottom => Self::Horizontal,
            Position::Left | Position::Right => Self::Vertical,
        }
    }
}

impl<'a> From<&'a Position> for Orientation {
    fn from(value: &'a Position) -> Self {
        match value {
            Position::Top | Position::Bottom => Self::Horizontal,
            Position::Left | Position::Right => Self::Vertical,
        }
    }
}
