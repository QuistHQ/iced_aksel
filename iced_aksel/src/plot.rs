//! Plot rendering and data traits.
//!
//! This module provides the core plotting infrastructure for rendering data on charts.
//! The main entry point is the [`PlotData`] trait, which you implement to draw your data.

use std::ops::Deref;

use crate::{
    layer::LayerId,
    render::{Primitive, RenderCache},
    shape::Shape,
};

use aksel::{Float, PlotRect, Transform};
use iced_core::{Font, Rectangle};

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
    R: crate::Renderer,
{
    /// Draws this data onto the plot.
    ///
    /// Use `plot.add_shape()` to add visual elements to the chart.
    fn draw(&self, plot: &mut Plot<D, R>, theme: &Theme);

    /// The version of the layer, used for caching.
    ///
    /// If version returns None (default), then the layer will always redraw.
    fn version(&self) -> Option<u64> {
        None
    }

    /// The id of the layer. Primarily used by the [`Cached`](crate::Cached) struct.
    fn id(&self) -> Option<LayerId> {
        None
    }
}

/// Internal rendering context for shapes.
///
/// Manages layer ordering and caching for efficient rendering.
pub struct Context<'a, D: Float, Renderer: crate::Renderer = iced_renderer::Renderer> {
    transform: &'a Transform<'a, D, f32, f32>,
    renderer: &'a mut Renderer,
    cache: &'a mut RenderCache<Renderer>,
}

impl<'a, D: Float, Renderer: crate::Renderer> Deref for Context<'a, D, Renderer> {
    type Target = Transform<'a, D, f32, f32>;

    fn deref(&self) -> &Self::Target {
        self.transform
    }
}

impl<'a, D: Float, Renderer: crate::Renderer> Context<'a, D, Renderer> {
    /// Returns the default font of the underlying renderer
    #[inline(always)]
    pub fn default_font(&self) -> Font {
        self.renderer.default_font()
    }

    /// Returns a mutable reference to the underlying [`RenderCache`].
    pub const fn cache(&mut self) -> &mut RenderCache<Renderer> {
        self.cache
    }

    /// Adds a low-level [`Primitive`] directly to the render cache.
    pub fn add_primitive(&mut self, primitive: Primitive) {
        self.cache.add_primitive(primitive);
    }

    /// Returns the screen bounds bounds of the plot
    pub const fn clip_bounds(&self) -> Rectangle {
        let bounds = self.transform.screen_bounds();
        Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bounds.height,
        }
    }
}

/// The plot rendering context for drawing shapes.
///
/// This is passed to your [`PlotData::draw`] implementation. Use [`Plot::add_shape`]
/// to render visual elements.
pub struct Plot<'a, D: Float, R: crate::Renderer = iced_renderer::Renderer> {
    context: Context<'a, D, R>,
}

impl<'a, D, R> Plot<'a, D, R>
where
    D: Float,
    R: crate::Renderer,
{
    /// Creates a new plot context.
    ///
    /// This is typically called internally by the Chart widget.
    pub const fn new(
        renderer: &'a mut R,
        cache: &'a mut RenderCache<R>,
        transform: &'a Transform<'a, D, f32, f32>,
    ) -> Self {
        let context = Context {
            transform,
            renderer,
            cache,
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
    pub fn plot_bounds(&self) -> PlotRect<D> {
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
        shape.render(&mut self.context);
    }
}
