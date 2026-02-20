use crate::{
    Float, Shape, Stroke, plot,
    render::{LineArrows, LineExtensions, Primitive},
};

use aksel::PlotPoint;
use iced_core::Point;

/// A primitive representing a straight segment between two points.
///
/// Supports infinite extensions and arrowheads.
///
/// # Usage
/// ```rust
/// use iced_aksel::shape::Line;
/// use iced_aksel::{Measure, Stroke};
/// use iced::Color;
/// use aksel::PlotPoint;
///
/// let trend = Line::new(
///     PlotPoint::new(0.0, 0.0),
///     PlotPoint::new(100.0, 100.0),
///     Stroke::new(Color::BLACK, Measure::Screen(2.0))
/// )
/// .infinite();
/// ```
#[derive(Debug, Clone)]
pub struct Line<D> {
    /// The start point of the line
    pub p1: PlotPoint<D>,
    /// The end point of the line
    pub p2: PlotPoint<D>,
    /// The stroke style (color, thickness, pattern)
    pub stroke: Stroke<D>,
    /// Whether to extend the line infinitely
    pub extensions: LineExtensions,
    /// Whether to draw arrowheads on the line
    pub arrows: LineArrows,
}

impl<D: Float, R: crate::Renderer> Shape<D, R> for Line<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        let Self {
            p1,
            p2,
            stroke,
            extensions,
            arrows,
        } = self;

        let stroke = stroke.resolve(ctx);
        let start = Point::new(ctx.x_to_screen(&p1.x), ctx.y_to_screen(&p1.y));
        let end = Point::new(ctx.x_to_screen(&p2.x), ctx.y_to_screen(&p2.y));
        let clip_bounds = ctx.clip_bounds();

        ctx.add_primitive(Primitive::Line {
            start,
            end,
            stroke,
            clip_bounds,
            extensions,
            arrows,
        });
    }
}

impl<D: Float> Line<D> {
    /// Creates a new `Line` segment between two points.
    pub const fn new(p1: PlotPoint<D>, p2: PlotPoint<D>, stroke: Stroke<D>) -> Self {
        Self {
            p1,
            p2,
            stroke,
            extensions: LineExtensions {
                start: false,
                end: false,
            },
            arrows: LineArrows {
                start: false,
                end: false,
                size: 3.0,
            },
        }
    }

    /// Sets the stroke style for the line.
    pub const fn stroke(mut self, stroke: Stroke<D>) -> Self {
        self.stroke = stroke;
        self
    }

    /// Extends the line infinitely in the start direction.
    pub const fn extend_start(mut self, enable: bool) -> Self {
        self.extensions.start = enable;
        self
    }

    /// Extends the line infinitely in the end direction.
    pub const fn extend_end(mut self, enable: bool) -> Self {
        self.extensions.end = enable;
        self
    }

    /// Extends the line infinitely in both directions.
    pub const fn infinite(mut self) -> Self {
        self.extensions.start = true;
        self.extensions.end = true;
        self
    }

    /// Adds an arrowhead at the start of the line.
    pub const fn arrow_start(mut self, enable: bool) -> Self {
        self.arrows.start = enable;
        self
    }

    /// Adds an arrowhead at the end of the line.
    pub const fn arrow_end(mut self, enable: bool) -> Self {
        self.arrows.end = enable;
        self
    }

    /// Adds arrowheads to both ends of the line.
    pub const fn arrows(mut self, enable: bool) -> Self {
        self.arrows.start = enable;
        self.arrows.end = enable;
        self
    }

    /// Sets the size multiplier for arrowheads (default is 3.0x line width).
    pub const fn arrow_size(mut self, multiplier: f32) -> Self {
        self.arrows.size = multiplier;
        self
    }
}
