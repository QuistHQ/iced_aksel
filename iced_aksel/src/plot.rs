//! Plot rendering and data traits.
//!
//! This module provides the core plotting infrastructure for rendering data on charts.
//! The main entry point is the [`PlotData`] trait, which you implement to draw your data.

use std::hash::Hash;
use std::ops::Deref;

use crate::{
    interaction::{self, Interaction},
    layer::LayerId,
    render::{Primitive, RenderCache},
};

use crate::interaction::InteractionsCache;
use aksel::{Float, PlotRect, Transform};
use iced_core::{Font, Rectangle};

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
pub trait PlotData<D, Message, Tag = (), R = iced_renderer::Renderer, Theme = iced_core::Theme>
where
    Message: Clone,
    D: Float,
    R: crate::Renderer,
    Tag: Hash + Eq + Clone,
{
    /// Draws this data onto the plot.
    ///
    /// Use `plot.add_shape()` to add visual elements to the chart.
    fn draw(&self, plot: &mut Plot<D, Message, Tag, R>, theme: &Theme);

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
    pub(crate) transform: &'a Transform<'a, D, f32, f32>,
    pub(crate) renderer: &'a mut Renderer,
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

    /// Measures a Text to get it's bounds
    pub fn measure_text(&self, text: iced_core::text::Text<&str>) -> iced_core::Size {
        use iced_core::text::Paragraph as _;
        <Renderer as iced_core::text::Renderer>::Paragraph::with_text(text).min_bounds()
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
pub struct Plot<
    'a,
    D: Float,
    Message: Clone,
    Tag = (),
    Renderer: crate::Renderer = iced_renderer::Renderer,
> {
    pub(crate) context: Context<'a, D, Renderer>,
    interactions: &'a mut InteractionsCache<Message, Tag>,
}

impl<'a, D: Float, Message: Clone, Tag, Renderer: crate::Renderer> Deref
    for Plot<'a, D, Message, Tag, Renderer>
{
    type Target = Context<'a, D, Renderer>;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl<'a, D, Message, Tag, Renderer> Plot<'a, D, Message, Tag, Renderer>
where
    Message: Clone,
    Tag: Hash + Eq + Clone,
    D: Float,
    Renderer: crate::Renderer,
{
    /// Creates a new plot context.
    ///
    /// This is typically called internally by the Chart widget.
    pub(crate) const fn new(
        renderer: &'a mut Renderer,
        cache: &'a mut RenderCache<Renderer>,
        transform: &'a Transform<'a, D, f32, f32>,
        interactions: &'a mut InteractionsCache<Message, Tag>,
    ) -> Self {
        let context = Context {
            transform,
            renderer,
            cache,
        };
        Self {
            context,
            interactions,
        }
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

    /// Renders a shape to the plot.
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
    pub fn render<S: crate::shape::Shape<D, Renderer>>(&mut self, shape: S) {
        shape.render(&mut self.context);
    }

    /// Pushes an interaction to the plot
    pub fn push_interaction(
        &mut self,
        id: impl Into<interaction::Id<Tag>>,
        interaction: impl Into<Interaction<Message, Tag>>,
    ) {
        let interaction = interaction.into();
        let Some(interaction) = interaction::ResolvedInteraction::new(interaction) else {
            return;
        };
        self.interactions.insert(id.into(), interaction);
    }
}
