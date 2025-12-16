use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use aksel::{Float, PlotPoint, Transform};
use iced_core::Color;

/// A primitive representing a circle or ellipse.
///
/// Can be defined as a perfect circle (one radius) or an ellipse (separate X/Y radii).
///
/// # Usage
/// ```rust
/// use iced_aksel::shape::Circle;
/// use iced_aksel::Measure;
/// use aksel::PlotPoint;
/// use iced_core::Color;
///
/// // Standard Circle
/// let circle = Circle::new(
///     PlotPoint::new(0.0, 0.0),
///     Measure::Screen(5.0)
/// ).fill(Color::RED);
///
/// // Ellipse (Wide and Short)
/// let ellipse = Circle::ellipse(
///     PlotPoint::new(0.0, 0.0),
///     Measure::Screen(10.0), // Radius X
///     Measure::Screen(5.0)   // Radius Y
/// ).fill(Color::BLUE);
/// ```
#[derive(Debug, Clone)]
pub struct Ellipse<D> {
    pub center: PlotPoint<D>,
    pub radius_x: Measure<D>,
    pub radius_y: Measure<D>,
    pub fill: Option<Color>,
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
    /// Creates a new perfect `Circle` defined by a center and a single radius.
    ///
    /// Note: The shape is invisible by default. You must call `.fill()` or `.stroke()` to render it.
    pub const fn new(center: PlotPoint<D>, radius: Measure<D>) -> Self {
        Self {
            center,
            radius_x: radius,
            radius_y: radius,
            fill: None,
            stroke: None,
        }
    }

    /// Creates a new `Ellipse` defined by a center and separate X and Y radii.
    ///
    /// Note: The shape is invisible by default. You must call `.fill()` or `.stroke()` to render it.
    pub const fn ellipse(center: PlotPoint<D>, radius_x: Measure<D>, radius_y: Measure<D>) -> Self {
        Self {
            center,
            radius_x,
            radius_y,
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
        let cx = transform.x_to_screen(&self.center.x);
        let cy = transform.y_to_screen(&self.center.y);

        let rx_px = self.resolve_measure(transform, self.radius_x, true);
        let ry_px = self.resolve_measure(transform, self.radius_y, false);

        let stroke_info = self.stroke.as_ref().map(|s| {
            let width = match s.thickness {
                Measure::Screen(w) => w,
                // For stroke thickness on ellipses, we default to isotropic averaging or X-axis
                // to avoid complexity. Using X-axis scale for simplicity.
                Measure::Plot(w) => self.resolve_measure(transform, Measure::Plot(w), true),
            };
            (s, width)
        });

        tess.draw_circle(buffer, cx, cy, rx_px, ry_px, self.fill, stroke_info);
    }

    fn resolve_measure(
        &self,
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
}
