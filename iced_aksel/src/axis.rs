//! Axis configuration, layout, and rendering logic.
//!
//! This module provides the [`Axis`] struct, which is the core component for defining
//! how data is mapped to screen coordinates and how visual elements (ticks, grids, labels)
//! are rendered.

use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use aksel::{Float, Scale};
use derivative::Derivative;
use iced_core::{
    Background, Border, Color, Layout, Pixels, Point, Rectangle, Shadow, Size, Text,
    alignment::Vertical,
    layout::{Limits, Node},
    mouse::Cursor,
    renderer::Quad,
    text::{Wrapping, paragraph::Plain},
    widget::text::Alignment,
};
use iced_graphics::{color, mesh::SolidVertex2D};

use crate::{
    plot,
    render::MeshBuffer,
    style::{Style, TextStyle},
};

mod cursor;
mod grid;
mod label;
mod position;
mod tick;

// UPDATED: Import specific optimized functions
use crate::render::manual::linear::{
    draw_dashed_line, draw_horizontal_dashed_line, draw_horizontal_line, draw_line_segment,
    draw_vertical_dashed_line, draw_vertical_line,
};
use crate::style::LineStyle;
pub use cursor::*;
pub use grid::*;
pub use label::*;
pub use position::*;
pub use tick::*;

type TickRendererFn<D> = Rc<RefCell<dyn FnMut(TickContext<D>) -> TickResult>>;
type CursorRendererFn<D> = Rc<RefCell<dyn FnMut(D) -> Option<CursorResult>>>;

/// An axis that maps data values to screen coordinates.
///
/// The `Axis` struct is responsible for:
/// 1. Defining the scale (linear, log, etc.) for mapping data to pixels.
/// 2. Configuring visual elements like ticks, grid lines, and labels.
/// 3. Handling layout and rendering of the axis and its interactive cursor.
///
/// # Example
///
/// ```rust
/// use iced_aksel::{Axis, axis::{Position, TickResult}, scale::Linear};
///
/// let axis = Axis::new(Linear::new(0.0, 100.0), Position::Bottom)
///     .with_thickness(40.0)
///     .with_cursor_formatter(|val| Some(format!("{:.1}", val)));
/// ```
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Axis<D> {
    position: Position,
    thickness: Pixels,
    invisible: bool,
    render_cursor: bool,
    render_grid: bool,

    #[derivative(Debug = "ignore")]
    scale: Box<dyn Scale<Domain = D, Normalized = f32>>,
    #[derivative(Debug = "ignore")]
    pub(crate) tick_renderer: Option<TickRendererFn<D>>,
    #[derivative(Debug = "ignore")]
    pub(crate) cursor_renderer: Option<CursorRendererFn<D>>,
    #[derivative(Debug = "ignore")]
    label_policy: LabelPolicy<D>,
}

// Updated to hold the full result, not just string
pub(crate) struct AxisOverlay<D> {
    label_candidates: Vec<LabelCandidate<D>>,
    cursor_state: Option<(Point, CursorResult)>,
    bounds: Rectangle,
    orientation: Orientation,
}

impl<D: Float> AxisOverlay<D> {
    pub fn draw<Renderer>(
        self,
        axis: &Axis<D>,
        renderer: &mut Renderer,
        style: &Style,
        viewport: &Rectangle,
    ) where
        Renderer: plot::Renderer + iced_core::text::Renderer<Font = iced_core::Font>,
    {
        let mut label_candidates = self.label_candidates;
        label_candidates.sort_by_key(|candidate| candidate.priority);

        axis.layout_labels(
            renderer,
            style,
            &self.bounds,
            self.orientation,
            label_candidates,
            viewport,
        );

        if let Some((cursor_pos, result)) = self.cursor_state {
            // Reconstruct paragraph using style from result
            let paragraph = Plain::<Renderer::Paragraph>::new(Text {
                content: result.label.clone(),
                bounds: self.bounds.size(),
                size: result.badge.text_style.size,
                line_height: result.badge.text_style.line_height,
                font: result.badge.text_style.font,
                align_x: Alignment::Left,
                align_y: Vertical::Top,
                shaping: result.badge.text_style.shaping,
                wrapping: Wrapping::None,
            });

            axis.draw_cursor_overlay(
                renderer,
                cursor_pos,
                paragraph,
                result, // Pass the full result object
                self.bounds,
                viewport,
                self.orientation,
                style,
            );
        }
    }
}

