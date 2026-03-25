use crate::interaction::{Area, IntoArea};
use crate::render::Primitive;
use crate::{Measure, Shape, plot};
use aksel::{Float, PlotPoint};
use iced_core::text::LineHeight;
use iced_core::{
    Color, Font, Point, Radians, Size,
    alignment::{Horizontal, Vertical},
    text::Wrapping,
};
use std::fmt::Debug;

/// Defines how text bounds should be interpreted.
///
/// `BoundsSize` allows you to specify text wrapping bounds in either screen pixels (fixed)
/// or plot units (scales with zoom). This is useful for wrapping text within plot rectangles
/// or maintaining fixed-width text boxes.
#[derive(Debug, Clone, Copy)]
pub enum Bounds<D> {
    /// Fixed bounds in screen pixels.
    ///
    /// The text will wrap at the specified pixel width/height regardless of zoom level.
    Screen(Size),

    /// Bounds in plot data units.
    ///
    /// The text will wrap within a rectangle defined in plot coordinates, scaling
    /// proportionally when zooming the chart.
    Plot(Size<D>),
}

impl<D: Float> Bounds<D> {
    /// Infinite bounds (no wrapping).
    pub const INFINITE: Self = Self::Screen(Size::INFINITE);

    /// Resolves the bounds to screen pixels.
    ///
    /// * If `Screen`, returns the size directly.
    /// * If `Plot`, converts the plot-space size to screen pixels using the transform.
    pub fn resolve(
        &self,
        transform: &aksel::Transform<D, f32, f32>,
        position: &PlotPoint<D>,
    ) -> Size {
        match self {
            Self::Screen(size) => *size,
            Self::Plot(size) => {
                let start = transform.chart_to_screen(position);
                let end = transform.chart_to_screen(&PlotPoint::new(
                    position.x + size.width,
                    position.y + size.height,
                ));

                Size::new((end.x - start.x).abs(), (end.y - start.y).abs())
            }
        }
    }
}

/// A text label rendered as a vector mesh.
///
/// Unlike geometric shapes, labels are rendered using the backend's native text engine
/// ensuring crisp, hinted typography.
///
/// # Example
///
/// ```rust
/// use iced_aksel::{PlotPoint, Measure, shape::Label};
/// use iced::Color;
/// use iced::alignment::{Horizontal, Vertical};
///
/// // Simple centered label
/// let label = Label::new("Hello", PlotPoint::new(10.0, 20.0))
///     .fill(Color::from_rgb(1.0, 0.0, 0.0))
///     .size(Measure::Screen(14.0)); // Keeps size relative to screen
///
/// // Left-aligned label at a data point
/// let marker_label = Label::new("Max", PlotPoint::new(50.0, 100.0))
///     .align(Horizontal::Left, Vertical::Center)
///     .size(Measure::Plot(14.0)); // Keeps size relative to plot
/// ```
#[derive(Debug, Clone)]
pub struct Label<D> {
    /// Text content to display
    pub content: String,
    /// Position in plot coordinates where the label is anchored
    pub position: PlotPoint<D>,
    /// Font-size of the label
    pub size: Measure<D>,
    /// Text rotation in radians
    pub rotation: Radians,
    /// Horizontal alignment
    pub horizontal_alignment: Horizontal,
    /// Vertical alignment
    pub vertical_alignment: Vertical,
    /// Color of the text
    pub fill: Color,
    /// Text tolerance quality override
    pub quality: Option<f32>,
    /// Letter spacing for the text
    ///
    /// TODO: Unused - Add this to rendering implementation!
    // pub letter_spacing: f32,
    /// Font override - Defaults to the default font of the application
    pub font: Option<Font>,
    /// Line height for the text (How much space **between** lines)
    pub line_height: f32,
    /// Bounding box of the text - If this is not set, no wrapping will occur, if set
    pub bounds: Bounds<D>,
    /// Wrapping of the text - Won't have an effect if no bounds have been set
    pub wrapping: Wrapping,
}

