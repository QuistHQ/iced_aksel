use crate::Quality;
use crate::render::text::Text;
use crate::{Measure, Shape, plot};
use aksel::{Float, PlotPoint};
use iced_core::{
    Color, Point,
    alignment::{Horizontal, Vertical},
};

/// A text label rendered as a vector mesh.
///
/// Unlike the standard [`Label`](crate::shape::Label), this converts a .ttf SVG into triangles,
/// allowing for more dynamic text aswell as rotation capabilities.
///
/// # Use Cases
/// - **Rotation:** Can be rotated to any angle (e.g., vertical axis labels).
/// - **Scaling:** Can use `Measure::Plot` to scale perfectly with the graph zoom.
/// - **Precision:** Maintains infinite sharpness at any zoom level.
///
/// # Performance Note
/// This is computationally more expensive than `Label`. For static, horizontal text
/// (like titles), prefer using `Label`. Use `VectorLabel` when rotation or dynamic scaling is required.
#[derive(Debug, Clone)]
pub struct VectorLabel<D> {
    pub content: String,
    pub position: PlotPoint<D>,
    pub size: Measure<D>,
    pub rotation: f32, // Radians
    pub horizontal_alignment: Horizontal,
    pub vertical_alignment: Vertical,
    pub fill: Color,
    /// The rendering quality (level of detail).
    pub quality: Quality,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for VectorLabel<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
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
            });
        });
    }
}

impl<D: Float> VectorLabel<D> {
    /// Creates a new `VectorLabel` at the given position.
    ///
    /// By default, the label is black, 12px (Screen), centered, and unrotated.
    pub fn new(content: impl ToString, position: PlotPoint<D>) -> Self {
        Self {
            content: content.to_string(),
            position,
            size: Measure::Screen(12.0),
            rotation: 0.0,
            horizontal_alignment: Horizontal::Center,
            vertical_alignment: Vertical::Center,
            fill: Color::BLACK,
            quality: Quality::default(), // Defaults to Medium
        }
    }

    /// Sets the fill color of the text.
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = color;
        self
    }

    /// Sets the size of the text.
    ///
    /// - `Measure::Screen(px)`: Fixed pixel size (e.g., 12px), stays constant when zooming.
    /// - `Measure::Plot(units)`: Size in plot units, scales up/down when zooming.
    pub fn size(mut self, size: Measure<D>) -> Self {
        self.size = size;
        self
    }

    /// Sets the rotation of the text in radians.
    pub fn rotation(mut self, radians: f32) -> Self {
        self.rotation = radians;
        self
    }

    /// Sets the horizontal and vertical alignment relative to the position.
    pub fn align(mut self, horizontal: Horizontal, vertical: Vertical) -> Self {
        self.horizontal_alignment = horizontal;
        self.vertical_alignment = vertical;
        self
    }

    /// Sets the rendering quality (Level of Detail).
    ///
    /// - `Quality::Medium` (Default) is balanced for most cases.
    /// - Use `Quality::Low` if rendering thousands of labels.
    /// - Use `Quality::High` for very large, cinematic text.
    pub fn quality(mut self, quality: Quality) -> Self {
        self.quality = quality;
        self
    }
}
