use iced_core::Pixels;

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
    pub thickness: Pixels,
}

impl Default for GridLine {
    fn default() -> Self {
        Self {
            thickness: Pixels(1.0),
        }
    }
}

impl GridLine {
    pub fn new<I: Into<Pixels>>(thickness: I) -> Self {
        Self {
            thickness: thickness.into(),
        }
    }
}
