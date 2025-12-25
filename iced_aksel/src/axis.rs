//! Axis configuration and rendering.
//!
//! This module provides the [`Axis`] type for configuring chart axes, including:
//!
//! - **Scales**: Linear, logarithmic, or custom domain-to-screen mappings
//! - **Position**: Top, bottom, left, or right placement
//! - **Ticks & Labels**: Customizable tick marks and text labels
//! - **Grids**: Optional grid lines extending into the plot area
//! - **Interactivity**: Cursor labels that follow mouse position
//!
//! # Example
//!
//! ```rust
//! use iced_aksel::{Axis, axis, scale::Linear};
//!
//! // Create a horizontal axis at the bottom
//! let x_axis = Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
//!     .with_thickness(50.0)
//!     .with_grid_renderer(|tick| {
//!         if tick.level == 0 {
//!             Some(axis::GridLine { thickness: 1.0.into() })
//!         } else {
//!             None
//!         }
//!     });
//! ```

use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use aksel::{Float, Scale};
use derivative::Derivative;
use iced_core::{
    Layout, Pixels, Point, Rectangle, Size, Text,
    alignment::Vertical,
    layout::{Limits, Node},
    mouse::Cursor,
    renderer::Quad,
    text::{LineHeight, Wrapping, paragraph::Plain},
    widget::text::{Alignment, Shaping},
};
use iced_graphics::{color, mesh::SolidVertex2D};

use crate::{
    plot,
    render::MeshBuffer,
    style::{AxisStyle, Style},
};

mod grid;
mod label;
mod position;
mod tick;

pub use grid::*;
pub use label::*;
pub use position::*;
pub use tick::*;

// TODO: Can we, somehow, refactor out Rc<RefCell<T>>? Or is it okay as it is?
type TickRendererFn<D> = Rc<RefCell<dyn FnMut(TickContext<D>) -> TickResult>>;
type CursorRendererFn<D> = Rc<RefCell<dyn FnMut(D) -> Option<Label>>>;

/// An axis that maps data values to screen coordinates.
///
/// Axes define the data range, screen position, and visual appearance of chart boundaries.
/// They generate ticks, labels, and optional grid lines automatically based on the configured scale.
///
/// # Example
///
/// ```rust
/// use iced_aksel::{Axis, axis, scale::Linear};
///
/// // Bottom X axis from 0 to 100
/// let x_axis = Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
///     .with_thickness(50.0);
///
/// // Left Y axis from -50 to 50, invisible axes but grid visible
/// let y_axis = Axis::new(Linear::new(-50.0, 50.0), axis::Position::Left)
///     .invisible()
///     .with_grid_renderer(|tick| {
///         if tick.level == 0 {
///             Some(axis::GridLine { thickness: 1.0.into() })
///         } else {
///             None
///         }
///     });
/// ```
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Axis<D> {
    position: Position,
    thickness: Pixels,
    invisible: bool,
    render_cursor: bool,
    render_grid: bool,
    label_spacing: Pixels,

    #[derivative(Debug = "ignore")]
    scale: Box<dyn Scale<Domain = D, Normalized = f32>>,
    #[derivative(Debug = "ignore")]
    pub(crate) tick_renderer: Option<TickRendererFn<D>>,
    #[derivative(Debug = "ignore")]
    pub(crate) cursor_formatter: Option<CursorRendererFn<D>>,
    #[derivative(Debug = "ignore")]
    label_policy: LabelPolicy<D>,
}

