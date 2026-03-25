use crate::{
    Shape, Stroke,
    interaction::{Area, IntoArea},
    plot,
    render::Primitive,
};
use aksel::{Float, PlotPoint};
use iced_core::Point;

/// A primitive representing a smooth Bézier curve.
///
/// Supports both **Quadratic** (1 control point) and **Cubic** (2 control points) curves.
///
/// # Usage
///
/// ## 1. Quadratic Curve (Simple Arch)
/// ```rust
/// use iced_aksel::shape::Bezier;
/// use iced_aksel::{Measure, Stroke, PlotPoint};
/// use iced::Color;
///
/// let curve = Bezier::quadratic(
///     PlotPoint::new(0.0, 0.0),  // Start
///     PlotPoint::new(5.0, 10.0), // Control Point
///     PlotPoint::new(10.0, 0.0),  // End
///     Stroke::new(Color::BLACK, Measure::Screen(2.0))
/// );
/// ```
///
/// ## 2. Cubic Curve (S-Shape)
/// ```rust
/// use iced_aksel::shape::Bezier;
/// use iced_aksel::{Measure, Stroke, PlotPoint};
/// use iced::Color;
///
/// let curve = Bezier::cubic(
///     PlotPoint::new(0.0, 0.0),  // Start
///     PlotPoint::new(5.0, 10.0), // Control 1
///     PlotPoint::new(5.0, -10.0),// Control 2
///     PlotPoint::new(10.0, 0.0),  // End
///     Stroke::new(Color::BLACK, Measure::Screen(2.0))
/// );
/// ```
#[derive(Debug, Clone)]
pub struct Bezier<D> {
    start: PlotPoint<D>,
    control_1: PlotPoint<D>,
    control_2: Option<PlotPoint<D>>,
    end: PlotPoint<D>,
    stroke: Stroke<D>,
}

impl<D: Float, R: crate::Renderer> Shape<D, R> for Bezier<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        let Self {
            start,
            control_1,
            control_2,
            end,
            stroke,
        } = self;

        let stroke = stroke.resolve(ctx);
        let start = Point::new(ctx.x_to_screen(&start.x), ctx.y_to_screen(&start.y));
        let end = Point::new(ctx.x_to_screen(&end.x), ctx.y_to_screen(&end.y));
        let control_1 = Point::new(ctx.x_to_screen(&control_1.x), ctx.y_to_screen(&control_1.y));
        let control_2 = control_2.map(|p| Point::new(ctx.x_to_screen(&p.x), ctx.y_to_screen(&p.y)));

        ctx.add_primitive(Primitive::BezierCurve {
            start,
            end,
            control_1,
            control_2,
            stroke,
        });
    }
}

impl<D: Float> Bezier<D> {
    /// Creates a **Quadratic** Bézier curve (Start -> Control -> End).
    pub const fn quadratic(
        start: PlotPoint<D>,
        control: PlotPoint<D>,
        end: PlotPoint<D>,
        stroke: Stroke<D>,
    ) -> Self {
        Self {
            start,
            control_1: control,
            control_2: None,
            end,
            stroke,
        }
    }

    /// Creates a **Cubic** Bézier curve (Start -> Control 1 -> Control 2 -> End).
    pub const fn cubic(
        start: PlotPoint<D>,
        control_1: PlotPoint<D>,
        control_2: PlotPoint<D>,
        end: PlotPoint<D>,
        stroke: Stroke<D>,
    ) -> Self {
        Self {
            start,
            control_1,
            control_2: Some(control_2),
            end,
            stroke,
        }
    }

    /// Sets the stroke style for the curve.
    pub const fn stroke(mut self, stroke: Stroke<D>) -> Self {
        self.stroke = stroke;
        self
    }
}

impl<'a, D: Float, Renderer: crate::Renderer> IntoArea<'a, D, Renderer> for &Bezier<D> {
    fn resolve_area(self, ctx: &plot::Context<'a, D, Renderer>) -> Area {
        let p0 = ctx.chart_to_screen(&self.start);
        let p1 = ctx.chart_to_screen(&self.control_1);
        let p3 = ctx.chart_to_screen(&self.end);
        let stroke = self.stroke.resolve(ctx);
        let segments = 30; // 30 segments is plenty for accurate UI hit-testing
        let mut points = Vec::with_capacity(segments + 1);

        if let Some(c2) = self.control_2 {
            // Cubic Bezier Evaluation
            let p2 = ctx.chart_to_screen(&c2);
            for i in 0..=segments {
                let t = i as f32 / segments as f32;
                let t_inv = 1.0 - t;

                let x = t.powi(3).mul_add(
                    p3.x,
                    (3.0 * t_inv * t.powi(2)).mul_add(
                        p2.x,
                        t_inv.powi(3).mul_add(p0.x, 3.0 * t_inv.powi(2) * t * p1.x),
                    ),
                );

                let y = t.powi(3).mul_add(
                    p3.y,
                    (3.0 * t_inv * t.powi(2)).mul_add(
                        p2.y,
                        t_inv.powi(3).mul_add(p0.y, 3.0 * t_inv.powi(2) * t * p1.y),
                    ),
                );

                points.push(Point::new(x, y));
            }
        } else {
            // Quadratic Bezier Evaluation
            for i in 0..=segments {
                let t = i as f32 / segments as f32;
                let t_inv = 1.0 - t;

                let x = t
                    .powi(2)
                    .mul_add(p3.x, t_inv.powi(2).mul_add(p0.x, 2.0 * t_inv * t * p1.x));
                let y = t
                    .powi(2)
                    .mul_add(p3.y, t_inv.powi(2).mul_add(p0.y, 2.0 * t_inv * t * p1.y));

                points.push(Point::new(x, y));
            }
        }

        // MAGIC: We return it as a Polyline, so no new hit-test math is needed!
        Area::Polyline {
            points,
            stroke_width: stroke.thickness.into(),
        }
    }
}
