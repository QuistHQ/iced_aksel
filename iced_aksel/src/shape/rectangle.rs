use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use aksel::{Float, PlotPoint, Transform};
use iced_core::Color;

/// A primitive representing an axis-aligned box.
///
/// Rectangles can be defined in two ways:
/// 1. **By Corners:** Defining a region between two specific data points (e.g., for Bar Charts).
/// 2. **Centered:** Defining a fixed-size box around a specific point (e.g., for Square Markers).
///
/// # Usage
///
/// ## 1. Data Region (Bar Chart)
/// ```rust
/// use iced_aksel::shape::Rectangle;
/// use aksel::PlotPoint;
///
/// // Spans strictly from (0,0) to (1,5) in plot coordinates
/// let bar = Rectangle::corners(
///     PlotPoint::new(0.0, 0.0),
///     PlotPoint::new(1.0, 5.0)
/// );
/// ```
///
/// ## 2. Fixed Marker (UI)
/// ```rust
/// use iced_aksel::shape::Rectangle;
/// use iced_aksel::Measure;
/// use aksel::PlotPoint;
///
/// // Always 20x20 pixels, centered at (5,5)
/// let marker = Rectangle::centered(
///     PlotPoint::new(5.0, 5.0),
///     Measure::Screen(20.0),
///     Measure::Screen(20.0)
/// );
/// ```
#[derive(Debug, Clone)]
enum Geometry<D> {
    /// Defined by two opposite corners in plot space.
    Corners { p1: PlotPoint<D>, p2: PlotPoint<D> },
    /// Defined by a center point and explicit dimensions.
    Centered {
        center: PlotPoint<D>,
        width: Measure<D>,
        height: Measure<D>,
    },
}

#[derive(Debug, Clone)]
pub struct Rectangle<D> {
    geometry: Geometry<D>,
    pub fill: Option<Color>,
    pub stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Rectangle<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Rectangle<D> {
    /// Creates a new `Rectangle` defined by two opposite corners in plot coordinates.
    ///
    /// Note: The shape is invisible by default. You must call `.fill()` or `.stroke()` to render it.
    pub const fn corners(p1: PlotPoint<D>, p2: PlotPoint<D>) -> Self {
        Self {
            geometry: Geometry::Corners { p1, p2 },
            fill: None,
            stroke: None,
        }
    }

    /// Creates a new `Rectangle` centered at a specific point with defined dimensions.
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

    /// Sets the fill color of the rectangle.
    #[inline]
    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    /// Sets the stroke style (border) of the rectangle.
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
        let (x_minimum, y_minimum, x_maximum, y_maximum) = match self.geometry {
            Geometry::Corners { p1, p2 } => {
                let x_one = transform.x_to_screen(&p1.x);
                let y_one = transform.y_to_screen(&p1.y);
                let x_two = transform.x_to_screen(&p2.x);
                let y_two = transform.y_to_screen(&p2.y);

                (
                    x_one.min(x_two),
                    y_one.min(y_two),
                    x_one.max(x_two),
                    y_one.max(y_two),
                )
            }
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

                (
                    center_x - half_width,
                    center_y - half_height,
                    center_x + half_width,
                    center_y + half_height,
                )
            }
        };

        let stroke_info = self.stroke.as_ref().map(|s| {
            let width_x = s.thickness.resolve_x(transform);
            let width_y = s.thickness.resolve_y(transform);
            // Pass tuple of (width_x, width_y) to support anisotropic strokes if needed
            (s, width_x, width_y)
        });

        tess.draw_rectangle(
            buffer,
            x_minimum,
            y_minimum,
            x_maximum,
            y_maximum,
            self.fill,
            stroke_info,
        );
    }
}
