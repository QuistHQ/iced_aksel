//! Shape primitives for rendering on plots.
//!
//! This module provides various geometric shapes that can be drawn on charts:
//!
//! - **[`Arc`]**: Circular arcs and annular sectors
//! - **[`Circle`]**: Filled or stroked circles
//! - **[`Label`]**: Text labels
//! - **[`Line`]**: Straight lines between two points
//! - **[`Polygon`]**: Closed filled or stroked polygons
//! - **[`Polyline`]**: Connected line segments
//! - **[`Rectangle`]**: Axis-aligned rectangles
//! - **[`Triangle`]**: Three-vertex polygons
//!
//! All shapes support both screen-space (pixel) and plot-space (data) sizing via [`Measure`](crate::Measure).
//!
//! # Example
//!
//! ```rust
//! use iced_aksel::{PlotPoint, Measure, shape::{Circle, Line, Rectangle}, Stroke};
//! use iced::Color;
//!
//! // Circle at (10, 20) with radius 5 pixels
//! let circle = Circle::new(PlotPoint::new(10.0, 20.0), Measure::Screen(5.0))
//!     .fill(Color::from_rgb(1.0, 0.0, 0.0));
//!
//! // Line from (0, 0) to (10, 10)
//! let line = Line::new(
//!     PlotPoint::new(0.0, 0.0),
//!     PlotPoint::new(10.0, 10.0),
//!     Stroke::new(Color::from_rgb(0.0, 0.0, 1.0), Measure::Screen(2.0))
//! );
//!
//! // Rectangle with plot-space dimensions
//! let rect = Rectangle::new(
//!     PlotPoint::new(5.0, 5.0),
//!     Measure::Plot(10.0),
//!     Measure::Plot(20.0)
//! ).fill(Color::from_rgb(0.0, 1.0, 0.0));
//! ```

use super::plot;

use aksel::Float;

mod arc;
mod area;
mod bezier;
mod ellipse;
mod label;
mod line;
mod polygon;
mod polyline;
mod rectangle;
mod spline;
mod triangle;

pub use arc::Arc;
pub use area::Area;
pub use bezier::Bezier;
pub use ellipse::Ellipse;
pub use label::{Bounds, Label};
pub use line::Line;
pub use polygon::Polygon;
pub use polyline::Polyline;
pub use rectangle::Rectangle;
pub use spline::Spline;
pub use triangle::Triangle;

/// Trait for shapes that can be rendered on a plot.
///
/// Implement this trait for custom shapes. The rendering context provides access
/// to tessellators (for mesh-based shapes) and text renderers (for labels).
///
/// # Example
///
/// ```rust
/// use iced_aksel::{shape::Shape, plot, PlotPoint};
/// use aksel::Float;
///
/// struct MyCustomShape {
///     position: PlotPoint<f64>,
/// }
///
/// impl Shape<f64> for MyCustomShape {
///     fn render(self, ctx: &mut plot::Context<'_, f64>) {
///         // Use ctx.render_mesh() for geometric shapes
///         // or ctx.render_text() for text rendering
///     }
/// }
/// ```
pub trait Shape<D, Renderer = iced_renderer::Renderer>
where
    D: Float,
    Renderer: plot::Renderer,
{
    /// Renders this shape into the plot context.
    fn render(self, ctx: &mut plot::Context<'_, D, Renderer>);
}
