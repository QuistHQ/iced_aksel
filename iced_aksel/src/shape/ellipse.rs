use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use aksel::{Float, PlotPoint, Transform};
use iced_core::Color;

/// A primitive representing an ellipse or circle.
///
/// This shape is defined by a center point and two radii (X and Y).
/// It allows for creating perfect circles (where radii are equal) or stretched ellipses.
///
/// # Usage
///
/// ## 1. Perfect Circle
/// ```rust
/// use iced_aksel::shape::Ellipse;
/// use iced_aksel::Measure;
/// use aksel::PlotPoint;
/// use iced_core::Color;
///
/// let circle = Ellipse::circle(
///     PlotPoint::new(0.0, 0.0),
///     Measure::Screen(10.0)
/// )
/// .fill(Color::RED);
/// ```
///
/// ## 2. Stretched Ellipse
/// ```rust
/// use iced_aksel::shape::Ellipse;
/// use iced_aksel::Measure;
/// use aksel::PlotPoint;
///
/// let oval = Ellipse::new(
///     PlotPoint::new(0.0, 0.0),
///     Measure::Screen(20.0), // Radius X
///     Measure::Screen(10.0)  // Radius Y
/// )
/// .stroke(iced_aksel::Stroke::default());
/// ```
#[derive(Debug, Clone)]
pub struct Ellipse<D> {
    /// The center point of the ellipse
    pub center: PlotPoint<D>,
    /// The horizontal radius
    pub radius_x: Measure<D>,
    /// The vertical radius
    pub radius_y: Measure<D>,
    /// The fill color for the ellipse interior
    pub fill: Option<Color>,
    /// The stroke style for the ellipse border
    pub stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Ellipse<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Ellipse<D> {
    /// Creates a new `Ellipse` defined by a center and separate X and Y radii.
    ///
    /// Note: The shape is invisible by default. You must call `.fill()` or `.stroke()` to render it.
    pub const fn new(center: PlotPoint<D>, radius_x: Measure<D>, radius_y: Measure<D>) -> Self {
        Self {
            center,
            radius_x,
            radius_y,
            fill: None,
            stroke: None,
        }
    }

    /// Creates a perfect `Circle` (an Ellipse where radius X equals radius Y).
    ///
    /// Note: The shape is invisible by default. You must call `.fill()` or `.stroke()` to render it.
    pub const fn circle(center: PlotPoint<D>, radius: Measure<D>) -> Self {
        Self {
            center,
            radius_x: radius,
            radius_y: radius,
            fill: None,
            stroke: None,
        }
    }

    /// Sets the fill color.
    #[inline]
    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    /// Sets the stroke style.
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

        let radius_x_pixels = self.radius_x.resolve_x(transform);
        let radius_y_pixels = self.radius_y.resolve_y(transform);

        let stroke_info = self.stroke.as_ref().map(|s| {
            // For stroke thickness on ellipses, we use X-axis scaling as a convention
            // to ensure a uniform stroke width, avoiding complex anisotropic stroking logic.
            let width_pixels = s.thickness.resolve_x(transform);
            (s, width_pixels)
        });

        tess.draw_circle(
            buffer,
            center_x,
            center_y,
            radius_x_pixels,
            radius_y_pixels,
            self.fill,
            stroke_info,
        );
    }
}
