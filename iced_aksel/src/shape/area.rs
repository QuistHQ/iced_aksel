use crate::{
    Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use aksel::{Float, PlotPoint, Transform};
use iced_core::{Color, Point};

/// A primitive representing an arbitrary filled area defined by a list of points.
///
/// Automatically handles convex and concave polygons.
///
/// # Usage
/// ```rust
/// use iced_aksel::shape::Area;
/// use aksel::PlotPoint;
/// use iced_core::Color;
///
/// let area = Area::new(vec![
///     PlotPoint::new(0.0, 0.0),
///     PlotPoint::new(1.0, 1.0)
///     PlotPoint::new(2.0, 2.0)
/// ])
/// .fill(Color::from_rgb(0.0, 0.0, 1.0));
/// ```
#[derive(Debug, Clone)]
pub struct Area<D> {
    points: Vec<PlotPoint<D>>,
    fill: Option<Color>,
    stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Area<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Area<D> {
    /// Creates a new `Area` from a list of points.
    ///
    /// Note: The shape is invisible by default. You must call `.fill()` or `.stroke()` to render it.
    pub const fn new(points: Vec<PlotPoint<D>>) -> Self {
        Self {
            points,
            fill: None,
            stroke: None,
        }
    }

    /// Sets the fill color of the area.
    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    /// Sets the stroke style (border) of the area.
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
        if self.points.len() < 3 {
            return;
        }

        let screen_points: Vec<Point> = self
            .points
            .iter()
            .map(|p| Point::new(transform.x_to_screen(&p.x), transform.y_to_screen(&p.y)))
            .collect();

        let stroke_info = self.stroke.as_ref().map(|s| {
            let width_pixels = s.thickness.resolve_x(transform);
            (s, width_pixels)
        });

        tess.draw_zone(buffer, &screen_points, self.fill, stroke_info);
    }
}
