use crate::font::{self, GeometricFont};
use crate::{Measure, Shape, plot};
use aksel::{Float, PlotPoint};
use iced_core::{
    Color, Point,
    alignment::{Horizontal, Vertical},
};

// Renamed Struct
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
}

impl<'a, D: Float, R: plot::Renderer> Shape<D, R> for VectorLabel<'a, D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tessellator| {
            // ... (Resolution logic same as before) ...
            let position = Point::new(
                transform.x_to_screen(&self.position.x),
                transform.y_to_screen(&self.position.y),
            );
            let font_size_px = self.size.resolve_y(transform);

            // Logic to choose font
            let font_ref = if let Some(f) = self.font {
                f
            } else {
                font::default()
            };

            // Call the new method on Tessellator
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
            );
        });
    }
}

// Renamed Implementation
impl<'a, D: Float> VectorLabel<'a, D> {
    /// Creates a new VectorLabel.
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
}
