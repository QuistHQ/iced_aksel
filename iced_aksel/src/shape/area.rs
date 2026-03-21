use crate::{
    Shape, Stroke,
    interaction::{self, IntoArea},
    plot::{self},
    render::Primitive,
};
use aksel::{Float, PlotPoint};
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
///     PlotPoint::new(1.0, 1.0),
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

impl<D: Float, R: crate::Renderer> Shape<D, R> for Area<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        let Self {
            points,
            fill,
            stroke,
        } = self;

        if points.len() < 3 {
            return;
        }

        let points: Vec<Point> = points
            .into_iter()
            .map(|p| Point::new(ctx.x_to_screen(&p.x), ctx.y_to_screen(&p.y)))
            .collect();

        let stroke = stroke.map(|s| s.resolve(ctx));

        ctx.add_primitive(Primitive::Area {
            points,
            fill,
            stroke,
        });
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
}

impl<'a, D: Float, Renderer: crate::Renderer> IntoArea<'a, D, Renderer> for &Area<D> {
    fn resolve_area(self, ctx: &plot::Context<'a, D, Renderer>) -> interaction::Area {
        interaction::Area::Polygon {
            points: self
                .points
                .iter()
                .map(|p| {
                    let sp = ctx.chart_to_screen(p);
                    Point::new(sp.x, sp.y)
                })
                .collect(),
        }
    }
}