impl<D: Float + Debug, R: crate::Renderer> Shape<D, R> for Label<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        let Self {
            content,
            position,
            size,
            rotation,
            horizontal_alignment,
            vertical_alignment,
            fill,
            quality,
            // letter_spacing, // TODO: Use this!
            font,
            line_height,
            bounds,
            wrapping,
        } = self;

        let font = font.unwrap_or_else(|| ctx.default_font());
        // 1. Resolve Position to Screen Coordinates
        let screen_position = ctx.chart_to_screen(&position);

        // 2. Resolve Size (Screen Pixels vs Plot Units)
        let font_size_in_pixels = size.resolve_y(ctx);

        // 3. Resolve bounds
        let bounds = bounds.resolve(ctx, &position);

        // 4. Draw
        ctx.add_primitive(Primitive::Text {
            content,
            position: Point::new(screen_position.x, screen_position.y),
            size: font_size_in_pixels.into(),
            rotation,
            horizontal_alignment,
            vertical_alignment,
            fill,
            quality,
            font,
            line_height: LineHeight::Relative(line_height),
            bounds,
            wrapping,
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
            rotation: Radians(0.0),
            horizontal_alignment: Horizontal::Left,
            vertical_alignment: Vertical::Center,
            fill: Color::BLACK,
            quality: None,
            // letter_spacing: 1.2,
            font: None,
            line_height: 1.0,
            bounds: Bounds::INFINITE,
            wrapping: Wrapping::None,
        }
    }

    /// Add bounds to the label - A label won't wrap, if no bounds are set
    pub const fn bounds(mut self, bounds: Bounds<D>) -> Self {
        self.bounds = bounds;
        self
    }

    /// Set the font for the label - If not set, the default font of the renderer will be rendered
    pub const fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    /// Set the font as an option (See [`Self::font`] for more info)
    pub const fn font_maybe(mut self, font: Option<Font>) -> Self {
        if font.is_some() {
            self.font = font;
        }
        self
    }

    /// Set the wrapping behaviour of the label
    pub const fn wrapping(mut self, wrapping: Wrapping) -> Self {
        self.wrapping = wrapping;
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
    pub fn rotation(mut self, radians: impl Into<Radians>) -> Self {
        self.rotation = radians.into();
        self
    }

    /// Sets the horizontal and vertical alignment relative to the position.
    pub fn align(
        mut self,
        horizontal: impl Into<Horizontal>,
        vertical: impl Into<Vertical>,
    ) -> Self {
        self.horizontal_alignment = horizontal.into();
        self.vertical_alignment = vertical.into();
        self
    }

    /// Overrides the rendering quality (Level of Detail).
    ///
    /// See [`crate::Quality::Custom`] for more info
    pub const fn quality(mut self, tolerance: f32) -> Self {
        self.quality = Some(tolerance);
        self
    }
}

impl<'a, D: Float, Renderer: crate::Renderer> IntoArea<'a, D, Renderer> for &Label<D> {
    fn resolve_area(self, ctx: &plot::Context<'a, D, Renderer>) -> Area {
        let sc = ctx.chart_to_screen(&self.position);
        let screen_pos = Point::new(sc.x, sc.y);

        let font_size_px = self.size.resolve_y(ctx);
        let bounds_size = self.bounds.resolve(ctx, &self.position);
        let font = self.font.unwrap_or_else(|| ctx.default_font());

        let text_size = ctx.measure_text(iced_core::text::Text {
            content: self.content.as_str(),
            bounds: bounds_size,
            size: iced_core::Pixels(font_size_px),
            line_height: iced_core::text::LineHeight::Relative(self.line_height),
            font,
            align_x: self.horizontal_alignment.into(),
            align_y: self.vertical_alignment,
            shaping: iced_core::text::Shaping::Basic,
            wrapping: self.wrapping,
        });

        // 2. Explicitly type the offsets as f32
        let dx: f32 = match self.horizontal_alignment {
            iced_core::alignment::Horizontal::Left => 0.0,
            iced_core::alignment::Horizontal::Center => -text_size.width / 2.0,
            iced_core::alignment::Horizontal::Right => -text_size.width,
        };
        let dy: f32 = match self.vertical_alignment {
            iced_core::alignment::Vertical::Top => 0.0,
            iced_core::alignment::Vertical::Center => -text_size.height / 2.0,
            iced_core::alignment::Vertical::Bottom => -text_size.height,
        };

        let corners: [Point<f32>; 4] = [
            Point::new(dx, dy),
            Point::new(dx + text_size.width, dy),
            Point::new(dx + text_size.width, dy + text_size.height),
            Point::new(dx, dy + text_size.height),
        ];

        let cos_r: f32 = self.rotation.0.cos();
        let sin_r: f32 = self.rotation.0.sin();

        let mut rotated_corners = Vec::with_capacity(4);
        for c in corners {
            let rx: f32 = c.x.mul_add(cos_r, -(c.y * sin_r));
            let ry: f32 = c.x.mul_add(sin_r, c.y * cos_r);
            rotated_corners.push(Point::new(screen_pos.x + rx, screen_pos.y + ry));
        }

        Area::Polygon {
            points: rotated_corners,
        }
    }
}