impl<D: Float> Axis<D> {
    /// Creates a new axis with the given scale and position.
    ///
    /// # Example
    ///
    /// ```rust
    /// use iced_aksel::{Axis, axis, scale::Linear};
    ///
    /// let axis = Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom);
    /// ```
    pub fn new(
        scale: impl Scale<Domain = D, Normalized = f32> + 'static,
        position: Position,
    ) -> Self {
        let tick_renderer = Rc::new(RefCell::new(|ctx: TickContext<D>| {
            let mut result = TickResult::with_tick_line(TickLine {
                length: match ctx.tick.level {
                    0 => 10.0,
                    _ => 5.0,
                }
                .into(),
                ..Default::default()
            });

            if ctx.tick.level == 0 {
                result = result.grid_line(GridLine {
                    thickness: 1.0.into(),
                });
            }

            result
        }));

        Self {
            position,
            thickness: 50.0.into(),
            render_cursor: true,
            render_grid: true,
            invisible: false,
            label_spacing: 5.0.into(),

            scale: Box::new(scale),
            tick_renderer: Some(tick_renderer),
            cursor_formatter: None,
            label_policy: LabelPolicy::default(),
        }
    }

    // =====================================
    // BUILDERS
    // =====================================

    /// Sets the spacing between labels and tick lines.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::{Axis, axis, scale::Linear};
    /// let axis = Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
    ///     .with_label_spacing(10.0);
    /// ```
    pub fn with_label_spacing<P: Into<Pixels>>(mut self, spacing: P) -> Self {
        self.label_spacing = spacing.into();
        self
    }

    /// Sets the thickness of the axis.
    ///
    /// For horizontal axes (Top/Bottom), this is the height.
    /// For vertical axes (Left/Right), this is the width.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::{Axis, axis, scale::Linear};
    /// let axis = Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
    ///     .with_thickness(60.0);
    /// ```
    pub fn with_thickness<P: Into<Pixels>>(mut self, thickness: P) -> Self {
        self.thickness = thickness.into();
        self
    }

    /// Sets a custom tick renderer.
    ///
    /// The renderer receives tick context and returns `Some(TickLine)` with optional label,
    /// or `None` to hide that tick entirely.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::{Axis, axis::{self, TickLine}, scale::Linear};
    /// let axis = Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
    ///     .with_tick_renderer(|ctx| {
    ///         Some(TickLine::simple(format!("{:.0}", ctx.tick.value)))
    ///     });
    /// ```
    pub fn with_tick_renderer<F>(mut self, renderer: F) -> Self
    where
        F: FnMut(TickContext<D>) -> TickResult + 'static,
    {
        self.tick_renderer = Some(Rc::new(RefCell::new(renderer)));
        self
    }

    /// Removes grid lines from this axis.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::{Axis, axis, scale::Linear};
    /// let axis = Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
    ///     .without_grid();
    /// ```
    pub const fn without_grid(mut self) -> Self {
        self.render_grid = false;
        self
    }

    /// Enables automatic label overlap detection and skipping.
    ///
    /// Labels that would overlap with previously placed labels within `min_gap_px`
    /// pixels will be hidden.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::{Axis, axis, scale::Linear};
    /// let axis = Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
    ///     .skip_overlapping_labels(10.0);
    /// ```
    pub fn skip_overlapping_labels(mut self, min_gap_px: f32) -> Self {
        self.label_policy = LabelPolicy::skip_overlapping(min_gap_px);
        self
    }

