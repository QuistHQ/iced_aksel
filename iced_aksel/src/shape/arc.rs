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
        let center_x = transform.x_to_screen(&self.center.x);
        let center_y = transform.y_to_screen(&self.center.y);

        // Calculate isotropic radii by taking the minimum scale of X and Y dimensions
        let outer_radius_pixels = {
            let x = self.radius.resolve_x(transform);
            let y = self.radius.resolve_y(transform);
            x.min(y)
        };

        let inner_radius_pixels = {
            let x = self.inner_radius.resolve_x(transform);
            let y = self.inner_radius.resolve_y(transform);
            x.min(y)
        };

        let stroke_info = self.stroke.as_ref().and_then(|stroke| {
            let width_x = stroke.thickness.resolve_x(transform);
            let width_y = stroke.thickness.resolve_y(transform);
            let width_pixels = width_x.min(width_y);

            if width_pixels < 0.1 {
                None
            } else {
                Some((stroke, width_pixels))
            }
        });

        tess.draw_arc(
            buffer,
            center_x,
            center_y,
            inner_radius_pixels,
            outer_radius_pixels,
            self.start_angle,
            self.end_angle,
            self.fill,
            stroke_info,
        );
    }
}
