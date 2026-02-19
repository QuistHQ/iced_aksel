//! Axis configuration, layout, and rendering logic.
//!
//! This module provides the [`Axis`] struct, which is the core component for defining
//! how data is mapped to screen coordinates and how visual elements (ticks, grids, labels)
//! are rendered.

use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

use aksel::{Float, Scale};
use derivative::Derivative;
use iced_core::{
    Layout, Pixels, Point, Rectangle, Size, Text,
    alignment::Vertical,
    layout::{Limits, Node},
    renderer::Quad,
    text::{Wrapping, paragraph::Plain},
    widget::text::Alignment,
};

use crate::{
    render::{Primitive, RenderCache},
    stroke::{ResolvedStroke, StrokeStyle},
    style::{AxisStyle, DashStyle, Style},
};

mod grid;
mod label;
mod marker;
mod position;
mod tick;

use crate::style::SpineStyle;
pub use grid::*;
pub use label::*;
pub use marker::*;
pub use position::*;
pub use tick::*;

type TickRendererFn<D, Theme> = RefCell<Box<dyn FnMut(TickContext<D, Theme>) -> TickResult>>;
type StyleOverrideFn = RefCell<Box<dyn FnMut(&mut AxisStyle)>>;

/// An axis that maps data values to screen coordinates.
///
/// The `Axis` struct is responsible for:
/// 1. Defining the scale (linear, log, etc.) for mapping data to pixels.
/// 2. Configuring visual elements like ticks, grid lines, and labels.
/// 3. Handling layout and rendering of the axis and its interactive marker.
///
/// # Example
///
/// ```rust
/// use iced_aksel::{Axis, axis::{Position, TickResult, TickContext}, scale::Linear};
///
/// let axis = Axis::new(Linear::new(0.0, 100.0), Position::Bottom)
///     .with_thickness(40.0)
///     .with_tick_renderer(|ctx: TickContext<f64>| TickResult {
///         tick_line: Some(ctx.tickline()),
///         grid_line: Some(ctx.gridline()),
///         label: Some(ctx.label(format!("{:.2}", ctx.tick.value))),
///         ..Default::default()
///     });
/// ```
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Axis<D, Theme = iced_core::Theme> {
    position: Position,
    thickness: Pixels,
    invisible: bool,
    render_grid: bool,

    #[derivative(Debug = "ignore")]
    scale: Box<dyn Scale<Domain = D, Normalized = f32>>,
    #[derivative(Debug = "ignore")]
    tick_renderer: Option<TickRendererFn<D, Theme>>,
    #[derivative(Debug = "ignore")]
    style_override: Option<StyleOverrideFn>,

    #[derivative(Debug = "ignore")]
    label_policy: LabelPolicy<D>,
}

impl<D: Float, Theme> Axis<D, Theme> {
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
        // Default tick renderer: major ticks get grid lines and long marks; minor ticks get short marks.
        let tick_renderer = RefCell::new(Box::new(|ctx: TickContext<D, Theme>| {
            let mut tickline = ctx.tickline();
            tickline.length = match ctx.tick.level {
                0 => 10.0,
                _ => 5.0,
            }
            .into();

            let mut result = TickResult {
                tick_line: Some(tickline),
                ..Default::default()
            };

            if ctx.tick.level == 0 {
                result.grid_line = Some(ctx.gridline());
            }

            result
        }));

