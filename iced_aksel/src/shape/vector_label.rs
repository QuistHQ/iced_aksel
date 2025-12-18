use crate::font::{self, GeometricFont};
use crate::{Measure, Shape, plot};
use aksel::{Float, PlotPoint};
use iced_core::{
    Color, Point,
    alignment::{Horizontal, Vertical},
};

#[derive(Debug, Clone)]
pub struct VectorLabel<'a, D> {
    pub content: String,
    pub position: PlotPoint<D>,
    pub size: Measure<D>,
    pub rotation: f32,
    pub horizontal_alignment: Horizontal,
    pub vertical_alignment: Vertical,
    pub fill: Color,
    pub font: Option<&'a GeometricFont<'a>>,
    // NEW: Adaptive Level of Detail control
    pub tolerance: f32,
}

impl<'a, D: Float, R: plot::Renderer> Shape<D, R> for VectorLabel<'a, D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tessellator| {
            // 1. Resolve Position
            let position = Point::new(
                transform.x_to_screen(&self.position.x),
                transform.y_to_screen(&self.position.y),
            );

            // 2. Resolve Size
            let font_size_px = self.size.resolve_y(transform);

            // 3. Resolve Font (Handle lifetime variance)
            let font_ref = if let Some(f) = self.font {
                f
            } else {
                font::default()
            };

            // 4. Draw with Tolerance
            tessellator.draw_vector_text(
                buffer,
                &self.content,
                position,
                font_size_px,
                self.rotation,
                self.fill,
                font_ref,
                self.horizontal_alignment,
                self.vertical_alignment,
                self.tolerance,
            );
        });
    }
}

impl<'a, D: Float> VectorLabel<'a, D> {
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
            // Default: 0.5px max error.
            // 0.1 is "High Quality", 1.0 is "Fast", 2.0 is "Low Poly".
            tolerance: 0.5,
        }
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill = color;
        self
    }

    pub fn size(mut self, size: Measure<D>) -> Self {
        self.size = size;
        self
    }

    pub fn rotation(mut self, radians: f32) -> Self {
        self.rotation = radians;
        self
    }

    pub fn align(mut self, horizontal: Horizontal, vertical: Vertical) -> Self {
        self.horizontal_alignment = horizontal;
        self.vertical_alignment = vertical;
        self
    }

    pub fn font(mut self, font: &'a GeometricFont<'a>) -> Self {
        self.font = Some(font);
        self
    }

    /// Sets the rendering precision (Level of Detail).
    ///
    /// The `tolerance` value represents the maximum allowed deviation (in screen pixels)
    /// between the mathematical curve and the generated mesh.
    ///
    /// - **0.1**: Cinematic quality (High vertex count, slower). Use for very large text.
    /// - **0.5** (Default): Balanced. Indistinguishable from perfect on most screens.
    /// - **1.0+**: Low detail (Fast). Good for small labels or high-performance requirements.
    pub fn tolerance(mut self, pixel_error: f32) -> Self {
        self.tolerance = pixel_error;
        self
    }
}
