use crate::{
    Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use aksel::{Float, PlotPoint, Transform};
use iced_core::Point;

/// A primitive representing a connected series of line segments.
///
/// Optimized for drawing data series and paths.
///
/// # Usage
/// ```rust
/// use iced_aksel::shape::Polyline;
/// use iced_aksel::Stroke;
/// use aksel::PlotPoint;
///
/// let data = vec![PlotPoint::new(0.0, 0.0), PlotPoint::new(1.0, 5.0)];
/// let series = Polyline::new(data)
///     .stroke(Stroke::default()); // Essential!
/// ```
#[derive(Debug, Clone)]
pub struct Polyline<D> {
    pub points: Vec<PlotPoint<D>>,
    pub stroke: Option<Stroke<D>>,
    pub extend_start: bool,
    pub extend_end: bool,
    pub arrow_start: bool,
    pub arrow_end: bool,
    pub arrow_size: f32,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Polyline<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Polyline<D> {
    /// Creates a new `Polyline` from a vector of points.
    ///
    /// Note: The shape is invisible by default. You **must** call `.stroke()` to render it.
    pub const fn new(points: Vec<PlotPoint<D>>) -> Self {
        Self {
            points,
            stroke: None,
            extend_start: false,
            extend_end: false,
            arrow_start: false,
            arrow_end: false,
            arrow_size: 3.0,
        }
    }

    /// Sets the stroke style for the polyline.
    pub const fn stroke(mut self, stroke: Stroke<D>) -> Self {
        self.stroke = Some(stroke);
        self
    }

    /// Extends the first segment of the polyline infinitely backwards.
    pub const fn extend_start(mut self, enable: bool) -> Self {
        self.extend_start = enable;
        self
    }

    /// Extends the last segment of the polyline infinitely forwards.
    pub const fn extend_end(mut self, enable: bool) -> Self {
        self.extend_end = enable;
        self
    }

    /// Adds an arrowhead at the start of the polyline.
    pub const fn arrow_start(mut self, enable: bool) -> Self {
        self.arrow_start = enable;
        self
    }

    /// Adds an arrowhead at the end of the polyline.
    pub const fn arrow_end(mut self, enable: bool) -> Self {
        self.arrow_end = enable;
        self
    }

    /// Sets the size multiplier for arrowheads.
    pub const fn arrow_size(mut self, size: f32) -> Self {
        self.arrow_size = size;
        self
    }

    fn tessellate(
        self,
        transform: &Transform<D, f32, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellator,
    ) {
        if self.points.len() < 2 {
            return;
        }

        let stroke = match self.stroke {
            Some(s) => s,
            None => return, // Invisible
        };

        // Resolve stroke thickness against X axis
        let width_pixels = stroke.thickness.resolve_x(transform);

        let screen_bounds = transform.screen_bounds();
        let clipping_rect = iced_core::Rectangle {
            x: screen_bounds.x,
            y: screen_bounds.y,
            width: screen_bounds.width,
            height: screen_bounds.height,
        };

        let screen_points_iterator = self
            .points
            .iter()
            .map(|p| Point::new(transform.x_to_screen(&p.x), transform.y_to_screen(&p.y)));

        tess.draw_polyline(
            buffer,
            screen_points_iterator,
            &stroke,
            width_pixels,
            clipping_rect,
            (self.extend_start, self.extend_end),
            (self.arrow_start, self.arrow_end, self.arrow_size),
        );
    }
}
