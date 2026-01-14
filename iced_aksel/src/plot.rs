//! Plot rendering and data traits.
//!
//! This module provides the core plotting infrastructure for rendering data on charts.
//! The main entry point is the [`PlotData`] trait, which you implement to draw your data.

use crate::{
    render::{MeshBuffer, Tessellator},
    shape::Shape,
};

use aksel::{Float, PlotRect, Transform};
use iced_core::Font;

/// Normalized drag delta for panning operations.
///
/// Values are in the range 0.0-1.0 and can be passed directly to axis `pan` methods.
///
/// # Example
///
/// ```rust
/// use iced_aksel::plot::DragDelta;
///
/// let delta = DragDelta { x: 0.1, y: 0.05 };
/// // Use with state.pan_axes(..., delta.x, delta.y)
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DragDelta {
    /// Normalized horizontal drag distance (0.0-1.0).
    pub x: f32,
    /// Normalized vertical drag distance (0.0-1.0).
    pub y: f32,
}

/// Renderer requirements for plotting.
///
/// This trait is automatically implemented for any renderer that satisfies the requirements.
pub trait Renderer:
    iced_core::Renderer
    + iced_graphics::mesh::Renderer
    + iced_core::text::Renderer<Font = iced_core::Font>
{
}

impl<T> Renderer for T where
    T: iced_core::Renderer
        + iced_graphics::mesh::Renderer
        + iced_core::text::Renderer<Font = iced_core::Font>
{
}

/// Trait for drawable data on a plot.
///
/// Implement this trait for your data types to render them on a chart. The `draw` method
/// receives a [`Plot`] context where you can add shapes.
///
/// # Example
///
/// ```rust
/// use iced_aksel::{plot::{Plot, PlotData}, PlotPoint, shape::Ellipse, Measure};
/// use iced::{Color, Theme};
///
/// struct DataPoints {
///     points: Vec<PlotPoint<f64>>,
/// }
///
/// impl PlotData<f64> for DataPoints {
///     fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
///         for point in &self.points {
///             plot.add_shape(
///                 Ellipse::new(*point, Measure::Screen(20.0), Measure::Screen(10.0))
///                     .fill(theme.palette().primary)
///             );
///         }
///     }
/// }
/// ```
pub trait PlotData<D, R = iced_renderer::Renderer, Theme = iced_core::Theme>
where
    D: Float,
    R: Renderer,
{
    /// Draws this data onto the plot.
    ///
    /// Use `plot.add_shape()` to add visual elements to the chart.
    fn draw(&self, plot: &mut Plot<D, R>, theme: &Theme);
}

/// Internal rendering context for shapes.
///
/// Manages layer ordering and buffering for efficient rendering.
pub struct Context<'a, D: Float, Renderer: self::Renderer = iced_renderer::Renderer> {
    transform: &'a Transform<'a, D, f32, f32>,
    clip_bounds: &'a iced_core::Rectangle,
    renderer: &'a mut Renderer,
    tessellators: &'a mut Tessellator,
    mesh_buffer: &'a mut MeshBuffer,
}

impl<'a, D: Float, Renderer: self::Renderer> Context<'a, D, Renderer> {
    /// Returns the default font of the underlying renderer
    #[inline(always)]
    pub fn default_font(&mut self) -> Font {
        self.renderer.default_font()
    }

    /// Renders a mesh-based shape (lines, polygons, etc.).
    ///
    /// Used internally by shapes to add geometry to the mesh buffer.
    pub fn render_mesh<F>(&mut self, f: F)
    where
        F: FnOnce(&Transform<'a, D, f32, f32>, &mut MeshBuffer, &mut Tessellator),
    {
        // Draw mesh
        f(self.transform, self.mesh_buffer, self.tessellators);

        // If buffer exceeds limit, render the mesh
        if self.mesh_buffer.vertices_count() >= self.mesh_buffer.limit() {
            self.mesh_buffer.render(self.renderer, self.clip_bounds);
        }
    }
}

/// The plot rendering context for drawing shapes.
///
/// This is passed to your [`PlotData::draw`] implementation. Use [`Plot::add_shape`]
/// to render visual elements.
pub struct Plot<'a, D: Float, R: self::Renderer = iced_renderer::Renderer> {
    context: Context<'a, D, R>,
}

impl<'a, D, R> Plot<'a, D, R>
where
    D: Float,
    R: self::Renderer,
{
    /// Creates a new plot context.
    ///
    /// This is typically called internally by the Chart widget.
    pub const fn new(
        tessellators: &'a mut Tessellator,
        renderer: &'a mut R,
        clip_bounds: &'a iced_core::Rectangle,
        mesh_buffer: &'a mut MeshBuffer,
        transform: &'a Transform<'a, D, f32, f32>,
    ) -> Self {
        let context = Context {
            transform,
            clip_bounds,
            renderer,
            tessellators,
            mesh_buffer,
        };
        Self { context }
    }

    /// Returns the current plot bounds in data space.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::plot::{Plot, PlotData};
    /// # struct MyData;
    /// # impl PlotData<f64> for MyData {
    /// #     fn draw(&self, plot: &mut Plot<f64>, theme: &iced::Theme) {
    /// let bounds = plot.bounds();
    /// let (x_min, x_max) = (bounds.min_x(), bounds.max_x());
    /// let (y_min, y_max) = (bounds.min_y(), bounds.max_y());
    /// #     }
    /// # }
    /// ```
    pub fn bounds(&self) -> PlotRect<D> {
        self.context.transform.plot_bounds()
    }

    /// Adds a shape to the plot.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::{plot::{Plot, PlotData}, PlotPoint, shape::Ellipse, Measure};
    /// # use iced::Color;
    /// # struct MyData;
    /// # impl PlotData<f64> for MyData {
    /// #     fn draw(&self, plot: &mut Plot<f64>, theme: &iced::Theme) {
    /// plot.add_shape(
    ///     Ellipse::new(PlotPoint::new(5.0, 10.0), Measure::Screen(20.0), Measure::Screen(10.0))
    ///         .fill(Color::from_rgb(1.0, 0.0, 0.0))
    /// );
    /// #     }
    /// # }
    /// ```
    pub fn add_shape<S: Shape<D, R>>(&mut self, shape: S) {
        shape.render(&mut self.context)
    }
}

impl<'a, D, R> Drop for Plot<'a, D, R>
where
    D: Float,
    R: self::Renderer,
{
    fn drop(&mut self) {
        self.context
            .mesh_buffer
            .render(self.context.renderer, self.context.clip_bounds);
    }
}
