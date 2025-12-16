use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use aksel::{Float, PlotPoint, Transform};
use iced_core::{Color, Point};

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
        let (x_min, y_min, x_max, y_max) = match self.geometry {
            Geometry::Corners { p1, p2 } => {
                let x1 = transform.x_to_screen(&p1.x);
                let y1 = transform.y_to_screen(&p1.y);
                let x2 = transform.x_to_screen(&p2.x);
                let y2 = transform.y_to_screen(&p2.y);
                (x1.min(x2), y1.min(y2), x1.max(x2), y1.max(y2))
            }
            Geometry::Centered {
                center,
                width,
                height,
            } => {
                let cx = transform.x_to_screen(&center.x);
                let cy = transform.y_to_screen(&center.y);

                let w_px = self.resolve_measure(transform, width, true);
                let h_px = self.resolve_measure(transform, height, false);

                let half_w = w_px / 2.0;
                let half_h = h_px / 2.0;

                (cx - half_w, cy - half_h, cx + half_w, cy + half_h)
            }
        };

        let stroke_info = self.stroke.as_ref().map(|s| {
            let (width_x, width_y) = match s.thickness {
                Measure::Screen(w) => (w, w),
                Measure::Plot(w) => {
                    let w_px_x = self.resolve_measure(transform, Measure::Plot(w), true);
                    let w_px_y = self.resolve_measure(transform, Measure::Plot(w), false);
                    (w_px_x, w_px_y)
                }
            };
            (s, width_x, width_y)
        });

        tess.draw_rectangle(buffer, x_min, y_min, x_max, y_max, self.fill, stroke_info);
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
