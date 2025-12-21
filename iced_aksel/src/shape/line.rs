use crate::{
    Float, Shape, Stroke, Transform,
    plot::{self},
    render::{MeshBuffer, Tessellator},
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
/// use iced_aksel::Stroke;
/// use aksel::PlotPoint;
///
/// let trend = Line::new(
///     PlotPoint::new(0.0, 0.0),
///     PlotPoint::new(100.0, 100.0)
/// )
/// .stroke(Stroke::default()) // Essential!
/// .infinite();
/// ```
#[derive(Debug, Clone)]
pub struct Line<D> {
    /// The start point of the line
    pub p1: PlotPoint<D>,
    /// The end point of the line
    pub p2: PlotPoint<D>,
    /// The stroke style (color, thickness, pattern)
    pub stroke: Option<Stroke<D>>,
    /// Whether to extend the line infinitely from the start point
    pub extend_start: bool,
    /// Whether to extend the line infinitely from the end point
    pub extend_end: bool,
    /// Whether to draw an arrowhead at the start
    pub arrow_start: bool,
    /// Whether to draw an arrowhead at the end
    pub arrow_end: bool,
    /// Size multiplier for arrowheads (relative to stroke width)
    pub arrow_size: f32,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Line<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Line<D> {
    /// Creates a new `Line` segment between two points.
    ///
    /// Note: The shape is invisible by default. You **must** call `.stroke()` to render it.
    pub const fn new(p1: PlotPoint<D>, p2: PlotPoint<D>) -> Self {
        Self {
            p1,
            p2,
            stroke: None,
            extend_start: false,
            extend_end: false,
            arrow_start: false,
            arrow_end: false,
            arrow_size: 3.0,
        }
    }

    /// Sets the stroke style for the line.
    pub const fn stroke(mut self, stroke: Stroke<D>) -> Self {
        self.stroke = Some(stroke);
        self
    }

    /// Extends the line infinitely in the start direction.
    pub const fn extend_start(mut self, enable: bool) -> Self {
        self.extend_start = enable;
        self
    }

    /// Extends the line infinitely in the end direction.
    pub const fn extend_end(mut self, enable: bool) -> Self {
        self.extend_end = enable;
        self
    }

    /// Extends the line infinitely in both directions.
    pub const fn infinite(mut self) -> Self {
        self.extend_start = true;
        self.extend_end = true;
        self
    }

    /// Adds an arrowhead at the start of the line.
    pub const fn arrow_start(mut self, enable: bool) -> Self {
        self.arrow_start = enable;
        self
    }

    /// Adds an arrowhead at the end of the line.
    pub const fn arrow_end(mut self, enable: bool) -> Self {
        self.arrow_end = enable;
        self
    }

    /// Adds arrowheads to both ends of the line.
    pub const fn arrows(mut self, enable: bool) -> Self {
        self.arrow_start = enable;
        self.arrow_end = enable;
        self
    }

    /// Sets the size multiplier for arrowheads (default is 3.0x line width).
    pub const fn arrow_size(mut self, multiplier: f32) -> Self {
        self.arrow_size = multiplier;
        self
    }

    fn tessellate(
        self,
        transform: &Transform<D, f32, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellator,
    ) {
        let stroke = match self.stroke {
            Some(s) => s,
            None => return, // Invisible if no stroke is defined
        };

        let raw_start_point = Point::new(
            transform.x_to_screen(&self.p1.x),
            transform.y_to_screen(&self.p1.y),
        );

        let raw_end_point = Point::new(
            transform.x_to_screen(&self.p2.x),
            transform.y_to_screen(&self.p2.y),
        );

        // We resolve stroke thickness using the X-axis for consistency.
        let stroke_width_pixels = stroke.thickness.resolve_x(transform);

        let screen_bounds = transform.screen_bounds();
        let clipping_rect = iced_core::Rectangle {
            x: screen_bounds.x,
            y: screen_bounds.y,
            width: screen_bounds.width,
            height: screen_bounds.height,
        };

        tess.draw_line(
            buffer,
            raw_start_point,
            raw_end_point,
            &stroke,
            stroke_width_pixels,
            clipping_rect,
            (self.extend_start, self.extend_end),
            (self.arrow_start, self.arrow_end, self.arrow_size),
        );
    }
}
