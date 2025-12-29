use iced_core::Pixels;

/// Configuration for a single grid line on the chart.
///
/// Grid lines are drawn perpendicular to the axis at each tick position.
#[derive(Debug, Clone, Copy)]
pub struct GridLine {
    /// The thickness of the grid line in pixels.
    pub thickness: Pixels,
    /// Whether the grid line should be dashed.
    pub dashed: bool,
}

impl Default for GridLine {
    fn default() -> Self {
        Self {
            thickness: Pixels(1.0),
            dashed: false,
        }
    }
}

impl GridLine {
    /// Creates a new solid grid line with the specified thickness.
    pub fn new<I: Into<Pixels>>(thickness: I) -> Self {
        Self {
            thickness: thickness.into(),
            dashed: false,
        }
    }

    /// Sets whether the grid line should be dashed.
    pub fn with_dashed(mut self, dashed: bool) -> Self {
        self.dashed = dashed;
        self
    }
}
