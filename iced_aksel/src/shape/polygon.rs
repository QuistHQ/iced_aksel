use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use aksel::{Float, PlotPoint, Transform};
use iced_core::{Color, Point};

/// A primitive representing a regular N-sided shape (Hexagon, Octagon, etc.).
///
/// # Usage
/// ```rust
/// use iced_aksel::shape::Polygon;
/// use iced_aksel::Measure;
/// use aksel::PlotPoint;
/// use iced_core::Color;
///
/// let hex = Polygon::new(
///     PlotPoint::new(20.0, 5.0),
///     Measure::Screen(6.0),
///     6
/// )
/// .fill(Color::from_rgb(0.0, 0.0, 1.0));
/// ```
#[derive(Debug, Clone)]
pub struct Polygon<D> {
    center: PlotPoint<D>,
    radius: Measure<D>,
    vertices: u16,
    rotation: f32,
    fill: Option<Color>,
    stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Polygon<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Polygon<D> {
    /// Creates a new regular `Polygon` with a center, radius, and number of vertices.
    ///
    /// Note: The shape is invisible by default. You must call `.fill()` or `.stroke()` to render it.
    pub const fn new(center: PlotPoint<D>, radius: Measure<D>, vertices: u16) -> Self {
        Self {
            center,
            radius,
            vertices,
            rotation: 0.0,
            fill: None,
            stroke: None,
        }
    }

    /// Sets the rotation of the polygon in **degrees**.
    /// `0.0` means the first vertex is at the top (North/-90 degrees).
    pub const fn rotation(mut self, degrees: f32) -> Self {
        self.rotation = degrees;
        self
    }

    /// Sets the fill color of the polygon.
    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    /// Sets the stroke style (border) of the polygon.
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
        if self.vertices < 3 {
            return;
        }

        let center = Point::new(
            transform.x_to_screen(&self.center.x),
            transform.y_to_screen(&self.center.y),
        );

        // For Polygons, we treat radius isotropically (min scale)
        let radius_px = resolve_isotropic(transform, self.radius);

        if radius_px < 0.5 {
            return;
        }

        let stroke_info = self.stroke.as_ref().map(|s| {
            let width = match s.thickness {
                Measure::Screen(w) => w,
                Measure::Plot(w) => resolve_isotropic(transform, Measure::Plot(w)),
            };
            (s, width)
        });

        tess.draw_polygon(
            buffer,
            center,
            radius_px,
            self.vertices,
            self.rotation,
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