    /// Sets a custom label rendering policy.
    ///
    /// The policy function receives label context and returns [`LabelDecision::Render`]
    /// or [`LabelDecision::Skip`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::{Axis, axis::{self, LabelDecision}, scale::Linear};
    /// let axis = Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
    ///     .with_custom_label_policy(|ctx| {
    ///         // Only show even numbers
    ///         if ctx.tick.value as i32 % 2 == 0 {
    ///             LabelDecision::Render
    ///         } else {
    ///             LabelDecision::Skip
    ///         }
    ///     });
    /// ```
    pub fn with_custom_label_policy<F>(mut self, policy: F) -> Self
    where
        F: for<'a> Fn(LabelDecisionContext<'a, D>) -> LabelDecision + 'static,
    {
        self.label_policy = LabelPolicy::custom(policy);
        self
    }

    /// Sets a formatter for the cursor label that follows the mouse.
    ///
    /// When the mouse hovers over the plot, this formatter is called with the data value
    /// under the cursor. Return `Some(Label)` to show a label, or `None` to hide it.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::{Axis, axis, scale::Linear};
    /// let axis = Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
    ///     .with_cursor_formatter(|value| {
    ///         Some(axis::Label {
    ///             content: format!("{:.2}", value),
    ///             size: 12.into(),
    ///             ..Default::default()
    ///         })
    ///     });
    /// ```
    pub fn with_cursor_formatter<F>(mut self, renderer: F) -> Self
    where
        F: FnMut(D) -> Option<Label> + 'static,
    {
        self.cursor_formatter = Some(Rc::new(RefCell::new(renderer)));
        self
    }

    /// Makes this axis invisible (no ticks, labels, or visual elements).
    ///
    /// The axis still defines the coordinate system and can render grid lines.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::{Axis, axis, scale::Linear};
    /// let axis = Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
    ///     .invisible();
    /// ```
    pub const fn invisible(mut self) -> Self {
        self.invisible = true;
        self
    }

    // =====================================
    // RUNTIME SETTERS
    // =====================================

    /// Changes the tick renderer for the axis. This defines which ticks should have lines and
    /// are allowed to have labels & grid lines.
    pub fn set_tick_renderer<F>(&mut self, renderer: F)
    where
        F: Fn(TickContext<D>) -> TickResult + 'static,
    {
        self.tick_renderer = Some(Rc::new(RefCell::new(renderer)));
    }

    /// Sets the axis as visible.
    pub const fn set_visibility(&mut self, visible: bool) {
        self.invisible = !visible;
    }

    /// Sets the thickness of the axis. This is the thickness of the axis in the direction
    /// perpendicular to the chart.
    ///
    /// Top/Bottom -> height
    /// Left/Right -> width
    ///
    /// For now it's a placeholder; customize this later using
    /// `tick_renderer`, fonts, padding, etc.
    pub fn set_thickness<P: Into<Pixels>>(&mut self, thickness: P) {
        self.thickness = thickness.into();
    }

    // =====================================
    // GETTERS
    // =====================================

    /// Checks if the axis is visible.
    pub const fn is_visible(&self) -> bool {
        !self.invisible
    }

    /// Gets the domain of the axis.
    pub fn domain(&self) -> (&D, &D) {
        self.scale.domain()
    }

    /// Gets the position of the axis. Which side is the axis wanting to be placed at for the chart?
    pub const fn position(&self) -> &Position {
        &self.position
    }

    /// Gets the orientation of the axis.
    pub fn orientation(&self) -> Orientation {
        Orientation::from(&self.position)
    }

    /// How thick this axis wants to be in the direction
    /// perpendicular to the chart.
    ///
    /// Top/Bottom -> height
    /// Left/Right -> width
    pub const fn thickness(&self) -> Pixels {
        if self.invisible {
            return Pixels(0.0);
        }
        self.thickness
    }

    // =====================================
    // CRATE / INTERNAL
    // =====================================

    /// Convert a screen position to a normalized value (0.0-1.0) along this axis
    pub(crate) fn screen_to_normalized(&self, screen_pos: f32, bounds: &Rectangle) -> f32 {
        match self.orientation() {
            Orientation::Horizontal => (screen_pos - bounds.x) / bounds.width,
            Orientation::Vertical => 1.0 - ((screen_pos - bounds.y) / bounds.height),
        }
    }

    /// Handle drag event on this axis - returns normalized delta
    pub(crate) fn translate_drag_delta(&self, delta: f32, bounds: &Rectangle) -> f32 {
        match self.orientation() {
            Orientation::Horizontal => -delta / bounds.width,
            Orientation::Vertical => delta / bounds.height,
        }
    }

    pub(crate) fn layout(&self, limits: &Limits) -> Node {
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
    pub(crate) fn draw<Renderer>(
        &self,
        renderer: &mut Renderer,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
        plot_bounds: &Rectangle,
        mesh_buffer: &mut MeshBuffer,
        viewport: &Rectangle,
    ) where
        Renderer: plot::Renderer,
    {
        if self.invisible && !self.render_grid {
            return; // We don't need to render anything
        }

        let theme = style.axis;
        let bounds = layout.bounds();
        let full_bounds = plot_bounds.union(&bounds);
        let orientation = Orientation::from(self.position());

        let mut label_candidates = Vec::new();

        // Render tick-related stuff (Axis ticks and grid)
        let (&d_min, &d_max) = self.scale.domain();

        for tick in self.ticks().into_iter() {
            let pos_norm = self.normalize(&tick.value);

            let tick_result = self.tick_renderer.as_ref().map(|renderer| {
                renderer.borrow_mut()(TickContext {
                    tick,
                    normalized_position: pos_norm,
                    axis_bounds: bounds,
                    scale_domain: (d_max, d_min),
                    orientation,
                })
            });

            if let Some(TickResult {
                mut tick_line,
                mut grid_line,
                mut label,
                label_priority,
            }) = tick_result
            {
                if self.is_visible() {
                    if let Some(label) = label.take() {
                        label_candidates.push(LabelCandidate {
                            tick,
                            normalized_position: pos_norm,
                            label,
                            priority: label_priority.unwrap_or(tick.level),
                        });
                    }

                    if let Some(line) = tick_line.take() {
                        self.draw_tick_line(&theme, line, &bounds, mesh_buffer, pos_norm);
                    }
                }

                if self.render_grid
                    && let Some(line) = grid_line.take()
                {
                    self.draw_grid_line(style, plot_bounds, line, mesh_buffer, pos_norm);
                }
            }
        }

        // We can early return here if the axis is invisible, as we would've already rendered all
        // the gridlines
        if self.invisible {
            return;
        }

        // Sort so the lowest tick levels (major) get processed first - E.g. they have higher
        // priority
        label_candidates.sort_by_key(|candidate| candidate.priority);

        self.layout_labels(
            renderer,
            &theme,
            &bounds,
            orientation,
            label_candidates,
            viewport,
        );

        if self.render_cursor
            && let Some(cursor_pos) = cursor.position_over(full_bounds)
            && let Some(cursor_renderer) = &self.cursor_formatter
        {
            renderer.start_layer(bounds);

            let mut cursor_rect = match orientation {
                // Create a vertical rect
                Orientation::Horizontal => Rectangle {
                    x: cursor_pos.x - (theme.cursor.base_width.0 / 2.0),
                    y: bounds.y,
                    height: bounds.height,
                    width: theme.cursor.base_width.0,
                },
                // Create a horizontal rect
                Orientation::Vertical => Rectangle {
                    x: bounds.x,
                    y: cursor_pos.y - (theme.cursor.base_width.0 / 2.0),
                    height: theme.cursor.base_width.0,
                    width: bounds.width,
                },
            };

            if let Some(label) = {
                let value_to_render = match orientation {
                    Orientation::Horizontal => (cursor_pos.x - plot_bounds.x) / plot_bounds.width,
                    Orientation::Vertical => {
                        1.0 - ((cursor_pos.y - plot_bounds.y) / plot_bounds.height)
                    }
                };

                self.denormalize_opt(value_to_render)
                    .and_then(|pos| cursor_renderer.borrow_mut()(pos))
            } {
                // Set alignment based on axis position to keep text within bounds
                let (align_x, align_y) = match self.position {
                    Position::Top => (Alignment::Center, Vertical::Top),
                    Position::Bottom => (Alignment::Center, Vertical::Bottom),
                    Position::Left => (Alignment::Left, Vertical::Center),
                    Position::Right => (Alignment::Right, Vertical::Center),
                };

                // Render label on top of cursor position using the label_formatter.
                let text = Plain::<Renderer::Paragraph>::new(Text {
                    content: label.content,
                    bounds: bounds.size(),
                    size: label.size,
                    line_height: LineHeight::Relative(1.0),
                    font: renderer.default_font(),
                    align_x,
                    align_y,
                    shaping: Shaping::Auto,
                    wrapping: Wrapping::None,
                });

                let min_bounds = text.min_bounds();

                // Resize cursor rect to fit text
                let horizontal_padding = label.padding.right + label.padding.left;
                let vertical_padding = label.padding.top + label.padding.bottom;
                let wanted_width = min_bounds.width + horizontal_padding;
                let wanted_height = min_bounds.height + vertical_padding;
                cursor_rect.width = wanted_width.clamp(0.0, bounds.width);
                cursor_rect.height = wanted_height.clamp(0.0, bounds.height);

                // Position cursor label at cursor position with padding and position cursor rect
                // properly
                let position = match self.position {
                    Position::Top => {
                        let pos = Point::new(cursor_pos.x, bounds.y + self.label_spacing.0);
                        cursor_rect.x = cursor_pos.x - (cursor_rect.width / 2.0);
                        cursor_rect.y = pos.y - label.padding.top;
                        pos
                    }
                    Position::Bottom => {
                        let pos = Point::new(
                            cursor_pos.x,
                            bounds.y + bounds.height - self.label_spacing.0,
                        );
                        cursor_rect.x = cursor_pos.x - (cursor_rect.width / 2.0);
                        cursor_rect.y = pos.y - cursor_rect.height + label.padding.top;
                        pos
                    }
                    Position::Left => {
                        let pos = Point::new(bounds.x + self.label_spacing.0, cursor_pos.y);
                        cursor_rect.x = pos.x - label.padding.left;
                        cursor_rect.y = cursor_pos.y - (cursor_rect.height / 2.0);
                        pos
                    }
                    Position::Right => {
                        let pos = Point::new(
                            bounds.x + bounds.width - self.label_spacing.0,
                            cursor_pos.y,
                        );
                        cursor_rect.x = pos.x - cursor_rect.width + label.padding.left;
                        cursor_rect.y = cursor_pos.y - (cursor_rect.height / 2.0);
                        pos
                    }
                };

                renderer.fill_text(
                    text.as_text().with_content(text.content().to_string()),
                    position,
                    theme.label_color,
                    bounds,
                );
            }

            let quad = Quad {
                bounds: cursor_rect,
                border: theme.cursor.border,
                shadow: theme.cursor.shadow,
                ..Default::default()
            };

            renderer.fill_quad(quad, theme.cursor.color);

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
        Renderer: plot::Renderer,
    {
        let mut accepted: Vec<PlacedLabelInfo<D>> = Vec::new();

        for candidate in label_candidates {
            let Some(resolved) =
                self.resolve_label_candidate(renderer, candidate, bounds, orientation)
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
        renderer: &Renderer,
        candidate: LabelCandidate<D>,
        bounds: &Rectangle,
        orientation: Orientation,
    ) -> Option<ResolvedLabelCandidate<Renderer, D>>
    where
        Renderer: iced_core::text::Renderer,
    {
        let label = candidate.label;
        if label.content.is_empty() {
            return None;
        }

        if candidate.normalized_position.is_sign_negative() {
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
            line_height: LineHeight::Relative(1.0),
            font: renderer.default_font(),
            align_x,
            align_y,
            shaping: Shaping::Auto,
            wrapping: Wrapping::None,
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

// Deref to `Scale` for ease of use
impl<D: Float> Deref for Axis<D> {
    type Target = dyn Scale<Domain = D, Normalized = f32>;

    fn deref(&self) -> &Self::Target {
        &*self.scale
    }
}
impl<D: Float> DerefMut for Axis<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.scale
    }
}
