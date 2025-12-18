use crate::Quality;
use crate::render::font::{self, GeometricFont};
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
pub struct VectorLabel<'a, D> {
    pub content: String,
    pub position: PlotPoint<D>,
    pub size: Measure<D>,
    pub rotation: f32, // Radians
    pub horizontal_alignment: Horizontal,
    pub vertical_alignment: Vertical,
    pub fill: Color,
    /// The font to use. If `None`, the default system font (IBM) is used.
    pub font: Option<&'a GeometricFont<'a>>,
    /// The rendering quality (level of detail).
    pub quality: Quality,
}

impl<'a, D: Float, R: plot::Renderer> Shape<D, R> for VectorLabel<'a, D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tessellator| {
            // 1. Resolve Position to Screen Coordinates
            let screen_position = Point::new(
                transform.x_to_screen(&self.position.x),
                transform.y_to_screen(&self.position.y),
            );

            // 2. Resolve Size (Screen Pixels vs Plot Units)
            let font_size_in_pixels = self.size.resolve_y(transform);

            // 3. Resolve Font (Use default if none provided)
            // We use a reference to avoid lifetime issues with 'static vs 'a
            let font_reference = if let Some(user_font) = self.font {
                user_font
            } else {
                font::default()
            };

            // 4. Draw
            tessellator.draw_vector_text(
                buffer,
                &self.content,
                screen_position,
                font_size_in_pixels,
                self.rotation,
                self.fill,
                font_reference,
                self.horizontal_alignment,
                self.vertical_alignment,
                self.quality,
            );
        });
    }
}

impl<'a, D: Float> VectorLabel<'a, D> {
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
            font: None,
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

    /// Sets a custom font.
    ///
    /// You must load the font into a [`GeometricFont`] first.
    pub fn font(mut self, font: &'a GeometricFont<'a>) -> Self {
        self.font = Some(font);
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
