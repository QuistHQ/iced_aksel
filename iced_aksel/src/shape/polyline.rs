use crate::{
    Shape, Stroke, plot,
    render::{LineArrows, LineExtensions, Primitive},
};
use aksel::{Float, PlotPoint};
use iced_core::Point;

/// A primitive representing a connected series of line segments.
///
/// Optimized for drawing data series and paths.
///
/// # Usage
/// ```rust
/// use iced_aksel::shape::Polyline;
/// use iced_aksel::{Measure, Stroke};
/// use iced::Color;
/// use aksel::PlotPoint;
///
/// let data = vec![PlotPoint::new(0.0, 0.0), PlotPoint::new(1.0, 5.0)];
/// let series = Polyline::new(data, Stroke::new(Color::BLACK, Measure::Screen(2.0)));
/// ```
#[derive(Debug, Clone)]
pub struct Polyline<D> {
    /// The points that define the polyline path
    pub points: Vec<PlotPoint<D>>,
    /// The stroke style (color, thickness, pattern)
    pub stroke: Stroke<D>,
    /// Whether to extend the line infinitely
    pub extensions: LineExtensions,
    /// Whether to draw arrowheads on the line
    pub arrows: LineArrows,
}

impl<D: Float, R: crate::Renderer> Shape<D, R> for Polyline<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        let Self {
            points,
            stroke,
            extensions,
            arrows,
        } = self;

        if points.len() < 2 {
            return;
        }

        let stroke = stroke.resolve(ctx);
        let screen_bounds = ctx.screen_bounds();
        let clip_bounds = iced_core::Rectangle {
            x: screen_bounds.x,
            y: screen_bounds.y,
            width: screen_bounds.width,
            height: screen_bounds.height,
        };

        let points = points
            .into_iter()
            .map(|p| Point::new(ctx.x_to_screen(&p.x), ctx.y_to_screen(&p.y)))
            .collect();

        ctx.add_primitive(Primitive::PolyLine {
            points,
            stroke,
            clip_bounds,
            extensions,
            arrows,
        });
    }
}

impl<D: Float> Polyline<D> {
    /// Creates a new `Polyline` from a vector of points.
    ///
    /// Note: The shape is invisible by default. You **must** call `.stroke()` to render it.
    pub const fn new(points: Vec<PlotPoint<D>>, stroke: Stroke<D>) -> Self {
        Self {
            points,
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

    /// Sets the stroke style for the polyline.
    pub const fn stroke(mut self, stroke: Stroke<D>) -> Self {
        self.stroke = stroke;
        self
    }

    /// Extends the first segment of the polyline infinitely backwards.
    pub const fn extend_start(mut self, enable: bool) -> Self {
        self.extensions.start = enable;
        self
    }

    /// Extends the last segment of the polyline infinitely forwards.
    pub const fn extend_end(mut self, enable: bool) -> Self {
        self.extensions.end = enable;
        self
    }

    /// Adds an arrowhead at the start of the polyline.
    pub const fn arrow_start(mut self, enable: bool) -> Self {
        self.arrows.start = enable;
        self
    }

    /// Adds an arrowhead at the end of the polyline.
    pub const fn arrow_end(mut self, enable: bool) -> Self {
        self.arrows.end = enable;
        self
    }

    /// Sets the size multiplier for arrowheads.
    pub const fn arrow_size(mut self, size: f32) -> Self {
        self.arrows.size = size;
        self
    }
}
