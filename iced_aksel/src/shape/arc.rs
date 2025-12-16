use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use aksel::{Float, PlotPoint, Transform};
use iced_core::Color;

/// A primitive representing a sector of a circle or a ring.
///
/// # Usage
/// ```rust
/// use iced_aksel::shape::Arc;
/// use iced_aksel::Measure;
/// use aksel::PlotPoint;
/// use iced_core::Color;
///
/// let sector = Arc::new(
///     PlotPoint::new(0.0, 0.0),
///     Measure::Screen(50.0),
///     0.0, // Radians
///     1.5  // Radians
/// )
/// .fill(Color::from_rgb(1.0, 0.0, 0.0));
/// ```
#[derive(Debug, Clone)]
pub struct Arc<D> {
    pub center: PlotPoint<D>,
    pub radius: Measure<D>,
    pub inner_radius: Measure<D>,
    pub start_angle: f32, // Radians
    pub end_angle: f32,   // Radians
    pub fill: Option<Color>,
    pub stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Arc<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Arc<D> {
    /// Creates a new `Arc`.
    ///
    /// * `radius`: The outer radius of the arc.
    /// * `start_angle`: Starting angle in **Radians**.
    /// * `end_angle`: Ending angle in **Radians**.
    ///
    /// Note: The shape is invisible by default. You must call `.fill()` or `.stroke()` to render it.
    pub const fn new(
        center: PlotPoint<D>,
        radius: Measure<D>,
        start_angle: f32,
        end_angle: f32,
    ) -> Self {
        Self {
            center,
            radius,
            inner_radius: Measure::Screen(0.0),
            start_angle,
            end_angle,
            fill: None,
            stroke: None,
        }
    }

    /// Sets the inner radius of the arc, creating a donut sector.
    pub const fn inner_radius(mut self, radius: Measure<D>) -> Self {
        self.inner_radius = radius;
        self
    }

    /// Sets the fill color of the arc.
    #[inline]
    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    /// Sets the stroke style (border) of the arc.
    #[inline]
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
        let cx = transform.x_to_screen(&self.center.x);
        let cy = transform.y_to_screen(&self.center.y);

        let outer_r = resolve_isotropic(transform, self.radius);
        let inner_r = resolve_isotropic(transform, self.inner_radius);

        let stroke_info = self.stroke.as_ref().and_then(|stroke| {
            let width = resolve_isotropic(transform, stroke.thickness);
            if width < 0.1 {
                None
            } else {
                Some((stroke, width))
            }
        });

        tess.draw_arc(
            buffer,
            cx,
            cy,
            inner_r,
            outer_r,
            self.start_angle,
            self.end_angle,
            self.fill,
            stroke_info,
        );
    }
}

fn resolve_isotropic<D: Float>(transform: &Transform<D, f32, f32>, measure: Measure<D>) -> f32 {
    match measure {
        Measure::Screen(px) => px,
        Measure::Plot(units) => {
            let px_x = resolve_measure(transform, Measure::Plot(units), true);
            let px_y = resolve_measure(transform, Measure::Plot(units), false);
            px_x.min(px_y)
        }
    }
}

fn resolve_measure<D: Float>(
    transform: &Transform<D, f32, f32>,
    measure: Measure<D>,
    is_x: bool,
) -> f32 {
    match measure {
        Measure::Screen(px) => px,
        Measure::Plot(units) => {
            let zero = if is_x {
                transform.x_to_screen(&D::zero())
            } else {
                transform.y_to_screen(&D::zero())
            };
            let val = if is_x {
                transform.x_to_screen(&units)
            } else {
                transform.y_to_screen(&units)
            };
            (val - zero).abs()
        }
    }
}
