use crate::{Shape, Stroke, plot, render::Primitive};
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
