use crate::Quality;
use crate::render::text::Text;
use crate::{Measure, Shape, plot};
use aksel::{Float, PlotPoint};
use iced_core::{
    Color, Font, Point, Size,
    alignment::Vertical,
    text::{Alignment, Wrapping},
};

/// A text label rendered as a vector mesh.

#[derive(Debug, Clone)]
pub struct Label<D> {
    pub content: String,
    pub position: PlotPoint<D>,
    pub size: Measure<D>,
    pub rotation: f32, // Radians
    pub horizontal_alignment: Alignment,
    pub vertical_alignment: Vertical,
    pub fill: Color,
    pub quality: Quality,
    pub letter_spacing: f32,
    pub font: Option<Font>,
    pub line_height: f32,
    pub bounds: Size,
    pub wrapping: Wrapping,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Label<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        let font = self.font.unwrap_or_else(|| ctx.default_font());
        ctx.render_text(move |transform, text_renderer| {
            // 1. Resolve Position to Screen Coordinates
            let screen_position = Point::new(
                transform.x_to_screen(&self.position.x),
                transform.y_to_screen(&self.position.y),
            );

            // 2. Resolve Size (Screen Pixels vs Plot Units)
            let font_size_in_pixels = self.size.resolve_y(transform);

            // 4. Draw
            text_renderer.draw_text(Text {
                content: &self.content,
                position: screen_position,
                size: font_size_in_pixels.into(),
                rotation: self.rotation,
                horizontal_alignment: self.horizontal_alignment,
                vertical_alignment: self.vertical_alignment,
                fill: self.fill,
                quality: self.quality,
                font,
                line_height: self.line_height.into(),
                bounds: self.bounds,
                wrapping: self.wrapping,
            });
        });
    }
}

impl<D: Float> Label<D> {
    /// Creates a new `Label` at the given position.
    ///
    /// By default, the label is black, 12px (Screen), centered, and unrotated.
    pub fn new(content: impl ToString, position: PlotPoint<D>) -> Self {
        Self {
            content: content.to_string(),
            position,
            size: Measure::Screen(12.0),
            rotation: 0.0,
            horizontal_alignment: Alignment::Center,
            vertical_alignment: Vertical::Center,
            fill: Color::BLACK,
            quality: Quality::default(), // Defaults to Medium
            letter_spacing: 1.2,
            font: None,
            line_height: 1.0,
            bounds: Size::INFINITE,
            wrapping: Wrapping::None,
        }
    }

    pub const fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    pub const fn font_maybe(mut self, font: Option<Font>) -> Self {
        self.font = font;
        self
    }

    /// Sets the fill color of the text.
    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = color;
        self
    }

    /// Sets the size of the text.
    ///
    /// - `Measure::Screen(px)`: Fixed pixel size (e.g., 12px), stays constant when zooming.
    /// - `Measure::Plot(units)`: Size in plot units, scales up/down when zooming.
    pub const fn size(mut self, size: Measure<D>) -> Self {
        self.size = size;
        self
    }

    /// Sets the rotation of the text in radians.
    pub const fn rotation(mut self, radians: f32) -> Self {
        self.rotation = radians;
        self
    }

    /// Sets the horizontal and vertical alignment relative to the position.
    pub fn align(
        mut self,
        horizontal: impl Into<Alignment>,
        vertical: impl Into<Vertical>,
    ) -> Self {
        self.horizontal_alignment = horizontal.into();
        self.vertical_alignment = vertical.into();
        self
    }

    /// Sets the rendering quality (Level of Detail).
    ///
    /// - `Quality::Medium` (Default) is balanced for most cases.
    /// - Use `Quality::Low` if rendering thousands of labels.
    /// - Use `Quality::High` for very large, cinematic text.
    pub const fn quality(mut self, quality: Quality) -> Self {
        self.quality = quality;
        self
    }
}
