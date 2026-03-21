use crate::{
    Measure, Shape, Stroke,
    interaction::{Area, IntoArea},
    plot,
    radii::{Radius, ResolvedRadius},
    render::Primitive,
};
use aksel::{Float, PlotPoint};
use iced_core::{Color, Point, Radians};

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
    radius: Radius<Measure<D>>,
    vertices: u16,
    rotation: Radians,
    fill: Option<Color>,
    stroke: Option<Stroke<D>>,
}

impl<D: Float, R: crate::Renderer> Shape<D, R> for Polygon<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        let Self {
            center,
            radius,
            vertices,
            rotation,
            fill,
            stroke,
        } = self;

        if vertices < 3 {
            return;
        }

        let center = Point::new(ctx.x_to_screen(&center.x), ctx.y_to_screen(&center.y));

        // For Polygons, we treat radius isotropically. We take the minimum scale
        // to ensure the polygon is not distorted if axes have different scales.
        let Some(radius) = radius.resolve_isotropic(ctx) else {
            return; // No radius, no rendering!
        };
        let stroke = stroke.map(|s| s.resolve(ctx));

        ctx.add_primitive(Primitive::Polygon {
            center,
            radius,
            vertices,
            rotation,
            fill,
            stroke,
        });
    }
}

impl<D: Float> Polygon<D> {
    /// Creates a new regular `Polygon` with a center, radius, and number of vertices.
    ///
    /// Note: The shape is invisible by default. You must call `.fill()` or `.stroke()` to render it.
    pub fn new(center: PlotPoint<D>, radius: impl Into<Radius<Measure<D>>>, vertices: u16) -> Self {
        Self {
            center,
            radius: radius.into(),
            vertices,
            rotation: Radians(0.0),
            fill: None,
            stroke: None,
        }
    }

    /// Sets the rotation of the polygon in **radians**.
    /// `0.0` means the first vertex is at the top (North/-90 degrees).
    pub fn rotation(mut self, radians: impl Into<Radians>) -> Self {
        self.rotation = radians.into();
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
}

impl<'a, D: Float, Renderer: crate::Renderer> IntoArea<'a, D, Renderer> for &Polygon<D> {
    fn resolve_area(self, ctx: &plot::Context<'a, D, Renderer>) -> Area {
        let center = ctx.chart_to_screen(&self.center);
        Area::RegularPolygon {
            center: Point::new(center.x, center.y),
            radius: self
                .radius
                .resolve_isotropic(ctx)
                .unwrap_or(ResolvedRadius::ZERO),
            vertices: self.vertices,
            rotation: self.rotation,
        }
    }
}
