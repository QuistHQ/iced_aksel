use crate::{
    Shape, Stroke,
    interaction::{Area, IntoArea},
    plot,
    render::Primitive,
};
use aksel::{Float, PlotPoint, ScreenPoint};
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

impl<'a, D: Float, Renderer: crate::Renderer> IntoArea<'a, D, Renderer> for &Spline<D> {
    fn resolve_area(self, ctx: &plot::Context<'a, D, Renderer>) -> Area {
        let stroke = self.stroke.resolve(ctx);

        if self.points.len() < 2 {
            return Area::Polyline {
                points: vec![],
                stroke_width: 0.0.into(),
            };
        }

        // Map to screen points
        let sp: Vec<ScreenPoint> = self.points.iter().map(|p| ctx.chart_to_screen(p)).collect();

        let mut flattened = Vec::new();
        let segments_per_curve = 15;

        // Catmull-Rom Spline flattening
        for i in 0..sp.len().saturating_sub(1) {
            let p0 = if i == 0 { sp[0] } else { sp[i - 1] };
            let p1 = sp[i];
            let p2 = sp[i + 1];
            let p3 = if i + 2 < sp.len() { sp[i + 2] } else { p2 };

            for step in 0..=segments_per_curve {
                let t = step as f32 / segments_per_curve as f32;
                let t2 = t * t;
                let t3 = t2 * t;

                // Catmull-Rom math incorporating the user's tension parameter
                let alpha = 1.0 - self.tension;

                let x = 0.5
                    * ((3.0f32.mul_add(-p2.x, 3.0f32.mul_add(p1.x, -p0.x)) + p3.x) * t3).mul_add(
                        alpha,
                        ((4.0f32.mul_add(p2.x, 2.0f32.mul_add(p0.x, -(5.0 * p1.x))) - p3.x) * t2)
                            .mul_add(alpha, 2.0f32.mul_add(p1.x, (-p0.x + p2.x) * t * alpha)),
                    );

                let y = 0.5
                    * ((3.0f32.mul_add(-p2.y, 3.0f32.mul_add(p1.y, -p0.y)) + p3.y) * t3).mul_add(
                        alpha,
                        ((4.0f32.mul_add(p2.y, 2.0f32.mul_add(p0.y, -(5.0 * p1.y))) - p3.y) * t2)
                            .mul_add(alpha, 2.0f32.mul_add(p1.y, (-p0.y + p2.y) * t * alpha)),
                    );

                flattened.push(Point::new(x, y));
            }
        }

        Area::Polyline {
            points: flattened,
            stroke_width: stroke.thickness.into(),
        }
    }
}
