use crate::{
    Measure, Shape, Stroke,
    interaction::{Area, IntoArea},
    plot,
    render::Primitive,
};
use aksel::{Float, PlotPoint};
use iced_core::{Color, Point};

/// A primitive representing a three-sided polygon.
///
/// Can be defined by arbitrary vertices or as a centered shape with specific width/height.
///
/// # Usage
/// ```rust
/// use iced_aksel::shape::Triangle;
/// use iced_aksel::Measure;
/// use aksel::PlotPoint;
/// use iced_core::Color;
///
/// // A wide, short triangle (Directional Marker)
/// let marker = Triangle::centered(
///     PlotPoint::new(10.0, 10.0),
///     Measure::Screen(20.0), // Width
///     Measure::Screen(10.0)  // Height
/// )
/// .fill(Color::WHITE);
/// ```
#[derive(Debug, Clone)]
enum Geometry<D> {
    /// A triangle defined by three distinct points in plot space.
    Vertices([PlotPoint<D>; 3]),
    /// A triangle defined by a center and dimensions (Isosceles).
    /// Vertices are calculated relative to the bounding box.
    Centered {
        center: PlotPoint<D>,
        width: Measure<D>,
        height: Measure<D>,
    },
}

/// A triangular shape that can be filled and/or stroked.
#[derive(Debug, Clone)]
pub struct Triangle<D> {
    geometry: Geometry<D>,
    /// The fill color for the triangle interior
    pub fill: Option<Color>,
    /// The stroke style for the triangle border
    pub stroke: Option<Stroke<D>>,
}

impl<D: Float, R: crate::Renderer> Shape<D, R> for Triangle<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        let Self {
            geometry,
            fill,
            stroke,
        } = self;

        let points = match geometry {
            Geometry::Vertices(pts) => (
                Point::new(ctx.x_to_screen(&pts[0].x), ctx.y_to_screen(&pts[0].y)),
                Point::new(ctx.x_to_screen(&pts[1].x), ctx.y_to_screen(&pts[1].y)),
                Point::new(ctx.x_to_screen(&pts[2].x), ctx.y_to_screen(&pts[2].y)),
            ),
            Geometry::Centered {
                center,
                width,
                height,
            } => {
                let center_x = ctx.x_to_screen(&center.x);
                let center_y = ctx.y_to_screen(&center.y);

                let width = width.resolve_x(ctx);
                let height = height.resolve_y(ctx);

                let half_width = width / 2.0;
                let half_height = height / 2.0;

                // Points for an Upward facing triangle inside the bounding box
                (
                    // Top Center
                    Point::new(center_x, center_y - half_height),
                    // Bottom Right
                    Point::new(center_x + half_width, center_y + half_height),
                    // Bottom Left
                    Point::new(center_x - half_width, center_y + half_height),
                )
            }
        };

        let stroke = stroke.map(|s| s.resolve(ctx));

        ctx.add_primitive(Primitive::Triangle {
            points: points.into(),
            fill,
            stroke,
        });
    }
}

impl<D: Float> Triangle<D> {
    /// Creates a new `Triangle` defined by three specific vertices.
    ///
    /// Note: The shape is invisible by default. You must call `.fill()` or `.stroke()` to render it.
    pub const fn new(p1: PlotPoint<D>, p2: PlotPoint<D>, p3: PlotPoint<D>) -> Self {
        Self {
            geometry: Geometry::Vertices([p1, p2, p3]),
            fill: None,
            stroke: None,
        }
    }
    /// Creates a new `Triangle` defined by three specific vertices.
    ///
    /// Note: The shape is invisible by default. You must call `.fill()` or `.stroke()` to render it.
    pub const fn vertices(points: [PlotPoint<D>; 3]) -> Self {
        Self {
            geometry: Geometry::Vertices(points),
            fill: None,
            stroke: None,
        }
    }

    /// Creates a new `Triangle` centered at a point with a specific width and height.
    /// The triangle points **Up** (North).
    ///
    /// Note: The shape is invisible by default. You must call `.fill()` or `.stroke()` to render it.
    pub const fn centered(center: PlotPoint<D>, width: Measure<D>, height: Measure<D>) -> Self {
        Self {
            geometry: Geometry::Centered {
                center,
                width,
                height,
            },
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
}

impl<'a, D: Float, Renderer: crate::Renderer> IntoArea<'a, D, Renderer> for &Triangle<D> {
    fn resolve_area(self, ctx: &plot::Context<'a, D, Renderer>) -> Area {
        match self.geometry {
            Geometry::Vertices(pts) => {
                let p1 = ctx.chart_to_screen(&pts[0]);
                let p2 = ctx.chart_to_screen(&pts[1]);
                let p3 = ctx.chart_to_screen(&pts[2]);
                Area::Triangle {
                    p1: Point::new(p1.x, p1.y),
                    p2: Point::new(p2.x, p2.y),
                    p3: Point::new(p3.x, p3.y),
                }
            }
            Geometry::Centered {
                center,
                width,
                height,
            } => {
                let sc = ctx.chart_to_screen(&center);
                let center_x = sc.x;
                let center_y = sc.y;

                let w = width.resolve_x(ctx);
                let h = height.resolve_y(ctx);

                let half_w = w / 2.0;
                let half_h = h / 2.0;

                Area::Triangle {
                    p1: Point::new(center_x, center_y - half_h),
                    p2: Point::new(center_x + half_w, center_y + half_h),
                    p3: Point::new(center_x - half_w, center_y + half_h),
                }
            }
        }
    }
}
