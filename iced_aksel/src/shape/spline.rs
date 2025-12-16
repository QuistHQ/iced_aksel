use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use aksel::{Float, PlotPoint, Transform};
use iced_core::Point;

/// A primitive representing a smooth curve passing through a list of points.
///
/// Unlike `Bezier`, which requires manual control points, a `Spline` automatically
/// calculates the curve to ensure it passes smoothly through every data point provided.
///
/// # Usage
/// ```rust
/// use iced_aksel::shape::Spline;
/// use iced_aksel::Stroke;
/// use aksel::PlotPoint;
///
/// let data = vec![
///     PlotPoint::new(0.0, 0.0),
///     PlotPoint::new(10.0, 15.0),
///     PlotPoint::new(20.0, 5.0)
/// ];
///
/// // Standard smooth curve
/// let curve = Spline::new(data)
///     .stroke(Stroke::default());
/// ```
#[derive(Debug, Clone)]
pub struct Spline<D> {
    pub points: Vec<PlotPoint<D>>,
    pub stroke: Option<Stroke<D>>,
    pub tension: f32,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Spline<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Spline<D> {
    /// Creates a new `Spline` from a vector of points.
    ///
    /// Note: The shape is invisible by default. You **must** call `.stroke()` to render it.
    pub const fn new(points: Vec<PlotPoint<D>>) -> Self {
        Self {
            points,
            stroke: None,
            tension: 0.0, // Default to standard Catmull-Rom smoothing
        }
    }

    /// Sets the stroke style for the spline.
    pub const fn stroke(mut self, stroke: Stroke<D>) -> Self {
        self.stroke = Some(stroke);
        self
    }

    /// Sets the tension of the curve.
    ///
    /// * `0.0`: Standard smooth curve (Catmull-Rom).
    /// * `1.0`: No smoothing (Straight lines).
    /// * Values between `0.0` and `1.0` blend between smooth and straight.
    pub const fn tension(mut self, tension: f32) -> Self {
        self.tension = tension;
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

        // Resolve thickness against X-axis
        let width_pixels = stroke.thickness.resolve_x(transform);

        let screen_points_iterator = self
            .points
            .iter()
            .map(|p| Point::new(transform.x_to_screen(&p.x), transform.y_to_screen(&p.y)));

        tess.draw_spline(
            buffer,
            screen_points_iterator,
            &stroke,
            width_pixels,
            self.tension,
        );
    }
}