        Self {
            position,
            thickness: 50.0.into(),
            render_grid: true,
            invisible: false,

            scale: Box::new(scale),
            tick_renderer: Some(tick_renderer),
            style_override: None,
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

    /// Adds a function that overrides the default styling coming from chart.
    ///
    /// If you just want to set a general style for the chart in general, see [`crate::Chart`] instead.
    pub fn style<F>(mut self, style_fn: F) -> Self
    where
        F: FnMut(&mut AxisStyle) + 'static,
    {
        self.style_override = Some(RefCell::new(Box::new(style_fn)));
        self
    }

    /// Creates the final style of this axis (overrides applied)
    pub(crate) fn create_style(&self, style: &Style) -> AxisStyle {
        let mut style = style.axis;
        if let Some(override_fn) = &self.style_override {
            (override_fn.borrow_mut())(&mut style)
        }
        style
    }

    /// Sets a custom renderer for ticks.
    ///
    /// This function gives you full control over which ticks render lines, grids, or labels.
    ///
    /// # Example
    /// ```rust
    /// # use iced_aksel::axis::{TickContext, TickResult};
    /// # let axis = iced_aksel::Axis::new(iced_aksel::scale::Linear::new(0.0, 10.0),
    /// iced_aksel::axis::Position::Bottom);
    /// axis.with_tick_renderer(|ctx: TickContext<f64>| {
    ///     if ctx.tick.level == 0 {
    ///         TickResult::with_label(ctx.label(format!("{:.1}", ctx.tick.value)))
    ///     } else {
    ///         TickResult::default() // Empty tick result
    ///     }
    /// });
    /// ```
    pub fn with_tick_renderer<F>(mut self, renderer: F) -> Self
    where
        F: FnMut(TickContext<D, Theme>) -> TickResult + 'static,
    {
        self.tick_renderer = Some(RefCell::new(Box::new(renderer)));
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

    /// Makes the axis invisible.
    ///
    /// It will still occupy layout space (defined by `thickness`) but will not render
    /// any ticks, lines, or labels. To remove it from layout entirely, set thickness to 0.
    pub const fn invisible(mut self) -> Self {
        self.invisible = true;
        self
    }

    /// Configures the axis to skip labels that would overlap.
    ///
    /// `min_gap_px` specifies the minimum distance in pixels required between labels.
    pub fn set_skip_overlapping_labels(&mut self, min_gap_px: f32) {
        self.label_policy = LabelPolicy::skip_overlapping(min_gap_px);
    }

    /// Updates the tick renderer in-place.
    pub fn set_tick_renderer<F>(&mut self, renderer: F)
    where
        F: Fn(TickContext<D, Theme>) -> TickResult + 'static,
    {
        self.tick_renderer = Some(RefCell::new(Box::new(renderer)));
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

    // TODO: Slight refactor to make it more readable
    // And to make it have less arguments
    /// Draws the axis, including ticks, grid lines, labels, and the interactive marker.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw<Renderer>(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        plot_bounds: &Rectangle,
        cache: &mut RenderCache<Renderer>,
        viewport: &Rectangle,
    ) where
        Renderer: crate::Renderer + iced_core::text::Renderer<Font = iced_core::Font>,
    {
        if self.invisible && !self.render_grid {
            return;
        }

        let style = self.create_style(style);
        let bounds = layout.bounds();
        let orientation = Orientation::from(self.position());
        let (&d_min, &d_max) = self.scale.domain();

        // --- Prioritize Ticks (Center-Out) ---
        let prioritized_ticks = self.collect_prioritized_ticks();

        let mut label_candidates = Vec::new();
        let mut candidate_max_bounds = Size::ZERO;

        // Iterate through the PRE-SORTED ticks
        for wrapper in prioritized_ticks {
            let tick = wrapper.tick;
            let pos_norm = self.normalize(&tick.value);

            let tick_result = self.tick_renderer.as_ref().map(|renderer| {
                renderer.borrow_mut()(TickContext {
                    tick,
                    normalized_position: pos_norm,
                    axis_bounds: &bounds,
                    scale_domain: (d_max, d_min),
                    orientation: &orientation,
                    style: &style,
                    theme,
                })
            });

            let Some(TickResult {
                tick_line,
                grid_line,
                label,
                label_badge: badge,
                label_priority,
            }) = tick_result
            else {
                continue;
            };

            // Draw Grid Lines (Global style + local config)
            if cache.needs_redraw()
                && self.render_grid
                && let Some(line) = grid_line
            {
                self.draw_grid_line(line, &bounds, plot_bounds, cache, pos_norm);
            }

            if self.invisible {
                continue;
            }

            // Collect labels for collision resolution
            if let Some(label) = label {
                label_candidates.push(LabelCandidate {
                    tick,
                    normalized_position: pos_norm,
                    label: label.clone(),
                    badge,
                    priority: label_priority.unwrap_or(tick.level),
                });

                // 3.1 Run all candidates, find the biggest label as source of truth for all label sizes
                // Make a paragraph for sizing
                let paragraph: Plain<Renderer::Paragraph> = Plain::new(Text {
                    content: label.content,
                    bounds: bounds.size(),
                    size: label.size,
                    line_height: label.line_height,
                    font: label.font.unwrap_or_else(|| renderer.default_font()),
                    align_x: Alignment::Left,
                    align_y: Vertical::Top,
                    shaping: iced_core::text::Shaping::Auto,
                    wrapping: Wrapping::None,
                });

                let width = paragraph.min_bounds().width + label.padding.left + label.padding.right;
                let height =
                    paragraph.min_bounds().height + label.padding.top + label.padding.bottom;

                if width > candidate_max_bounds.width {
                    candidate_max_bounds.width = width;
                }
                if height > candidate_max_bounds.height {
                    candidate_max_bounds.height = height;
                }
            }

            // Draw Tick Marks (Axis style + local config)
            if let Some(line) = tick_line {
                self.draw_tick_line(line, &bounds, renderer, pos_norm);
            }
        }

        self.draw_spine(renderer, &bounds, &style.spine, viewport);

        if self.invisible {
            return;
        }

        // 3. Resolve and Render Labels
        label_candidates.sort_by_key(|candidate| candidate.priority);

        self.layout_labels(
            renderer,
            &style,
            &bounds,
            orientation,
            label_candidates,
            candidate_max_bounds,
            viewport,
        );
    }

    /// Renders the axis spine (the continuous line along the axis) as a Quad in a separate layer.
    fn draw_spine<Renderer>(
        &self,
        renderer: &mut Renderer,
        bounds: &Rectangle,
        style: &SpineStyle,
        viewport: &Rectangle,
    ) where
        Renderer: crate::Renderer,
    {
        if style.width.0 <= 0.0 {
            return;
        }

        let width = style.width.0;
        let color = style.color;

        let spine_rect = match self.position {
            Position::Top => {
                // Spine at bottom edge of top axis
                Rectangle {
                    x: bounds.x,
                    y: bounds.y + bounds.height - width,
                    width: bounds.width,
                    height: width,
                }
            }
            Position::Bottom => {
                // Spine at top edge of bottom axis
                Rectangle {
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width,
                    height: width,
                }
            }
            Position::Left => {
                // Spine at right edge of left axis
                Rectangle {
                    x: bounds.x + bounds.width - width,
                    y: bounds.y,
                    width,
                    height: bounds.height,
                }
            }
            Position::Right => {
                // Spine at left edge of right axis
                Rectangle {
                    x: bounds.x,
                    y: bounds.y,
                    width,
                    height: bounds.height,
                }
            }
        };

        // Render spine in a separate layer to ensure it's always on top
        renderer.start_layer(*viewport);
        renderer.fill_quad(
            Quad {
                bounds: spine_rect,
                ..Default::default()
            },
            color,
        );
        renderer.end_layer();
    }

    /// Draws the interactive marker badge and line.
    ///
    /// This method ensures the badge stays within the viewport even if the mouse
    /// is at the extreme edges of the axis, preventing clipping.
    ///
    /// # Arguments
    /// * `normalized_position` - A value in the range 0.0..=1.0 that matches the axis orientation
    pub(super) fn draw_marker_overlay<Renderer>(
        &self,
        renderer: &mut Renderer,
        normalized_position: f32,
        marker: Marker,
        bounds: Rectangle,
        chart_bounds: &Rectangle,
        text_offset: Pixels,
    ) where
        Renderer: crate::Renderer + iced_core::text::Renderer<Font = iced_core::Font>,
    {
        let orientation = self.orientation();

        // Convert normalized position (0.0..=1.0) to screen coordinates
        let pos = match orientation {
            Orientation::Horizontal => bounds.width.mul_add(normalized_position, bounds.x),
            Orientation::Vertical => bounds.height.mul_add(1.0 - normalized_position, bounds.y),
        };
        let paragraph = Plain::<Renderer::Paragraph>::new(Text {
            content: marker.label.content,
            bounds: bounds.size(),
            size: marker.label.size,
            line_height: marker.label.line_height,
            font: marker.label.font.unwrap_or_else(|| renderer.default_font()),
            align_x: Alignment::Left,
            align_y: Vertical::Top,
            shaping: iced_core::text::Shaping::Auto,
            wrapping: Wrapping::None,
        });

        let rail_pos = self.calculate_rail_position(&bounds, orientation, text_offset.0);
        let min_bounds = paragraph.min_bounds();
        let padding = marker.label.padding;
        let badge_width = min_bounds.width + padding.left + padding.right;
        let badge_height = min_bounds.height + padding.top + padding.bottom;

        // Calculate initial badge position
        let mut badge_rect = match orientation {
            Orientation::Horizontal => {
                let x = pos - (badge_width / 2.0);
                let y = match self.position {
                    Position::Top => rail_pos - padding.bottom - min_bounds.height - padding.top,
                    _ => rail_pos,
                };
                Rectangle::new(Point::new(x, y), Size::new(badge_width, badge_height))
            }
            Orientation::Vertical => {
                let y = pos - (badge_height / 2.0);
                let x = match self.position {
                    Position::Right => rail_pos,
                    _ => rail_pos - badge_width, // badge_width already includes all padding
                };
                Rectangle::new(Point::new(x, y), Size::new(badge_width, badge_height))
            }
        };

        // Clamp badge position to viewport (the fix for extremes)
        match orientation {
            Orientation::Horizontal => {
                if badge_rect.x < chart_bounds.x {
                    badge_rect.x = chart_bounds.x;
                }
                if badge_rect.x + badge_rect.width > chart_bounds.x + chart_bounds.width {
                    badge_rect.x = chart_bounds.x + chart_bounds.width - badge_rect.width;
                }
            }
            Orientation::Vertical => {
                if badge_rect.y < chart_bounds.y {
                    badge_rect.y = chart_bounds.y;
                }
                if badge_rect.y + badge_rect.height > chart_bounds.y + chart_bounds.height {
                    badge_rect.y = chart_bounds.y + chart_bounds.height - badge_rect.height;
                }
            }
        }

        let gap = marker.line.gap.0;
        let line_width = marker.line.width.0;

        // Calculate marker line position (respecting the gap)
        let marker_line_rect = match orientation {
            Orientation::Horizontal => {
                let (y_start, y_end) = match self.position {
                    Position::Top => {
                        let line_start = bounds.y + bounds.height;
                        let line_end = (badge_rect.y + badge_rect.height + gap).min(line_start);
                        (line_end, line_start)
                    }
                    _ => {
                        let line_start = bounds.y;
                        let line_end = (badge_rect.y - gap).max(line_start);
                        (line_start, line_end)
                    }
                };

                Rectangle {
                    x: pos - (line_width / 2.0),
                    y: y_start.min(y_end),
                    width: line_width,
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
                    y: pos - (line_width / 2.0),
                    width: (x_end - x_start).abs(),
                    height: line_width,
                }
            }
        };

        // Render using the full chart_bounds clip to allow the badge to "bleed" out of the axis bounds
        renderer.start_layer(*chart_bounds);

        renderer.fill_quad(
            Quad {
                bounds: marker_line_rect,
                ..Default::default()
            },
            marker.line.color,
        );

        renderer.fill_quad(
            Quad {
                bounds: badge_rect,
                border: marker.badge.border,
                shadow: marker.badge.shadow,
                ..Default::default()
            },
            marker.badge.background,
        );

        let text_pos = Point::new(badge_rect.x + padding.left, badge_rect.y + padding.top);

        renderer.fill_text(
            paragraph
                .as_text()
                .with_content(paragraph.content().to_string()),
            text_pos,
            marker.label.color,
            *chart_bounds,
        );

        renderer.end_layer();
    }

    /// Calculates the base line position (the "rail") for text and decorations.
    fn calculate_rail_position(
        &self,
        bounds: &Rectangle,
        _orientation: Orientation,
        offset: f32,
    ) -> f32 {
        match self.position {
            Position::Bottom => bounds.y + offset,
            Position::Top => (bounds.y + bounds.height) - offset,
            Position::Left => (bounds.x + bounds.width) - offset,
            Position::Right => bounds.x + offset,
        }
    }

    /// Lays out and renders axis labels, resolving overlaps if the policy requires it.
    // TODO: refactor arguments
    #[allow(clippy::too_many_arguments)]
    fn layout_labels<Renderer>(
        &self,
        renderer: &mut Renderer,
        style: &AxisStyle,
        bounds: &Rectangle,
        orientation: Orientation,
        label_candidates: Vec<LabelCandidate<D>>,
        candidate_max_size: Size,
        viewport: &Rectangle,
    ) where
        Renderer: crate::Renderer + iced_core::text::Renderer<Font = iced_core::Font>,
    {
        let mut accepted: Vec<PlacedLabelInfo<D>> = Vec::new();

        for candidate in label_candidates {
            let Some(resolved) = self.resolve_label_candidate(
                candidate,
                &candidate_max_size,
                renderer,
                bounds,
                orientation,
                style.text_offset.0,
            ) else {
                continue;
            };

            let ResolvedLabelCandidate {
                tick,
                normalized_position,
                bounds: label_bounds,
                paragraph,
                position,
                badge,
                badge_bounds,
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
                // 1. Render Badge (Background)
                if let Some(badge) = badge {
                    renderer.fill_quad(
                        Quad {
                            bounds: badge_bounds,
                            border: badge.border,
                            shadow: badge.shadow,
                            ..Default::default()
                        },
                        badge.background,
                    );
                }

                // 2. Render Text
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

    /// Measures a label candidate and calculates its screen bounds.
    fn resolve_label_candidate<Renderer>(
        &self,
        candidate: LabelCandidate<D>,
        candidate_max_size: &Size,
        renderer: &Renderer,
        bounds: &Rectangle,
        orientation: Orientation,
        offset: f32,
    ) -> Option<ResolvedLabelCandidate<Renderer, D>>
    where
        Renderer: iced_core::text::Renderer<Font = iced_core::Font>,
    {
        let label = candidate.label;
        if label.content.is_empty() {
            return None;
        }

        if candidate.normalized_position.is_sign_negative() {
            return None;
        }

        let paragraph = Plain::new(Text {
            content: label.content,
            bounds: bounds.size(),
            size: label.size,
            line_height: label.line_height,
            font: label.font.unwrap_or_else(|| renderer.default_font()),
            align_x: Alignment::Left,
            align_y: Vertical::Top,
            shaping: iced_core::text::Shaping::Auto,
            wrapping: Wrapping::None,
        });

        let rail_pos = self.calculate_rail_position(bounds, orientation, offset);

        let text_width = paragraph.min_bounds().width;
        let text_height = paragraph.min_bounds().height;

        let padding = label.padding;

        let position = match self.position {
            Position::Top => {
                let center_x = bounds
                    .width
                    .mul_add(candidate.normalized_position, bounds.x);
                let total_width = text_width + padding.left + padding.right;
                let x = center_x - (total_width / 2.0) + padding.left;

                let y = rail_pos - text_height - padding.bottom;
                Point::new(x, y)
            }
            Position::Bottom => {
                let center_x = bounds
                    .width
                    .mul_add(candidate.normalized_position, bounds.x);

                let total_width = text_width + padding.left + padding.right;
                let x = center_x - (total_width / 2.0) + padding.left;

                let y = rail_pos + padding.top;
                Point::new(x, y)
            }
            Position::Left => {
                let center_y = bounds
                    .height
                    .mul_add(1.0 - candidate.normalized_position, bounds.y);

                let total_height = text_height + padding.top + padding.bottom;
                let y = center_y - (total_height / 2.0) + padding.top;

                let x = rail_pos - text_width - padding.right;
                Point::new(x, y)
            }
            Position::Right => {
                let center_y = bounds
                    .height
                    .mul_add(1.0 - candidate.normalized_position, bounds.y);

                let total_height = text_height + padding.top + padding.bottom;
                let y = center_y - (total_height / 2.0) + padding.top;

                let x = rail_pos + padding.left;
                Point::new(x, y)
            }
        };

        let (start, end) = match orientation {
            Orientation::Horizontal => {
                let center = bounds
                    .width
                    .mul_add(candidate.normalized_position, bounds.x);
                let half = candidate_max_size.width / 2.0;
                (center - half, center + half)
            }
            Orientation::Vertical => {
                let center = bounds
                    .height
                    .mul_add(1.0 - candidate.normalized_position, bounds.y);
                let half = candidate_max_size.height / 2.0;
                (center - half, center + half)
            }
        };

        // NEW: Calculate the full 2D badge rectangle
        let badge_bounds = Rectangle {
            x: position.x - padding.left,
            y: position.y - padding.top,
            width: text_width + padding.left + padding.right,
            height: text_height + padding.top + padding.bottom,
        };

        Some(ResolvedLabelCandidate {
            tick: candidate.tick,
            normalized_position: candidate.normalized_position,
            bounds: LabelBounds::new(start, end),
            paragraph,
            position,
            badge_bounds,
            badge: candidate.badge,
            color: label.color,
        })
    }

    /// Renders a single tick mark
    fn draw_tick_line<Renderer>(
        &self,
        line: TickLine,
        bounds: &Rectangle,
        renderer: &mut Renderer,
        pos_norm: f32,
    ) where
        Renderer: crate::Renderer,
    {
        let thickness = line.width.0;
        let half_thickness = thickness / 2.0;
        let length = line.length.0;
        let fill = line.color;

        match self.position {
            Position::Bottom => {
                let x = bounds.width.mul_add(pos_norm, bounds.x) - half_thickness;
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(
                            Point { x, y: bounds.y },
                            Size {
                                width: thickness,
                                height: length,
                            },
                        ),
                        snap: true,
                        ..Default::default()
                    },
                    fill,
                );
            }
            Position::Top => {
                let x = bounds.width.mul_add(pos_norm, bounds.x) - half_thickness;
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(
                            Point { x, y: bounds.y },
                            Size {
                                width: thickness,
                                height: length,
                            },
                        ),
                        snap: true,
                        ..Default::default()
                    },
                    fill,
                );
            }
            Position::Right => {
                let y = bounds.height.mul_add(1.0 - pos_norm, bounds.y) - half_thickness;
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(
                            Point { x: bounds.x, y },
                            Size {
                                width: length,
                                height: thickness,
                            },
                        ),
                        snap: true,
                        ..Default::default()
                    },
                    fill,
                );
            }
            Position::Left => {
                let y = bounds.height.mul_add(1.0 - pos_norm, bounds.y) - half_thickness;
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(
                            Point {
                                x: bounds.x + bounds.width - length,
                                y,
                            },
                            Size {
                                width: length,
                                height: thickness,
                            },
                        ),
                        snap: true,
                        ..Default::default()
                    },
                    fill,
                );
            }
        }
    }

    /// Renders a single grid line into the cache.
    fn draw_grid_line<Renderer>(
        &self,
        line: GridLine,
        axis_bounds: &Rectangle,
        plot_bounds: &Rectangle,
        cache: &mut RenderCache<Renderer>,
        pos_norm: f32,
    ) where
        Renderer: crate::Renderer,
    {
        let orientation = self.orientation();
        let thickness = line.width.0;
        let fill = line.color;

        match orientation {
            Orientation::Horizontal => {
                let x = axis_bounds.width.mul_add(pos_norm, axis_bounds.x);
                if let Some(DashStyle {
                    line_length,
                    gap_length,
                }) = line.dashed
                {
                    cache.add_primitive(Primitive::VerticalLine {
                        x,
                        y_start: plot_bounds.y,
                        y_end: plot_bounds.y + plot_bounds.height,
                        stroke: ResolvedStroke {
                            fill,
                            thickness,
                            style: StrokeStyle::Dashed {
                                gap: gap_length,
                                dash: line_length,
                            },
                        },
                        snap: true,
                    });
                } else {
                    cache.add_primitive(Primitive::VerticalLine {
                        x,
                        y_start: plot_bounds.y,
                        y_end: plot_bounds.y + plot_bounds.height,
                        stroke: ResolvedStroke {
                            fill,
                            thickness,
                            style: StrokeStyle::Solid,
                        },
                        snap: true,
                    });
                }
            }
            Orientation::Vertical => {
                let y = axis_bounds.height.mul_add(1.0 - pos_norm, axis_bounds.y);
                if let Some(DashStyle {
                    line_length,
                    gap_length,
                }) = line.dashed
                {
                    cache.add_primitive(Primitive::HorizontalLine {
                        y,
                        x_start: plot_bounds.x,
                        x_end: plot_bounds.x + plot_bounds.width,
                        stroke: ResolvedStroke {
                            fill,
                            thickness,
                            style: StrokeStyle::Dashed {
                                dash: line_length,
                                gap: gap_length,
                            },
                        },
                        snap: true,
                    });
                } else {
                    cache.add_primitive(Primitive::HorizontalLine {
                        y,
                        x_start: plot_bounds.x,
                        x_end: plot_bounds.x + plot_bounds.width,
                        stroke: ResolvedStroke {
                            fill,
                            thickness,
                            style: StrokeStyle::Solid,
                        },
                        snap: true,
                    });
                }
            }
        }
    }

    /// Collects ticks and sorts them so that "Center" ticks in minor intervals come before "Edge" ticks.
    fn collect_prioritized_ticks(&self) -> Vec<PrioritizedTick<D>> {
        let all_ticks = self.ticks();
        let mut prioritized = Vec::with_capacity(all_ticks.len());

        // 1. Identify Major Intervals
        // (We only need the values for this, so we map to f32 immediately)
        let mut major_tick_values: Vec<f32> = all_ticks
            .iter()
            .filter(|t| t.level == 0)
            .filter_map(|t| t.value.to_f32())
            .collect();

        // Sort to ensure valid intervals
        major_tick_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // 2. Score every tick
        for tick in all_ticks {
            let val = tick.value.to_f32().unwrap_or(0.0);

            let score = if tick.level == 0 {
                0.0 // Priority 1: Major Ticks
            } else {
                // Find which interval this tick belongs to
                // standard binary search or simple iteration is fine for <1000 ticks
                let upper_idx = major_tick_values.partition_point(|&x| x <= val);

                if upper_idx > 0 && upper_idx < major_tick_values.len() {
                    let lower_val = major_tick_values[upper_idx - 1];
                    let upper_val = major_tick_values[upper_idx];
                    let interval = upper_val - lower_val;

                    if interval.abs() < f32::EPSILON {
                        0.0
                    } else {
                        // Distance from center of interval (0.0 is perfect center)
                        let center = (lower_val + upper_val) / 2.0;
                        let dist = (val - center).abs();

                        // Priority 2: Middle Ticks (Score ~1.0)
                        // Priority 3: Edge Ticks (Score ~1.5)
                        1.0 + (dist / interval)
                    }
                } else {
                    // Ticks outside valid major intervals (e.g. at the very edge of the domain)
                    2.0
                }
            };

            prioritized.push(PrioritizedTick { tick, score });
        }

        // 3. Sort (Stable sort to preserve any internal logic from the scale)
        prioritized.sort_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        prioritized
    }
}

impl<D: Float, Theme> Deref for Axis<D, Theme> {
    type Target = dyn Scale<Domain = D, Normalized = f32>;

    fn deref(&self) -> &Self::Target {
        &*self.scale
    }
}

impl<D: Float, Theme> DerefMut for Axis<D, Theme> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.scale
    }
}
