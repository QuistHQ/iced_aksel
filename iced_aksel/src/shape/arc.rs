use crate::{Measure, Shape, Stroke, plot, radii::Radius, render::Primitive};

use aksel::{Float, PlotPoint};
use iced_core::{Color, Point, Radians};

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
    /// The center point of the arc
    pub center: PlotPoint<D>,
    /// The outer radius of the arc
    pub radius: Radius<Measure<D>>,
    /// The inner radius of the arc (0 for a pie slice)
    pub inner_radius: Radius<Measure<D>>,
    /// The starting angle in radians
    pub start_angle: Radians,
    /// The ending angle in radians
    pub end_angle: Radians,
    /// The fill color for the arc interior
    pub fill: Option<Color>,
    /// The stroke style for the arc border
    pub stroke: Option<Stroke<D>>,
}

impl<D: Float, R: crate::Renderer> Shape<D, R> for Arc<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        let Self {
            center,
            radius,
            inner_radius,
            start_angle,
            end_angle,
            fill,
            stroke,
        } = self;

        // If the outer radius doesn't resolve, we don't render anything
        let Some(radius_outer) = radius.resolve_isotropic(ctx) else {
            return;
        };

        let radius_inner = inner_radius.resolve_isotropic(ctx);
        let center = Point::new(ctx.x_to_screen(&center.x), ctx.y_to_screen(&center.y));
        let stroke = stroke.map(|s| s.resolve(ctx));

        ctx.add_primitive(Primitive::Arc {
            center,
            radius_inner,
            radius_outer,
            start_angle,
            end_angle,
            stroke,
            fill,
        });
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
    pub fn new(
        center: PlotPoint<D>,
        radius: impl Into<Radius<Measure<D>>>,
        start_angle: impl Into<Radians>,
        end_angle: impl Into<Radians>,
    ) -> Self {
        Self {
            center,
            radius: radius.into(),
            inner_radius: Radius(Measure::Screen(0.0)),
            start_angle: start_angle.into(),
            end_angle: end_angle.into(),
            fill: None,
            stroke: None,
        }
    }

    /// Sets the inner radius of the arc, creating a donut sector.
    pub fn inner_radius(mut self, radius: impl Into<Radius<Measure<D>>>) -> Self {
        self.inner_radius = radius.into();
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
}

impl<D: Float> From<&Arc<D>> for crate::interaction::Area<D> {
    fn from(value: &Arc<D>) -> Self {
        crate::interaction::Area::Arc {
            center: value.center,
            radius_outer: value.radius.0,
            radius_inner: value.inner_radius.0,
            start_angle_rads: value.start_angle.0,
            end_angle_rads: value.end_angle.0,
        }
    }
}
