use crate::axis::GridLine;

use super::Orientation;

use aksel::{Float, Tick};
use iced_core::{Pixels, Rectangle};

pub mod label;

pub use label::*;

pub struct TickResult {
    pub tick_line: Option<TickLine>,
    pub grid_line: Option<GridLine>,
    pub label: Option<Label>,
}

impl Default for TickResult {
    fn default() -> Self {
        Self {
            tick_line: Some(TickLine::default()),
            grid_line: Some(GridLine::default()),
            label: None,
        }
    }
}

impl TickResult {
    /// Creates a new empty [TickResult]
    pub const fn new() -> Self {
        Self {
            tick_line: None,
            grid_line: None,
            label: None,
        }
    }

    /// Creates a new empty [TickResult] with a label
    pub fn with_label<L: Into<Label>>(label: L) -> Self {
        Self {
            label: Some(label.into()),
            ..Self::new()
        }
    }

    /// Creates a new [TickResult] with a tick-line
    pub fn with_tick_line(line: TickLine) -> Self {
        Self {
            tick_line: Some(line),
            ..Self::new()
        }
    }

    /// Creates a new [TickResult] with a grid-line
    pub fn with_grid_line(line: GridLine) -> Self {
        Self {
            grid_line: Some(line),
            ..Self::new()
        }
    }

    /// Adds a label to the [TickResult]
    pub fn label<L: Into<Label>>(mut self, label: L) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Adds a tick-line to the [TickResult]
    pub const fn tick_line(mut self, line: TickLine) -> Self {
        self.tick_line = Some(line);
        self
    }

    /// Adds a grid-line to the [TickResult]
    pub const fn grid_line(mut self, line: GridLine) -> Self {
        self.grid_line = Some(line);
        self
    }
}

/// Defines the visual styling and content of a single tick mark on an Axis.
///
/// # The Tick Rendering Pipeline
///
/// Understanding `TickLine` requires understanding how `aksel` processes axes.
/// The relationship flows as follows:
///
/// 1.  **Generation (`Scale`):** The [`Scale`](crate::Scale) (e.g., `Linear`, `Log`) calculates logical positions
///     and importance levels for potential ticks, creating [`Tick`] objects.
/// 2.  **Contextualization (`Axis`):** The Axis wraps each logical [`Tick`] into a [`TickLabelContext`](crate::axis::TickLabelContext).
///     This provides context like the axis bounds, domain, and orientation.
/// 3.  **Styling (User Logic):** The user provides a closure via [`Axis::with_tick_renderer`](crate::Axis::with_tick_renderer).
///     This closure receives the context and decides **if** and **how** that tick should be drawn
///     by returning an `Option<TickLine>`.
///
/// This separation allows you to have completely dynamic styling—for example, making every
/// major tick thick and red, while making minor ticks thin and grey, or hiding specific ticks entirely.
///
/// [Image of chart axis anatomy showing major and minor ticks]
///
/// # Example
///
/// This example demonstrates how to configure an axis to render major ticks with labels
/// and thick lines, while rendering minor ticks as smaller lines without labels.
///
/// ```rust
/// use iced_aksel::{Axis, axis::TickLine, scale::Linear};
/// use iced::Pixels;
///
/// let axis = Axis::new(Linear::new(0.0, 100.0), iced_aksel::axis::Position::Bottom)
///     .with_tick_renderer(|ctx| {
///         match ctx.tick.level {
///             // Major Tick (Level 0): Thicker, longer, and has a text label
///             0 => Some(TickLine {
///                 thickness: Pixels(2.0),
///                 length: Pixels(10.0),
///                 // We use the context's value to format the text
///                 label: Some(iced_aksel::axis::Label {
///                     content: format!("{:.1}", ctx.tick.value),
///                     ..Default::default()
///                 }),
///             }),
///             // Minor Tick (Level 1): Thinner, shorter, no label
///             1 => Some(TickLine {
///                 thickness: Pixels(1.0),
///                 length: Pixels(5.0),
///                 label: None,
///             }),
///             // Any other importance level: Do not draw (return None)
///             _ => None,
///         }
///     });
/// ```
#[derive(Debug, Clone)]
pub struct TickLine {
    /// The visual thickness (stroke width) of the tick line.
    pub thickness: Pixels,

    /// The length of the tick line perpendicular to the axis.
    pub length: Pixels,
}

impl Default for TickLine {
    #[inline(always)]
    fn default() -> Self {
        Self {
            thickness: Pixels(1.0),
            length: Pixels(5.0),
        }
    }
}

/// This is all the information you would need to define a `TickLine` properly.
#[derive(Debug, Clone, Copy)]
pub struct TickContext<D> {
    pub tick: Tick<D>,
    pub normalized_position: f32,
    pub axis_bounds: Rectangle,
    pub scale_domain: (D, D),
    pub orientation: Orientation,
}

impl<D: Float> TickContext<D> {
    pub const fn axis_span(&self) -> f32 {
        match self.orientation {
            Orientation::Horizontal => self.axis_bounds.width,
            Orientation::Vertical => self.axis_bounds.height,
        }
    }

    pub fn scale_span(&self) -> D {
        let (min, max) = self.scale_domain;
        min.abs_sub(max)
    }
}
