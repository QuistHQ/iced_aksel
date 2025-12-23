use crate::{Shape, plot};
use aksel::{Float, PlotPoint};
use iced_core::{
    Color, Pixels, Point, Rectangle, Size,
    alignment::{Horizontal, Vertical},
    text::{LineHeight, Shaping, Text, Wrapping},
};

/// A text label positioned at a specific point in the chart.
///
/// Unlike geometric shapes, labels are rendered using the backend's native text engine
/// via [`TextRenderer::fill_text`](crate::plot::TextRenderer::fill_text), ensuring crisp, hinted typography.
///
/// # Example
///
/// ```rust
/// use iced_aksel::{PlotPoint, shape::Label};
/// use iced::Color;
/// use iced::alignment::{Horizontal, Vertical};
///
/// // Simple centered label
/// let label = Label::new("Hello", PlotPoint::new(10.0, 20.0))
///     .fill(Color::from_rgb(1.0, 0.0, 0.0))
///     .size(16.0);
///
/// // Left-aligned label at a data point
/// let marker_label = Label::new("Max", PlotPoint::new(50.0, 100.0))
///     .align(Horizontal::Left, Vertical::Center)
///     .size(14.0);
/// ```
#[derive(Debug, Clone)]
pub struct Label<D> {
    /// The text content to display
    pub content: String,
    /// The position in plot coordinates where the label is anchored
    pub position: PlotPoint<D>,
    /// Horizontal alignment relative to the position
    pub horizontal_alignment: Horizontal,
    /// Vertical alignment relative to the position
    pub vertical_alignment: Vertical,
    /// The color of the text
    pub fill: Color,
    /// The font size in logical pixels
    pub font_size: f32,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Label<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_text(move |transform, renderer| {
            // 1. Resolve Position
            let position = Point::new(
                transform.x_to_screen(&self.position.x),
                transform.y_to_screen(&self.position.y),
            );

            // 2. Resolve Clip Bounds (Screen Rect)
            let b = transform.screen_bounds();
            let clip_bounds = Rectangle::new(Point::new(b.x, b.y), Size::new(b.width, b.height));

            // 3. Construct Iced Text Object
            let text = Text {
                content: self.content,
                bounds: Size::new(500., 500.),
                size: Pixels(self.font_size),
                line_height: LineHeight::default(),
                font: renderer.default_font(),
                align_x: self.horizontal_alignment.into(),
                align_y: self.vertical_alignment,
                shaping: Shaping::Basic,
                wrapping: Wrapping::None,
            };

            // 4. Draw
            renderer.fill_text(text, position, self.fill, clip_bounds);
        });
    }
}

impl<D: Float> Label<D> {
    // =========================================================================
    //  Constructors
    // =========================================================================

    /// Creates a new Label.
    ///
    /// Default style: Black, 12px, Centered.
    pub fn new(content: impl ToString, position: PlotPoint<D>) -> Self {
        Self {
            content: content.to_string(),
            position,
            horizontal_alignment: Horizontal::Center,
            vertical_alignment: Vertical::Center,
            fill: Color::BLACK,
            font_size: 12.0,
        }
    }

    // =========================================================================
    //  Builder Methods
    // =========================================================================

    /// Sets the text color.
    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = color;
        self
    }

    /// Sets the font size in logical pixels.
    pub const fn size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    // TODO: Re-enable this - How can we do that?
    // /// Sets the font.
    // pub const fn font(mut self, font: Font) -> Self {
    //     self.font = font;
    //     self
    // }

    /// Sets the horizontal and vertical alignment relative to the `position`.
    ///
    /// - `Horizontal::Left`: The text starts at `position.x`.
    /// - `Horizontal::Center`: The text is centered on `position.x`.
    /// - `Horizontal::Right`: The text ends at `position.x`.
    ///
    /// (Similarly for Vertical alignment)
    pub const fn align(mut self, horizontal: Horizontal, vertical: Vertical) -> Self {
        self.horizontal_alignment = horizontal;
        self.vertical_alignment = vertical;
        self
    }
}
