use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use aksel::{Float, PlotPoint, Transform};
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
/// .fill(Color::RED);
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

impl<D: Float, R: plot::Renderer> Shape<D, R> for Triangle<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
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

    fn tessellate(
        self,
        transform: &Transform<D, f32, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellator,
    ) {
        let (point_one, point_two, point_three) = match self.geometry {
            Geometry::Vertices(pts) => (
                Point::new(
                    transform.x_to_screen(&pts[0].x),
                    transform.y_to_screen(&pts[0].y),
                ),
                Point::new(
                    transform.x_to_screen(&pts[1].x),
                    transform.y_to_screen(&pts[1].y),
                ),
                Point::new(
                    transform.x_to_screen(&pts[2].x),
                    transform.y_to_screen(&pts[2].y),
                ),
            ),
            Geometry::Centered {
                center,
                width,
                height,
            } => {
                let center_x = transform.x_to_screen(&center.x);
                let center_y = transform.y_to_screen(&center.y);

                let width_pixels = width.resolve_x(transform);
                let height_pixels = height.resolve_y(transform);

                let half_width = width_pixels / 2.0;
                let half_height = height_pixels / 2.0;

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

        let stroke_info = self.stroke.as_ref().and_then(|stroke| {
            // Default to X-axis scale for stroke thickness to ensure consistency
            let width_pixels = stroke.thickness.resolve_x(transform);

            if width_pixels < 0.1 {
                None
            } else {
                Some((stroke, width_pixels))
            }
        });

        tess.draw_triangle(
            buffer,
            point_one,
            point_two,
            point_three,
            self.fill,
            stroke_info,
        );
    }
}