impl<D: Float> Axis<D> {
    /// Creates a new `Axis` with the given scale and position.
    ///
    /// By default, the axis will render:
    /// - Major ticks with labels
    /// - Minor ticks (smaller lines)
    /// - Grid lines aligned with major ticks
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
                color: Color::from_rgb(0.4, 0.4, 0.4),
                ..Default::default()
            });

            if ctx.tick.level == 0 {
                result = result.grid_line(GridLine {
                    thickness: 1.0.into(),
                    dashed: false,
                    color: Color::from_rgb(0.2, 0.2, 0.2),
                });
            }

            result
        }));

        // Default Cursor Renderer
        let cursor_renderer = Rc::new(RefCell::new(|_val| None));

        Self {
            position,
            thickness: 50.0.into(),
            render_cursor: true,
            render_grid: true,
            invisible: false,

            scale: Box::new(scale),
            tick_renderer: Some(tick_renderer),
            cursor_renderer: Some(cursor_renderer),
            label_policy: LabelPolicy::default(),
        }
    }

    /// Sets the reserved thickness of the axis in pixels.
    ///
    /// This determines the space reserved for the axis in the chart layout.
    /// Increase this if your labels are being clipped or overlapping with the chart area.
    pub fn with_thickness<P: Into<Pixels>>(mut self, thickness: P) -> Self {
        self.thickness = thickness.into();
        self
    }

    /// Sets a custom renderer for ticks.
    ///
    /// This function gives you full control over which ticks render lines, grids, or labels.
    ///
    /// # Example
    /// ```rust,ignore
    /// axis.with_tick_renderer(|ctx| {
    ///     if ctx.tick.level == 0 {
    ///         TickResult::with_label(format!("{:.1}", ctx.tick.value))
    ///     } else {
    ///         TickResult::default() // Just a line
    ///     }
    /// })
    /// ```
    pub fn with_tick_renderer<F>(mut self, renderer: F) -> Self
    where
        F: FnMut(TickContext<D>) -> TickResult + 'static,
    {
        self.tick_renderer = Some(Rc::new(RefCell::new(renderer)));
        self
    }

    /// Disables grid line rendering for this axis.
    pub const fn without_grid(mut self) -> Self {
        self.render_grid = false;
        self
    }

    /// Configures the axis to skip labels that would overlap.
    ///
    /// `min_gap_px` specifies the minimum distance in pixels required between labels.
    pub fn skip_overlapping_labels(mut self, min_gap_px: f32) -> Self {
        self.label_policy = LabelPolicy::skip_overlapping(min_gap_px);
        self
    }

    /// Sets a custom policy for determining which labels to render.
    ///
    /// Useful for advanced collision detection or custom filtering logic.
    pub fn with_custom_label_policy<F>(mut self, policy: F) -> Self
    where
        F: for<'a> Fn(LabelDecisionContext<'a, D>) -> LabelDecision + 'static,
    {
        self.label_policy = LabelPolicy::custom(policy);
        self
    }

    /// Sets the formatter for the interactive cursor badge.
    ///
    /// If not set, the cursor badge will not be rendered.
    /// The closure receives the data value at the cursor position and returns the string to display.
    pub fn with_cursor_renderer<F>(mut self, renderer: F) -> Self
    where
        F: FnMut(D) -> Option<CursorResult> + 'static,
    {
        self.cursor_renderer = Some(Rc::new(RefCell::new(renderer)));
        self
    }

    /// Makes the axis invisible.
    ///
    /// It will still occupy layout space (defined by `thickness`) but will not render
    /// any ticks, lines, or labels. To remove it from layout entirely, set thickness to 0.
    pub const fn invisible(mut self) -> Self {
        self.invisible = true;
        self
    }

    /// Updates the tick renderer in-place.
    pub fn set_tick_renderer<F>(&mut self, renderer: F)
    where
        F: Fn(TickContext<D>) -> TickResult + 'static,
    {
        self.tick_renderer = Some(Rc::new(RefCell::new(renderer)));
    }

    /// Sets the visibility of the axis.
    pub const fn set_visibility(&mut self, visible: bool) {
        self.invisible = !visible;
    }

    /// Updates the thickness of the axis in-place.
    pub fn set_thickness<P: Into<Pixels>>(&mut self, thickness: P) {
        self.thickness = thickness.into();
    }

    /// Returns true if the axis is currently visible.
    pub const fn is_visible(&self) -> bool {
        !self.invisible
    }

    /// Returns the data domain (min, max) of the axis.
    pub fn domain(&self) -> (&D, &D) {
        self.scale.domain()
    }

    /// Returns the layout position of the axis.
    pub const fn position(&self) -> &Position {
        &self.position
    }

    /// Returns the orientation (Horizontal/Vertical) based on the position.
    pub fn orientation(&self) -> Orientation {
        Orientation::from(&self.position)
    }

    /// Returns the current thickness of the axis.
    pub const fn thickness(&self) -> Pixels {
        if self.invisible {
            return Pixels(0.0);
        }
        self.thickness
    }

    /// Converts a screen coordinate to a normalized value (0.0 - 1.0).
    pub(crate) fn screen_to_normalized(&self, screen_pos: f32, bounds: &Rectangle) -> f32 {
        match self.orientation() {
            Orientation::Horizontal => (screen_pos - bounds.x) / bounds.width,
            Orientation::Vertical => 1.0 - ((screen_pos - bounds.y) / bounds.height),
        }
    }

    /// Converts a drag delta in pixels to a normalized delta.
    ///
    /// This handles the inversion of Y-axis coordinates automatically.
    pub(crate) fn translate_drag_delta(&self, delta: f32, bounds: &Rectangle) -> f32 {
        match self.orientation() {
            Orientation::Horizontal => -delta / bounds.width,
            Orientation::Vertical => delta / bounds.height,
        }
    }

    /// Calculates the layout node for this axis.
    pub(crate) fn layout(&self, limits: &Limits) -> Node {
        let min = limits.min();
        let max = limits.max();

        let thickness = self.thickness().0;

        let size = match self.position {
            Position::Top | Position::Bottom => {
                let height = thickness.clamp(min.height, max.height).max(0.0);
                Size::new(max.width, height)
            }
            Position::Left | Position::Right => {
                let width = thickness.clamp(min.width, max.width).max(0.0);
                Size::new(width, max.height)
            }
        };

        Node::new(size)
    }

    pub(crate) fn draw_grid(
        &self,
        layout: Layout<'_>,
        plot_bounds: &Rectangle,
        mesh_buffer: &mut MeshBuffer,
    ) {
        if !self.render_grid || (self.invisible && !self.render_grid) {
            return;
        }

        let bounds = layout.bounds();
        let orientation = Orientation::from(self.position());
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
                grid_line: Some(line),
                ..
            }) = tick_result
            {
                self.draw_grid_line(&layout.bounds(), plot_bounds, line, mesh_buffer, pos_norm);
            }
        }
    }

    pub(crate) fn draw_ticks(
        &self,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
        plot_bounds: &Rectangle,
        mesh_buffer: &mut MeshBuffer,
    ) -> AxisOverlay<D> {
        let bounds = layout.bounds();
        let orientation = Orientation::from(self.position());

        if self.invisible {
            return AxisOverlay {
                label_candidates: Vec::new(),
                cursor_state: None,
                bounds,
                orientation,
            };
        }

        let (&d_min, &d_max) = self.scale.domain();
        let full_bounds = plot_bounds.union(&bounds);

        let cursor_state = if self.render_cursor
            && let Some(cursor_pos) = cursor.position_over(full_bounds)
            && let Some(cursor_renderer) = &self.cursor_renderer
        {
            let value_to_render = match orientation {
                Orientation::Horizontal => (cursor_pos.x - plot_bounds.x) / plot_bounds.width,
                Orientation::Vertical => {
                    1.0 - ((cursor_pos.y - plot_bounds.y) / plot_bounds.height)
                }
            };

            self.denormalize_opt(value_to_render)
                .and_then(|val| cursor_renderer.borrow_mut()(val))
                .map(|result| (cursor_pos, result))
        } else {
            None
        };

        let mut label_candidates = Vec::new();

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

            let Some(TickResult {
                tick_line,
                label,
                label_style,
                label_priority,
                ..
            }) = tick_result
            else {
                continue;
            };

            if let Some(label) = label {
                label_candidates.push(LabelCandidate {
                    tick,
                    normalized_position: pos_norm,
                    label,
                    style: label_style.unwrap_or(TextStyle::new(12., Color::WHITE)),
                    priority: label_priority.unwrap_or(tick.level),
                });
            }

            if let Some(line) = tick_line {
                self.draw_tick_line(line, &bounds, mesh_buffer, pos_norm);
            }
        }

        self.draw_axis_spine(&style.axis.line, &bounds, mesh_buffer);

        AxisOverlay {
            label_candidates,
            cursor_state,
            bounds,
            orientation,
        }
    }

    fn draw_cursor_overlay<Renderer>(
        &self,
        renderer: &mut Renderer,
        cursor_pos: Point,
        paragraph: Plain<Renderer::Paragraph>,
        cursor_result: CursorResult,
        bounds: Rectangle,
        viewport: &Rectangle,
        orientation: Orientation,
        style: &Style,
    ) where
        Renderer: plot::Renderer + iced_core::text::Renderer<Font = iced_core::Font>,
    {
        // 1. Calculate the fixed "Rail" position (same as tick labels)
        let rail_pos = self.calculate_rail_position(&bounds, orientation, style.axis.text_offset);
        let min_bounds = paragraph.min_bounds();
        let padding = cursor_result.badge.padding;

        // 2. Determine Text Origin (independent of padding!)
        // This ensures the text baseline aligns perfectly with tick labels.
        let text_origin = match orientation {
            Orientation::Horizontal => {
                let x = cursor_pos.x - (min_bounds.width / 2.0);
                let y = match self.position {
                    // Top: Text sits ON TOP of the rail (bottom of text at rail)
                    Position::Top => rail_pos - min_bounds.height,
                    // Bottom: Text hangs BELOW the rail (top of text at rail)
                    _ => rail_pos,
                };
                Point::new(x, y)
            }
            Orientation::Vertical => {
                let y = cursor_pos.y - (min_bounds.height / 2.0);
                let x = match self.position {
                    // Right: Text sits to the RIGHT of rail (left of text at rail)
                    Position::Right => rail_pos,
                    // Left: Text sits to the LEFT of rail (right of text at rail)
                    _ => rail_pos - min_bounds.width,
                };
                Point::new(x, y)
            }
        };

        // 3. Calculate Badge Rect relative to the fixed Text Origin
        // Padding simply expands the box outwards, without shifting the text.
        let mut badge_rect = Rectangle {
            x: text_origin.x - padding.left,
            y: text_origin.y - padding.top,
            width: min_bounds.width + padding.left + padding.right,
            height: min_bounds.height + padding.top + padding.bottom,
        };

        // 4. Clamp to Viewport (optional, keeps badge on screen)
        match orientation {
            Orientation::Horizontal => {
                if badge_rect.x < viewport.x {
                    // Shift both badge AND text if we hit screen edge
                    let shift = viewport.x - badge_rect.x;
                    badge_rect.x += shift;
                    // Note: We only shift text if we are clamping to screen edges
                    // text_origin.x += shift;
                } else if badge_rect.x + badge_rect.width > viewport.x + viewport.width {
                    let shift = (badge_rect.x + badge_rect.width) - (viewport.x + viewport.width);
                    badge_rect.x -= shift;
                }
            }
            Orientation::Vertical => {
                if badge_rect.y < viewport.y {
                    let shift = viewport.y - badge_rect.y;
                    badge_rect.y += shift;
                } else if badge_rect.y + badge_rect.height > viewport.y + viewport.height {
                    let shift = (badge_rect.y + badge_rect.height) - (viewport.y + viewport.height);
                    badge_rect.y -= shift;
                }
            }
        }

        // Re-calculate text position based on potentially clamped badge rect
        let text_pos = Point::new(badge_rect.x + padding.left, badge_rect.y + padding.top);

        // 5. Draw Cursor Line
        // The line connects the axis to the badge. The gap is applied here.
        let gap = cursor_result.line.gap.0;
        let cursor_line_width = cursor_result.line.width.0;
        let cursor_line_color = cursor_result.line.color;

        let cursor_line_rect = match orientation {
            Orientation::Horizontal => {
                let (y_start, y_end) = match self.position {
                    Position::Top => {
                        let line_start = bounds.y + bounds.height;
                        // Line goes from axis up to (badge_bottom + gap)
                        let line_end = (badge_rect.y + badge_rect.height + gap).min(line_start);
                        (line_end, line_start)
                    }
                    _ => {
                        let line_start = bounds.y;
                        // Line goes from axis down to (badge_top - gap)
                        let line_end = (badge_rect.y - gap).max(line_start);
                        (line_start, line_end)
                    }
                };

                Rectangle {
                    x: cursor_pos.x - (cursor_line_width / 2.0),
                    y: y_start.min(y_end),
                    width: cursor_line_width.into(),
                    height: (y_end - y_start).abs(),
                }
            }
            Orientation::Vertical => {
                let (x_start, x_end) = match self.position {
                    Position::Right => {
                        let line_start = bounds.x;
                        let line_end = (badge_rect.x - gap).max(line_start);
                        (line_start, line_end)
                    }
                    _ => {
                        let line_start = bounds.x + bounds.width;
                        let line_end = (badge_rect.x + badge_rect.width + gap).min(line_start);
                        (line_end, line_start)
                    }
                };

                Rectangle {
                    x: x_start.min(x_end),
                    y: cursor_pos.y - (cursor_line_width / 2.0),
                    width: (x_end - x_start).abs(),
                    height: cursor_line_width.into(),
                }
            }
        };

        // ... Standard Rendering Code (same as before) ...
        let border = cursor_result.badge.border.unwrap_or_default();
        let background = cursor_result
            .badge
            .background
            .unwrap_or(Background::Color(Color::TRANSPARENT));
        let shadow = cursor_result.badge.shadow.unwrap_or_default();

        renderer.start_layer(*viewport);

        renderer.fill_quad(
            Quad {
                bounds: cursor_line_rect,
                ..Default::default()
            },
            cursor_line_color,
        );

        renderer.fill_quad(
            Quad {
                bounds: badge_rect,
                border,
                shadow,
                ..Default::default()
            },
            background,
        );

        renderer.fill_text(
            paragraph
                .as_text()
                .with_content(paragraph.content().to_string()),
            text_pos,
            cursor_result.badge.text_style.color,
            *viewport,
        );

        renderer.end_layer();
    }

    fn calculate_rail_position(
        &self,
        bounds: &Rectangle,
        _orientation: Orientation,
        offset: Pixels,
    ) -> f32 {
        match self.position {
            Position::Bottom => bounds.y + offset.0,
            Position::Top => (bounds.y + bounds.height) - offset.0,
            Position::Left => (bounds.x + bounds.width) - offset.0,
            Position::Right => bounds.x + offset.0,
        }
    }

    fn layout_labels<Renderer>(
        &self,
        renderer: &mut Renderer,
        style: &Style,
        bounds: &Rectangle,
        orientation: Orientation,
        label_candidates: Vec<LabelCandidate<D>>,
        viewport: &Rectangle,
    ) where
        Renderer: plot::Renderer + iced_core::text::Renderer<Font = iced_core::Font>,
    {
        let mut accepted: Vec<PlacedLabelInfo<D>> = Vec::new();

        for candidate in label_candidates {
            let Some(resolved) = self.resolve_label_candidate(
                candidate,
                bounds,
                orientation,
                style.axis.text_offset,
            ) else {
                continue;
            };

            let ResolvedLabelCandidate {
                tick,
                normalized_position,
                bounds: label_bounds,
                paragraph,
                position,
                color,
            }: ResolvedLabelCandidate<Renderer, _> = resolved;

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
                    color,
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
        offset: Pixels,
    ) -> Option<ResolvedLabelCandidate<Renderer, D>>
    where
        Renderer: iced_core::text::Renderer<Font = iced_core::Font>,
    {
        let label_content = candidate.label;
        if label_content.is_empty() {
            return None;
        }

        if candidate.normalized_position.is_sign_negative() {
            return None;
        }

        let text_style = candidate.style;

        let paragraph = Plain::new(Text {
            content: label_content,
            bounds: bounds.size(),
            size: text_style.size,
            line_height: text_style.line_height,
            font: text_style.font,
            align_x: Alignment::Left,
            align_y: Vertical::Top,
            shaping: text_style.shaping,
            wrapping: Wrapping::None,
        });

        let text_bounds = paragraph.min_bounds();
        let rail_pos = self.calculate_rail_position(bounds, orientation, offset);

        let position = match self.position {
            Position::Top => {
                let center_x = bounds
                    .width
                    .mul_add(candidate.normalized_position, bounds.x);
                Point::new(
                    center_x - (text_bounds.width / 2.0),
                    rail_pos - text_bounds.height,
                )
            }
            Position::Bottom => {
                let center_x = bounds
                    .width
                    .mul_add(candidate.normalized_position, bounds.x);
                Point::new(center_x - (text_bounds.width / 2.0), rail_pos)
            }
            Position::Left => {
                let center_y = bounds
                    .height
                    .mul_add(1.0 - candidate.normalized_position, bounds.y);
                Point::new(
                    rail_pos - text_bounds.width,
                    center_y - (text_bounds.height / 2.0),
                )
            }
            Position::Right => {
                let center_y = bounds
                    .height
                    .mul_add(1.0 - candidate.normalized_position, bounds.y);
                Point::new(rail_pos, center_y - (text_bounds.height / 2.0))
            }
        };

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
            color: text_style.color,
        })
    }

    fn draw_tick_line(
        &self,
        line: TickLine,
        bounds: &Rectangle,
        mesh_buffer: &mut MeshBuffer,
        pos_norm: f32,
    ) {
        // UPDATED: Use optimized primitives with snap=true for sharp rendering
        match self.position {
            Position::Bottom => {
                let x = bounds.width.mul_add(pos_norm, bounds.x);
                draw_vertical_line(
                    mesh_buffer,
                    x,
                    bounds.y,
                    bounds.y + line.length.0,
                    line.thickness.0,
                    line.color,
                    true,
                );
            }
            Position::Top => {
                let x = bounds.width.mul_add(pos_norm, bounds.x);
                draw_vertical_line(
                    mesh_buffer,
                    x,
                    bounds.y + bounds.height - line.length.0,
                    bounds.y + bounds.height,
                    line.thickness.0,
                    line.color,
                    true,
                );
            }
            Position::Right => {
                let y = bounds.height.mul_add(1.0 - pos_norm, bounds.y);
                draw_horizontal_line(
                    mesh_buffer,
                    bounds.x,
                    bounds.x + line.length.0,
                    y,
                    line.thickness.0,
                    line.color,
                    true,
                );
            }
            Position::Left => {
                let y = bounds.height.mul_add(1.0 - pos_norm, bounds.y);
                draw_horizontal_line(
                    mesh_buffer,
                    bounds.x + bounds.width - line.length.0,
                    bounds.x + bounds.width,
                    y,
                    line.thickness.0,
                    line.color,
                    true,
                );
            }
        };
    }

    fn draw_grid_line(
        &self,
        axis_bounds: &Rectangle,
        plot_bounds: &Rectangle,
        line: GridLine,
        mesh_buffer: &mut MeshBuffer,
        pos_norm: f32,
    ) {
        let orientation = self.orientation();
        let width = line.thickness.0;
        let color = line.color;

        // Note: Dash parameters are hardcoded for now, mimicking original behavior.
        // These could be exposed in GridLine later.
        let dash_len = 5.0;
        let gap_len = 5.0;

        match orientation {
            Orientation::Horizontal => {
                // If axis is horizontal, grid lines are vertical.
                let x = axis_bounds.width.mul_add(pos_norm, axis_bounds.x);
                let y_start = plot_bounds.y;
                let y_end = plot_bounds.y + plot_bounds.height;

                if line.dashed {
                    draw_vertical_dashed_line(
                        mesh_buffer,
                        x,
                        y_start,
                        y_end,
                        width,
                        color,
                        dash_len,
                        gap_len,
                        true,
                    );
                } else {
                    draw_vertical_line(mesh_buffer, x, y_start, y_end, width, color, true);
                }
            }
            Orientation::Vertical => {
                // If axis is vertical, grid lines are horizontal.
                let y = axis_bounds.height.mul_add(1.0 - pos_norm, axis_bounds.y);
                let x_start = plot_bounds.x;
                let x_end = plot_bounds.x + plot_bounds.width;

                if line.dashed {
                    draw_horizontal_dashed_line(
                        mesh_buffer,
                        x_start,
                        x_end,
                        y,
                        width,
                        color,
                        dash_len,
                        gap_len,
                        true,
                    );
                } else {
                    draw_horizontal_line(mesh_buffer, x_start, x_end, y, width, color, true);
                }
            }
        }
    }

    /// Renders the main axis spine.
    ///
    /// The spine grows "inward" into the axis area (based on its thickness) so that it
    /// does not overlap with the plot area.
    fn draw_axis_spine(&self, style: &LineStyle, bounds: &Rectangle, mesh_buffer: &mut MeshBuffer) {
        let thickness = style.width.0;
        let color = style.color;
        // We use half-width offset because draw_*_line centers the line on the coordinate.
        let offset = thickness / 2.0;

        match self.position {
            Position::Bottom => {
                // Spine is at the top edge of the Bottom axis area (y)
                // Growing downwards into the area (y + thickness)
                draw_horizontal_line(
                    mesh_buffer,
                    bounds.x,
                    bounds.x + bounds.width,
                    bounds.y + offset,
                    thickness,
                    color,
                    true,
                );
            }
            Position::Top => {
                // Spine is at the bottom edge of the Top axis area (y + height)
                // Growing upwards
                draw_horizontal_line(
                    mesh_buffer,
                    bounds.x,
                    bounds.x + bounds.width,
                    bounds.y + bounds.height - offset,
                    thickness,
                    color,
                    true,
                );
            }
            Position::Left => {
                // Spine is at the right edge of the Left axis area (x + width)
                // Growing leftwards
                draw_vertical_line(
                    mesh_buffer,
                    bounds.x + bounds.width - offset,
                    bounds.y,
                    bounds.y + bounds.height,
                    thickness,
                    color,
                    true,
                );
            }
            Position::Right => {
                // Spine is at the left edge of the Right axis area (x)
                // Growing rightwards
                draw_vertical_line(
                    mesh_buffer,
                    bounds.x + offset,
                    bounds.y,
                    bounds.y + bounds.height,
                    thickness,
                    color,
                    true,
                );
            }
        };
    }
}

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
