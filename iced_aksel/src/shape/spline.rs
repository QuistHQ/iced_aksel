use crate::{Shape, Stroke, plot, render::Primitive};
use aksel::{Float, PlotPoint};
use iced_core::Point;

/// A primitive representing a smooth curve passing through a list of points.
///
/// Unlike `Bezier`, which requires manual control points, a `Spline` automatically
/// calculates the curve to ensure it passes smoothly through every data point provided.
///
/// # Usage
/// ```rust
/// use iced_aksel::shape::Spline;
/// use iced_aksel::{Measure, Stroke};
/// use iced::Color;
/// use aksel::PlotPoint;
///
/// let data = vec![
///     PlotPoint::new(0.0, 0.0),
///     PlotPoint::new(10.0, 15.0),
///     PlotPoint::new(20.0, 5.0)
/// ];
///
/// // Standard smooth curve
/// let curve = Spline::new(data, Stroke::new(Color::BLACK, Measure::Screen(2.0)));
/// ```
#[derive(Debug, Clone)]
pub struct Spline<D> {
    /// The points that the curve passes through
    pub points: Vec<PlotPoint<D>>,
    /// The stroke style (color, thickness, pattern)
    pub stroke: Stroke<D>,
    /// The tension of the curve (0.0 = smooth, 1.0 = straight lines)
    pub tension: f32,
}

impl<D: Float, R: crate::Renderer> Shape<D, R> for Spline<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        let Self {
            points,
            stroke,
            tension,
        } = self;

        if points.len() < 2 {
            return;
        }

        let stroke = stroke.resolve(ctx);
        let points = points
            .into_iter()
            .map(|p| Point::new(ctx.x_to_screen(&p.x), ctx.y_to_screen(&p.y)))
            .collect();

        ctx.add_primitive(Primitive::Spline {
            points,
            stroke,
            tension,
        })
    }
}

impl<D: Float> Spline<D> {
    /// Creates a new `Spline` from a vector of points.
    pub const fn new(points: Vec<PlotPoint<D>>, stroke: Stroke<D>) -> Self {
        Self {
            points,
            stroke,
            tension: 0.0, // Default to standard Catmull-Rom smoothing
        }
    }

    /// Sets the stroke style for the spline.
    pub const fn stroke(mut self, stroke: Stroke<D>) -> Self {
        self.stroke = stroke;
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
}
impl<D: Float> From<&Spline<D>> for crate::interaction::Area<D> {
    fn from(value: &Spline<D>) -> Self {
        crate::interaction::Area::Spline {
            points: value.points.clone(),
            width: value.stroke.thickness,
            tension: value.tension,
        }
    }
}
