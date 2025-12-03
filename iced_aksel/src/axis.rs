use aksel::{Float, Tick};
use derivative::Derivative;
use iced::{
    Pixels, Point, Rectangle, Size,
    advanced::{
        Layout, Text,
        graphics::{color, mesh::SolidVertex2D},
        layout::{Limits, Node},
        renderer::Quad,
        text::paragraph::Plain,
    },
    alignment::Vertical,
    mouse::Cursor,
    widget::text::{Alignment, Shaping},
};

use crate::{
    Catalog,
    render::MeshBuffer,
    style::{AxisStyle, Style},
};

use super::Scale;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Top,
    Bottom,
    Left,
    Right,
}

impl From<Position> for Orientation {
    fn from(value: Position) -> Self {
        match value {
            Position::Top | Position::Bottom => Self::Horizontal,
            Position::Left | Position::Right => Self::Vertical,
        }
    }
}

impl<'a> From<&'a Position> for Orientation {
    fn from(value: &'a Position) -> Self {
        match value {
            Position::Top | Position::Bottom => Self::Horizontal,
            Position::Left | Position::Right => Self::Vertical,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub struct GridLine {
    pub thickness: Pixels,
}

impl Default for GridLine {
    fn default() -> Self {
        Self {
            thickness: Pixels(1.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TickLine {
    pub thickness: Pixels,
    pub length: Pixels,
    pub label: Option<Label>,
}

impl Default for TickLine {
    #[inline(always)]
    fn default() -> Self {
        Self {
            thickness: Pixels(1.0),
            length: Pixels(5.0),
            label: None,
        }
    }
}

impl TickLine {
    #[inline(always)]
    pub fn simple(content: String) -> Self {
        Self {
            label: Some(Label {
                content,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Label {
    pub size: Pixels,
    pub content: String,
}

impl Default for Label {
    #[inline(always)]
    fn default() -> Self {
        Self {
            size: Pixels(12.0),
            content: String::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LabelBounds {
    pub start: f32,
    pub end: f32,
}

impl LabelBounds {
    pub const fn new(start: f32, end: f32) -> Self {
        Self { start, end }
    }

    pub fn overlaps_with_gap(&self, other: &Self, min_gap: f32) -> bool {
        (self.start < other.end + min_gap) && (other.start < self.end + min_gap)
    }
}

#[derive(Debug, Clone)]
pub struct PlacedLabelInfo<D> {
    pub tick: Tick<D>,
    pub normalized_position: f32,
    pub bounds: LabelBounds,
}

#[derive(Debug, Clone, Copy)]
pub struct TickLabelContext<D> {
    pub tick: Tick<D>,
    pub normalized_position: f32,
    pub axis_bounds: Rectangle,
    pub orientation: Orientation,
}

impl<D> TickLabelContext<D> {
    pub const fn axis_span(&self) -> f32 {
        match self.orientation {
            Orientation::Horizontal => self.axis_bounds.width,
            Orientation::Vertical => self.axis_bounds.height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelDecision {
    Render,
    Skip,
}

#[derive(Debug)]
pub struct LabelDecisionContext<'a, D> {
    pub tick: Tick<D>,
    pub normalized_position: f32,
    pub bounds: LabelBounds,
    pub orientation: Orientation,
    pub accepted: &'a [PlacedLabelInfo<D>],
}

type LabelPolicyFn<D> = dyn for<'a> Fn(LabelDecisionContext<'a, D>) -> LabelDecision + 'static;

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub enum LabelPolicy<D> {
    #[default]
    All,
    SkipOverlapping {
        min_gap: f32,
    },
    Custom(#[derivative(Debug = "ignore")] Box<LabelPolicyFn<D>>),
}

impl<D> LabelPolicy<D> {
    pub const fn all() -> Self {
        Self::All
    }

    pub const fn skip_overlapping(min_gap: f32) -> Self {
        Self::SkipOverlapping { min_gap }
    }

    pub fn custom<F>(policy: F) -> Self
    where
        F: for<'a> Fn(LabelDecisionContext<'a, D>) -> LabelDecision + 'static,
    {
        Self::Custom(Box::new(policy))
    }

    fn should_render(&self, context: LabelDecisionContext<'_, D>) -> bool {
        match self {
            Self::All => true,
            Self::SkipOverlapping { min_gap } => context
                .accepted
                .iter()
                .all(|placed| !context.bounds.overlaps_with_gap(&placed.bounds, *min_gap)),
            Self::Custom(policy) => matches!(policy(context), LabelDecision::Render),
        }
    }
}

/// DISCUSS: Skal grids og axis-ticks altid være det samme?
type TickRendererFn<D> = Box<dyn Fn(TickLabelContext<D>) -> Option<TickLine>>;
type LabelFormatter<D> = Box<dyn Fn(D) -> Option<Label>>;
type GridRendererFn<D> = Box<dyn Fn(Tick<D>) -> Option<GridLine>>;

struct LabelCandidate<D> {
    tick: Tick<D>,
    normalized_position: f32,
    label: Label,
}

struct ResolvedLabelCandidate<Renderer, D>
where
    Renderer: iced::advanced::text::Renderer,
{
    tick: Tick<D>,
    normalized_position: f32,
    bounds: LabelBounds,
    paragraph: Plain<Renderer::Paragraph>,
    position: Point,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Axis<D> {
    position: Position,
    thickness: Pixels,
    invisible: bool,
    render_cursor: bool,
    label_spacing: Pixels,

    #[derivative(Debug = "ignore")]
    scale: Box<dyn Scale<Domain = D, Normalized = D>>,
    #[derivative(Debug = "ignore")]
    pub(crate) grid_renderer: Option<GridRendererFn<D>>,
    #[derivative(Debug = "ignore")]
    pub(crate) tick_renderer: Option<TickRendererFn<D>>,
    #[derivative(Debug = "ignore")]
    pub(crate) cursor_formatter: Option<LabelFormatter<D>>,
    #[derivative(Debug = "ignore")]
    label_policy: LabelPolicy<D>,
}

impl<D: Float> Axis<D> {
    pub fn new(
        scale: impl Scale<Domain = D, Normalized = D> + 'static,
        position: Position,
    ) -> Self {
        // Default grid renderer - only render major ticks (level 0)
        let grid_renderer = Box::new(|tick: Tick<D>| {
            if tick.level != 0 {
                return None;
            }
            Some(GridLine {
                thickness: 1.0.into(),
            })
        });

        let tick_renderer = Box::new(|ctx: TickLabelContext<D>| {
            Some(TickLine {
                thickness: 1.0.into(),
                length: match ctx.tick.level {
                    0 => 10.0,
                    _ => 5.0,
                }
                .into(),
                label: None,
            })
        });

        Self {
            position,
            thickness: 50.0.into(),
            render_cursor: true,
            invisible: false,
            label_spacing: 5.0.into(),

            scale: Box::new(scale),
            grid_renderer: Some(grid_renderer),
            tick_renderer: Some(tick_renderer),
            cursor_formatter: None,
            label_policy: LabelPolicy::default(),
        }
    }

    /// Sets the spacing between labels and tick-lines
    pub fn with_label_spacing<P: Into<Pixels>>(mut self, spacing: P) -> Self {
        self.label_spacing = spacing.into();
        self
    }

    /// Sets the thickness of the axis (width for vertical, heigh for horizontal)
    pub fn with_thickness<P: Into<Pixels>>(mut self, thickness: P) -> Self {
        self.thickness = thickness.into();
        self
    }

    pub fn with_grid_renderer<F>(mut self, renderer: F) -> Self
    where
        F: Fn(Tick<D>) -> Option<GridLine> + 'static,
    {
        self.grid_renderer = Some(Box::new(renderer));
        self
    }

    pub fn without_grid(mut self) -> Self {
        self.grid_renderer = None;
        self
    }

    pub fn with_tick_renderer<F>(mut self, renderer: F) -> Self
    where
        F: Fn(TickLabelContext<D>) -> Option<TickLine> + 'static,
    {
        self.tick_renderer = Some(Box::new(renderer));
        self
    }

    pub fn set_tick_renderer<F>(&mut self, renderer: F)
    where
        F: Fn(TickLabelContext<D>) -> Option<TickLine> + 'static,
    {
        self.tick_renderer = Some(Box::new(renderer));
    }

    pub fn label_policy(mut self, policy: LabelPolicy<D>) -> Self {
        self.label_policy = policy;
        self
    }

    pub fn skip_overlapping_labels(mut self, min_gap_px: f32) -> Self {
        self.label_policy = LabelPolicy::skip_overlapping(min_gap_px);
        self
    }

    pub fn with_custom_label_policy<F>(mut self, policy: F) -> Self
    where
        F: for<'a> Fn(LabelDecisionContext<'a, D>) -> LabelDecision + 'static,
    {
        self.label_policy = LabelPolicy::custom(policy);
        self
    }

    pub fn with_cursor_formatter<F>(mut self, renderer: F) -> Self
    where
        F: Fn(D) -> Option<Label> + 'static,
    {
        self.cursor_formatter = Some(Box::new(renderer));
        self
    }

    pub const fn invisible(mut self) -> Self {
        self.invisible = true;
        self
    }

    pub const fn set_visible(&mut self, visible: bool) {
        self.invisible = !visible;
    }

    pub const fn is_visible(&self) -> bool {
        !self.invisible
    }

    pub fn domain(&self) -> (&D, &D) {
        self.scale.domain()
    }

    pub fn scale(&self) -> &dyn Scale<Domain = D, Normalized = D> {
        &*self.scale
    }

    pub fn scale_mut(&mut self) -> &mut dyn Scale<Domain = D, Normalized = D> {
        &mut *self.scale
    }

    pub const fn position(&self) -> &Position {
        &self.position
    }

    pub fn orientation(&self) -> Orientation {
        Orientation::from(&self.position)
    }

    pub fn set_thickness<P: Into<Pixels>>(&mut self, thickness: P) {
        self.thickness = thickness.into();
    }

    /// How thick this axis wants to be in the direction
    /// perpendicular to the chart.
    ///
    /// Top/Bottom -> height
    /// Left/Right -> width
    ///
    /// For now it's a placeholder; customize this later using
    /// `tick_renderer`, fonts, padding, etc.
    pub const fn thickness(&self) -> Pixels {
        self.thickness
    }

    /// Convert a screen position to a normalized value (0.0-1.0) along this axis
    pub(crate) fn screen_to_normalized(&self, screen_pos: f32, bounds: &Rectangle) -> f32 {
        match self.orientation() {
            Orientation::Horizontal => (screen_pos - bounds.x) / bounds.width,
            Orientation::Vertical => 1.0 - ((screen_pos - bounds.y) / bounds.height),
        }
    }

    pub fn denormalize(&self, normalized: D) -> D {
        self.scale.denormalize(normalized)
    }

    /// Handle drag event on this axis - returns normalized delta
    pub(crate) fn translate_drag_delta(&self, delta: f32, bounds: &Rectangle) -> f32 {
        match self.orientation() {
            Orientation::Horizontal => -delta / bounds.width,
            Orientation::Vertical => delta / bounds.height,
        }
    }

    pub fn layout(&self, limits: &Limits) -> Node {
        let min = limits.min();
        let max = limits.max();

        let thickness = self.thickness().0;

        let size = match self.position {
            // Horizontal: use all available width, choose a height.
            Position::Top | Position::Bottom => {
                let height = thickness.clamp(min.height, max.height).max(0.0);
                Size::new(max.width, height)
            }
            // Vertical: use all available height, choose a width.
            Position::Left | Position::Right => {
                let width = thickness.clamp(min.width, max.width).max(0.0);
                Size::new(width, max.height)
            }
        };

        Node::new(size)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw<Renderer, Theme>(
        &self,
        renderer: &mut Renderer,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
        plot_bounds: &Rectangle,
        mesh_buffer: &mut MeshBuffer,
        viewport: &Rectangle,
    ) where
        Renderer: iced::advanced::Renderer
            + iced::advanced::graphics::mesh::Renderer
            + iced::advanced::text::Renderer<Font = iced::Font>,
        Theme: Catalog,
    {
        let theme = style.axis;
        let bounds = layout.bounds();
        let orientation = Orientation::from(self.position());

        let mut label_candidates = Vec::new();

        // Render tick-related stuff (Axis ticks and grid)
        for tick in self.scale().ticks().into_iter() {
            let pos_norm = self.scale().normalize(&tick.value).to_f32().unwrap();

            let tick_line = self.tick_renderer.as_ref().and_then(|f| {
                f(TickLabelContext {
                    tick,
                    normalized_position: pos_norm,
                    axis_bounds: bounds,
                    orientation,
                })
            });

            if let Some(mut line) = tick_line {
                if let Some(label) = line.label.take() {
                    label_candidates.push(LabelCandidate {
                        tick,
                        normalized_position: pos_norm,
                        label,
                    });
                }
                self.draw_tick_line(&theme, line, &bounds, mesh_buffer, pos_norm);
            }

            if let Some(line) = self.grid_renderer.as_ref().and_then(|f| f(tick)) {
                self.draw_grid_line(style, plot_bounds, line, mesh_buffer, pos_norm);
            }
        }

        self.layout_labels(
            renderer,
            &theme,
            &bounds,
            orientation,
            label_candidates,
            viewport,
        );

        // Combined plot and axis bounds
        let full_bounds = plot_bounds.union(&bounds);

        if self.render_cursor
            && let Some(cursor_pos) = cursor.position_over(full_bounds)
        {
            renderer.start_layer(bounds);
            let cursor_rect = match orientation {
                // Create a vertical rect
                Orientation::Horizontal => Rectangle {
                    x: cursor_pos.x - (theme.cursor.width.0 / 2.0),
                    y: bounds.y,
                    height: bounds.height,
                    width: theme.cursor.width.0,
                },
                // Create a horizontal rect
                Orientation::Vertical => Rectangle {
                    x: bounds.x,
                    y: cursor_pos.y - (theme.cursor.width.0 / 2.0),
                    height: theme.cursor.width.0,
                    width: bounds.width,
                },
            };
            let quad = Quad {
                bounds: cursor_rect,
                ..Default::default()
            };
            renderer.fill_quad(quad, theme.cursor.color);

            if let Some(cursor_formatter) = &self.cursor_formatter
                && let Some(label) = {
                    let position_to_format = match orientation {
                        Orientation::Horizontal => {
                            (cursor_pos.x - plot_bounds.x) / plot_bounds.width
                        }
                        Orientation::Vertical => {
                            1.0 - ((cursor_pos.y - plot_bounds.y) / plot_bounds.height)
                        }
                    };
                    D::from(position_to_format)
                        .and_then(|normalized| self.scale().denormalize_opt(normalized))
                        .and_then(cursor_formatter)
                }
            {
                // Set alignment based on axis position to keep text within bounds
                let (align_x, align_y) = match self.position {
                    Position::Top => (Alignment::Center, Vertical::Top),
                    Position::Bottom => (Alignment::Center, Vertical::Bottom),
                    Position::Left => (Alignment::Left, Vertical::Center),
                    Position::Right => (Alignment::Right, Vertical::Center),
                };

                // Render label on top of cursor position using the label_formatter.
                let text = Text {
                    content: label.content,
                    bounds: bounds.size(),
                    size: label.size,
                    line_height: iced::widget::text::LineHeight::Relative(1.0),
                    font: iced::Font::default(),
                    align_x,
                    align_y,
                    shaping: Shaping::Auto,
                    wrapping: iced::widget::text::Wrapping::None,
                };

                // Position cursor label at cursor position with padding
                let position = match self.position {
                    Position::Top => Point::new(cursor_pos.x, bounds.y + self.label_spacing.0),
                    Position::Bottom => Point::new(
                        cursor_pos.x,
                        bounds.y + bounds.height - self.label_spacing.0,
                    ),
                    Position::Left => Point::new(bounds.x + self.label_spacing.0, cursor_pos.y),
                    Position::Right => {
                        Point::new(bounds.x + bounds.width - self.label_spacing.0, cursor_pos.y)
                    }
                };

                renderer.fill_text(text, position, theme.label_color, bounds);
            };

            renderer.end_layer();
        }
    }

    fn layout_labels<Renderer>(
        &self,
        renderer: &mut Renderer,
        theme: &AxisStyle,
        bounds: &Rectangle,
        orientation: Orientation,
        label_candidates: Vec<LabelCandidate<D>>,
        viewport: &Rectangle,
    ) where
        Renderer: iced::advanced::Renderer
            + iced::advanced::graphics::mesh::Renderer
            + iced::advanced::text::Renderer<Font = iced::Font>,
    {
        let mut accepted: Vec<PlacedLabelInfo<D>> = Vec::new();

        for candidate in label_candidates {
            let Some(resolved) =
                self.resolve_label_candidate::<Renderer>(candidate, bounds, orientation)
            else {
                continue;
            };

            let ResolvedLabelCandidate {
                tick,
                normalized_position,
                bounds: label_bounds,
                paragraph,
                position,
            } = resolved;

            let context = LabelDecisionContext {
                tick,
                normalized_position,
                bounds: label_bounds,
                orientation,
                accepted: &accepted,
            };

            if self.label_policy.should_render(context) {
                renderer.fill_text(
                    paragraph
                        .as_text()
                        .with_content(paragraph.content().to_string()),
                    position,
                    theme.label_color,
                    *viewport,
                );

                accepted.push(PlacedLabelInfo {
                    tick,
                    normalized_position,
                    bounds: label_bounds,
                });
            }
        }
    }

    fn resolve_label_candidate<Renderer>(
        &self,
        candidate: LabelCandidate<D>,
        bounds: &Rectangle,
        orientation: Orientation,
    ) -> Option<ResolvedLabelCandidate<Renderer, D>>
    where
        Renderer: iced::advanced::text::Renderer<Font = iced::Font>,
    {
        let label = candidate.label;
        if label.content.is_empty() {
            return None;
        }

        let (align_x, align_y, position) = match self.position {
            Position::Top => (
                Alignment::Center,
                Vertical::Top,
                Point::new(
                    bounds
                        .width
                        .mul_add(candidate.normalized_position, bounds.x),
                    bounds.y + self.label_spacing.0,
                ),
            ),
            Position::Bottom => (
                Alignment::Center,
                Vertical::Bottom,
                Point::new(
                    bounds
                        .width
                        .mul_add(candidate.normalized_position, bounds.x),
                    bounds.y + bounds.height - self.label_spacing.0,
                ),
            ),
            Position::Left => (
                Alignment::Left,
                Vertical::Center,
                Point::new(
                    bounds.x + self.label_spacing.0,
                    bounds
                        .height
                        .mul_add(1.0 - candidate.normalized_position, bounds.y),
                ),
            ),
            Position::Right => (
                Alignment::Right,
                Vertical::Center,
                Point::new(
                    bounds.x + bounds.width - self.label_spacing.0,
                    bounds
                        .height
                        .mul_add(1.0 - candidate.normalized_position, bounds.y),
                ),
            ),
        };

        let paragraph = Plain::new(Text {
            content: label.content,
            bounds: bounds.size(),
            size: label.size,
            line_height: iced::widget::text::LineHeight::Relative(1.0),
            font: iced::Font::default(),
            align_x,
            align_y,
            shaping: Shaping::Auto,
            wrapping: iced::widget::text::Wrapping::None,
        });

        let text_bounds = paragraph.min_bounds();
        let (start, end) = match orientation {
            Orientation::Horizontal => {
                let center = bounds
                    .width
                    .mul_add(candidate.normalized_position, bounds.x);
                let half = text_bounds.width / 2.0;
                (center - half, center + half)
            }
            Orientation::Vertical => {
                let center = bounds
                    .height
                    .mul_add(1.0 - candidate.normalized_position, bounds.y);
                let half = text_bounds.height / 2.0;
                (center - half, center + half)
            }
        };

        Some(ResolvedLabelCandidate {
            tick: candidate.tick,
            normalized_position: candidate.normalized_position,
            bounds: LabelBounds::new(start, end),
            paragraph,
            position,
        })
    }

    fn draw_tick_line(
        &self,
        theme: &AxisStyle,
        line: TickLine,
        bounds: &Rectangle,
        mesh_buffer: &mut MeshBuffer,
        pos_norm: f32,
    ) {
        // Convert position of tick before rendering as a line to a screen position
        // Round to whole pixels for consistent rendering
        let (x0, y0, x1, y1) = match self.position {
            Position::Bottom => {
                // Vertical line extending downward from top of bounds
                let x = bounds.width.mul_add(pos_norm, bounds.x).round();
                (x, bounds.y, x + line.thickness.0, bounds.y + line.length.0)
            }
            Position::Top => {
                // Vertical line extending upward from bottom of bounds
                let x = bounds.width.mul_add(pos_norm, bounds.x).round();
                (
                    x,
                    bounds.y + bounds.height - line.length.0,
                    x + line.thickness.0,
                    bounds.y + bounds.height,
                )
            }
            Position::Right => {
                // Horizontal line extending rightward from left of bounds
                let y = bounds.height.mul_add(1.0 - pos_norm, bounds.y).round();
                (bounds.x, y, bounds.x + line.length.0, y + line.thickness.0)
            }
            Position::Left => {
                // Horizontal line extending leftward from right of bounds
                let y = bounds.height.mul_add(1.0 - pos_norm, bounds.y).round();
                (
                    bounds.x + bounds.width - line.length.0,
                    y,
                    bounds.x + bounds.width,
                    y + line.thickness.0,
                )
            }
        };

        let color = color::pack(theme.tick_color);
        mesh_buffer.add(
            &[0, 1, 2, 2, 1, 3],
            &[
                SolidVertex2D {
                    position: [x0, y0],
                    color,
                },
                SolidVertex2D {
                    position: [x1, y0],
                    color,
                },
                SolidVertex2D {
                    position: [x0, y1],
                    color,
                },
                SolidVertex2D {
                    position: [x1, y1],
                    color,
                },
            ],
        );
    }

    fn draw_grid_line(
        &self,
        style: &Style,
        bounds: &Rectangle,
        line: GridLine,
        mesh_buffer: &mut MeshBuffer,
        pos_norm: f32,
    ) {
        let orientation = self.orientation();
        let (x0, y0, x1, y1) = match orientation {
            Orientation::Horizontal => {
                let x = bounds.width.mul_add(pos_norm, bounds.x).round();
                (x, bounds.y, x + line.thickness.0, bounds.y + bounds.height)
            }
            Orientation::Vertical => {
                let y = bounds.height.mul_add(1.0 - pos_norm, bounds.y).round();
                (bounds.x, y, bounds.x + bounds.width, y + line.thickness.0)
            }
        };

        let packed = color::pack(style.grid_color);
        mesh_buffer.add(
            &[0, 1, 2, 2, 1, 3],
            &[
                SolidVertex2D {
                    position: [x0, y0],
                    color: packed,
                },
                SolidVertex2D {
                    position: [x1, y0],
                    color: packed,
                },
                SolidVertex2D {
                    position: [x0, y1],
                    color: packed,
                },
                SolidVertex2D {
                    position: [x1, y1],
                    color: packed,
                },
            ],
        );
    }
}
