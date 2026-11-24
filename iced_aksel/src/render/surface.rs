//! The drawing target of the chart decorations (axes, ticks, labels and spines).

use super::{Primitive, RenderCache};
use iced_core::{Color, Font, Point, Rectangle, Text, renderer::Quad};

/// A target the chart decorations can be drawn onto.
///
/// This lets the axes draw themselves without knowing whether they end up on
/// screen or in an exported document.
pub trait Surface {
    /// The renderer used to measure text.
    type Renderer: crate::Renderer;

    /// Returns the font used when a label doesn't specify one.
    fn default_font(&self) -> Font;

    /// Fills a [`Quad`] with the given color.
    fn fill_quad(&mut self, quad: Quad, color: Color);

    /// Draws the given text with its top-left corner at `position`.
    fn fill_text(
        &mut self,
        text: Text<String, Font>,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    );

    /// Starts a new layer, clipped to the given bounds.
    fn start_layer(&mut self, bounds: Rectangle);

    /// Ends the current layer.
    fn end_layer(&mut self);

    /// Adds a [`Primitive`] to the plot area.
    fn add_primitive(&mut self, primitive: Primitive);
}

/// A [`Surface`] drawing directly to an iced renderer.
pub struct Screen<'a, Renderer: crate::Renderer> {
    pub renderer: &'a mut Renderer,
    pub cache: &'a mut RenderCache<Renderer>,
}

impl<Renderer: crate::Renderer> Surface for Screen<'_, Renderer> {
    type Renderer = Renderer;

    fn default_font(&self) -> Font {
        self.renderer.default_font()
    }

    fn fill_quad(&mut self, quad: Quad, color: Color) {
        self.renderer.fill_quad(quad, color);
    }

    fn fill_text(
        &mut self,
        text: Text<String, Font>,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
        self.renderer.fill_text(text, position, color, clip_bounds);
    }

    fn start_layer(&mut self, bounds: Rectangle) {
        self.renderer.start_layer(bounds);
    }

    fn end_layer(&mut self) {
        self.renderer.end_layer();
    }

    fn add_primitive(&mut self, primitive: Primitive) {
        self.cache.add_primitive(primitive);
    }
}
