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

    pub(crate) spine_style: Option<LineStyle>,
    pub(crate) text_offset: Pixels,
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

        // Cursor Drawing Logic
        if let Some((cursor_pos, result)) = self.cursor_state {
            // Only prepare text if we have a label and a badge style to render it with
            let paragraph =
                if let (Some(label), Some(badge)) = (&result.label, &result.cursor_badge) {
                    Some(Plain::<Renderer::Paragraph>::new(Text {
                        content: label.clone(),
                        bounds: self.bounds.size(),
                        size: badge.text_style.size,
                        line_height: badge.text_style.line_height,
                        font: badge.text_style.font,
                        align_x: Alignment::Left,
                        align_y: Vertical::Top,
                        shaping: badge.text_style.shaping,
                        wrapping: Wrapping::None,
                    }))
                } else {
                    None
                };

            // We pass the whole result object to draw_cursor_overlay
            // It will check for Some(line), Some(badge), etc. internally.
            axis.draw_cursor_overlay(
                renderer,
                cursor_pos,
                paragraph,
                result,
                self.bounds,
                viewport,
                self.orientation,
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
        Self {
            position,
            thickness: 50.0.into(),
            render_cursor: true,
            render_grid: true,
            invisible: false,

            scale: Box::new(scale),
            tick_renderer: None,
            cursor_renderer: None,
            label_policy: LabelPolicy::default(),

            spine_style: None,
            text_offset: Pixels(6.0),
        }
    }

    /// Sets the `Spine` of the axis. This is the line between the `Axis` and the `Plot`.
    ///
    /// No line will be rendered if this is not set
    pub fn with_spine_style(mut self, style: LineStyle) -> Self {
        self.spine_style = Some(style);
        self
    }

    /// Sets the reserved thickness of the axis in pixels.
    ///
    /// This determines the space reserved for the axis in the chart layout.
    /// Increase this if your labels are being clipped or overlapping with the chart area.
    pub fn with_thickness<P: Into<Pixels>>(mut self, thickness: P) -> Self {
        self.thickness = thickness.into();
        self
    }

    pub fn with_text_offset(mut self, offset: Pixels) -> Self {
        self.text_offset = offset;
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

            if let Some(label_text) = label {
                if let Some(s) = label_style {
                    label_candidates.push(LabelCandidate {
                        tick,
                        normalized_position: pos_norm,
                        text: label_text,
                        // TODO: Im not sure how to default this properly as we will need some styling to go through,
                        // but we never want the styling to be static.
                        style: s,
                        priority: label_priority.unwrap_or(tick.level),
                    });
                }
            }

            if let Some(line) = tick_line {
                self.draw_tick_line(line, &bounds, mesh_buffer, pos_norm);
            }
        }

        if let Some(spine_style) = self.spine_style {
            self.draw_axis_spine(&spine_style, &bounds, mesh_buffer);
        }

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
        paragraph: Option<Plain<Renderer::Paragraph>>,
        cursor_result: CursorResult,
        bounds: Rectangle,
        viewport: &Rectangle,
        orientation: Orientation,
    ) where
        Renderer: plot::Renderer + iced_core::text::Renderer<Font = iced_core::Font>,
    {
        // 1. Calculate Geometry
        // We calculate the badge bounds relative to the cursor.
        // If there is no text (paragraph is None), size is just padding (or 0).
        let padding = cursor_result
            .cursor_badge
            .as_ref()
            .map(|b| b.padding)
            .unwrap_or_default();

        let (content_width, content_height) = if let Some(p) = &paragraph {
            let min = p.min_bounds();
            (min.width, min.height)
        } else {
            (0.0, 0.0)
        };

        let badge_size = Size::new(
            content_width + padding.left + padding.right,
            content_height + padding.top + padding.bottom,
        );

        // Calculate the "Rail" position (where axis text sits).
        // Since we don't have a default style stored on Axis anymore,
        // we use a hardcoded reasonable offset (12.0) for positioning logic.
        // This ensures the line stops at a consistent place.
        let rail_pos = self.calculate_rail_position(&bounds, orientation, Pixels(12.0));

        let (badge_x, badge_y) = match orientation {
            Orientation::Horizontal => {
                let x = cursor_pos.x - (content_width / 2.0) - padding.left;
                let y = match self.position {
                    Position::Top => rail_pos - badge_size.height,
                    _ => rail_pos,
                };
                (x, y)
            }
            Orientation::Vertical => {
                let y = cursor_pos.y - (content_height / 2.0) - padding.top;
                let x = match self.position {
                    Position::Left => rail_pos - badge_size.width,
                    _ => rail_pos,
                };
                (x, y)
            }
        };

        let mut badge_rect = Rectangle::new(Point::new(badge_x, badge_y), badge_size);

        // Clamp to viewport
        if orientation == Orientation::Horizontal {
            badge_rect.x = badge_rect
                .x
                .clamp(viewport.x, viewport.x + viewport.width - badge_rect.width);
        } else {
            badge_rect.y = badge_rect
                .y
                .clamp(viewport.y, viewport.y + viewport.height - badge_rect.height);
        }

        renderer.start_layer(*viewport);

        // 2. Draw Cursor Line (If present)
        if let Some(line) = cursor_result.cursor_line {
            let gap = line.gap.0;
            let half_width = line.width.0 / 2.0;

            let line_rect = match orientation {
                Orientation::Horizontal => {
                    let (y_min, y_max) = match self.position {
                        Position::Top => (
                            (badge_rect.y + badge_rect.height + gap).min(bounds.y + bounds.height),
                            bounds.y + bounds.height,
                        ),
                        _ => (bounds.y, (badge_rect.y - gap).max(bounds.y)),
                    };
                    Rectangle::new(
                        Point::new(cursor_pos.x - half_width, y_min),
                        Size::new(line.width.0, y_max - y_min),
                    )
                }
                Orientation::Vertical => {
                    let (x_min, x_max) = match self.position {
                        Position::Right => (bounds.x, (badge_rect.x - gap).max(bounds.x)),
                        _ => (
                            (badge_rect.x + badge_rect.width + gap).min(bounds.x + bounds.width),
                            bounds.x + bounds.width,
                        ),
                    };
                    Rectangle::new(
                        Point::new(x_min, cursor_pos.y - half_width),
                        Size::new(x_max - x_min, line.width.0),
                    )
                }
            };

            renderer.fill_quad(
                Quad {
                    bounds: line_rect,
                    ..Default::default()
                },
                line.color,
            );
        }

        // 3. Draw Badge Background & Text
        // Only proceed if we have text content (paragraph) AND badge settings.
        // If settings are missing, we draw nothing (simple & strict).
        if let (Some(p), Some(badge)) = (paragraph, cursor_result.cursor_badge) {
            // Draw Background
            renderer.fill_quad(
                Quad {
                    bounds: badge_rect,
                    border: badge.border.unwrap_or_default(),
                    shadow: badge.shadow.unwrap_or_default(),
                    ..Default::default()
                },
                badge.background.unwrap_or(Color::TRANSPARENT),
            );

            // Draw Text using the color from the badge settings
            let text_pos = Point::new(badge_rect.x + padding.left, badge_rect.y + padding.top);

            renderer.fill_text(
                p.as_text().with_content(p.content().to_string()),
                text_pos,
                badge.text_style.color,
                *viewport,
            );
        }

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
            let Some(resolved) =
                self.resolve_label_candidate(candidate, bounds, orientation, self.text_offset)
            else {
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
        let label_content = candidate.text;
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
