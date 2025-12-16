use crate::{
    Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use aksel::{Float, PlotPoint, Transform};
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
/// use iced_aksel::Stroke;
/// use aksel::PlotPoint;
///
/// let curve = Bezier::quadratic(
///     PlotPoint::new(0.0, 0.0),  // Start
///     PlotPoint::new(5.0, 10.0), // Control Point
///     PlotPoint::new(10.0, 0.0)  // End
/// )
/// .stroke(Stroke::default());
/// ```
///
/// ## 2. Cubic Curve (S-Shape)
/// ```rust
/// use iced_aksel::shape::Bezier;
/// use aksel::PlotPoint;
///
/// let curve = Bezier::cubic(
///     PlotPoint::new(0.0, 0.0),  // Start
///     PlotPoint::new(5.0, 10.0), // Control 1
///     PlotPoint::new(5.0, -10.0),// Control 2
///     PlotPoint::new(10.0, 0.0)  // End
/// );
/// ```
#[derive(Debug, Clone)]
pub struct Bezier<D> {
    start: PlotPoint<D>,
    control_1: PlotPoint<D>,
    control_2: Option<PlotPoint<D>>,
    end: PlotPoint<D>,
    stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Bezier<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Bezier<D> {
    /// Creates a **Quadratic** Bézier curve (Start -> Control -> End).
    ///
    /// Note: The shape is invisible by default. You **must** call `.stroke()` to render it.
    pub const fn quadratic(start: PlotPoint<D>, control: PlotPoint<D>, end: PlotPoint<D>) -> Self {
        Self {
            start,
            control_1: control,
            control_2: None,
            end,
            stroke: None,
        }
    }

    /// Creates a **Cubic** Bézier curve (Start -> Control 1 -> Control 2 -> End).
    ///
    /// Note: The shape is invisible by default. You **must** call `.stroke()` to render it.
    pub const fn cubic(
        start: PlotPoint<D>,
        control_1: PlotPoint<D>,
        control_2: PlotPoint<D>,
        end: PlotPoint<D>,
    ) -> Self {
        Self {
            start,
            control_1,
            control_2: Some(control_2),
            end,
            stroke: None,
        }
    }

    /// Sets the stroke style for the curve.
    pub const fn stroke(mut self, stroke: Stroke<D>) -> Self {
        self.stroke = Some(stroke);
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
            None => return, // Invisible
        };

        let start = Point::new(
            transform.x_to_screen(&self.start.x),
            transform.y_to_screen(&self.start.y),
        );
        let end = Point::new(
            transform.x_to_screen(&self.end.x),
            transform.y_to_screen(&self.end.y),
        );
        let ctrl1 = Point::new(
            transform.x_to_screen(&self.control_1.x),
            transform.y_to_screen(&self.control_1.y),
        );

        let ctrl2 = self
            .control_2
            .map(|p| Point::new(transform.x_to_screen(&p.x), transform.y_to_screen(&p.y)));

        // Resolve thickness against X-axis
        let width_pixels = stroke.thickness.resolve_x(transform);

        tess.draw_bezier(buffer, start, ctrl1, ctrl2, end, &stroke, width_pixels);
    }
}
