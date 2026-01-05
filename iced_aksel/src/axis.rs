use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use aksel::{Float, Scale};
use derivative::Derivative;
use iced_core::{
    Color, Layout, Pixels, Point, Rectangle, Size, Text,
    alignment::Vertical,
    layout::{Limits, Node},
    mouse::Cursor,
    renderer::Quad,
    text::{Wrapping, paragraph::Plain},
    widget::text::Alignment,
};

use crate::{
    plot,
    render::MeshBuffer,
    style::{CursorStyle, GridStyle, LineStyle, Style, TickStyle},
};

mod cursor;
mod label;
mod position;
mod tick;

use crate::render::manual::linear::{
    draw_dashed_line, draw_horizontal_dashed_line, draw_horizontal_line, draw_vertical_dashed_line,
    draw_vertical_line,
};

pub use cursor::*;
pub use label::*;
pub use position::*;
pub use tick::*;

type TickRendererFn<D> = Rc<RefCell<dyn FnMut(TickContext<D>) -> TickResult>>;
type CursorRendererFn<D> = Rc<RefCell<dyn FnMut(D, &CursorStyle) -> Option<CursorResult>>>;

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

    pub(crate) text_offset: Pixels,
}

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
        _style: &Style,
        viewport: &Rectangle,
    ) where
        Renderer: plot::Renderer + iced_core::text::Renderer<Font = iced_core::Font>,
    {
        let mut label_candidates = self.label_candidates;
        label_candidates.sort_by_key(|candidate| candidate.priority);

        axis.layout_labels(
            renderer,
            &self.bounds,
            self.orientation,
            label_candidates,
            viewport,
        );

        if let Some((cursor_pos, result)) = self.cursor_state {
            let paragraph = if let Some(label) = &result.label {
                let text_style = &result.style.text;
                Some(Plain::<Renderer::Paragraph>::new(Text {
                    content: label.clone(),
                    bounds: self.bounds.size(),
                    size: text_style.size,
                    line_height: text_style.line_height,
                    font: text_style.font,
                    align_x: Alignment::Left,
                    align_y: Vertical::Top,
                    shaping: text_style.shaping,
                    wrapping: Wrapping::None,
                }))
            } else {
                None
            };

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

            text_offset: Pixels(6.0),
        }
    }

    // --- Configuration ---

    pub fn with_thickness<P: Into<Pixels>>(mut self, thickness: P) -> Self {
        self.thickness = thickness.into();
        self
    }

    pub fn with_text_offset<P: Into<Pixels>>(mut self, offset: P) -> Self {
        self.text_offset = offset.into();
        self
    }

    pub fn with_tick_renderer<F>(mut self, renderer: F) -> Self
    where
        F: FnMut(TickContext<D>) -> TickResult + 'static,
    {
        self.tick_renderer = Some(Rc::new(RefCell::new(renderer)));
        self
    }

    pub fn with_cursor_renderer<F>(mut self, renderer: F) -> Self
    where
        F: FnMut(D, &CursorStyle) -> Option<CursorResult> + 'static,
    {
        self.cursor_renderer = Some(Rc::new(RefCell::new(renderer)));
        self
    }

    pub const fn without_grid(mut self) -> Self {
        self.render_grid = false;
        self
    }

    pub const fn invisible(mut self) -> Self {
        self.invisible = true;
        self
    }

    // --- Getters & Layout ---

    pub const fn position(&self) -> &Position {
        &self.position
    }
    pub fn orientation(&self) -> Orientation {
        Orientation::from(&self.position)
    }

    pub const fn thickness(&self) -> Pixels {
        if self.invisible {
            Pixels(0.0)
        } else {
            self.thickness
        }
    }

    pub fn domain(&self) -> (&D, &D) {
        self.scale.domain()
    }

    pub(crate) fn screen_to_normalized(&self, screen_pos: f32, bounds: &Rectangle) -> f32 {
        match self.orientation() {
            Orientation::Horizontal => (screen_pos - bounds.x) / bounds.width,
            Orientation::Vertical => 1.0 - ((screen_pos - bounds.y) / bounds.height),
        }
    }

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

    pub fn is_visible(&self) -> bool {
        !self.invisible
    }

    // --- Drawing Logic ---

    pub(crate) fn draw_grid(
        &self,
        global_style: &Style,
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

        // Pass 1: Extract Theme Defaults
        let theme_axis = &global_style.axis;

        for tick in self.ticks().into_iter() {
            let pos_norm = self.normalize(&tick.value);

            let tick_result = if let Some(renderer) = &self.tick_renderer {
                // Pass Theme Styles to Closure
                renderer.borrow_mut()(TickContext {
                    tick,
                    tick_style: &theme_axis.ticks,
                    grid_style: &theme_axis.grid,
                    normalized_position: pos_norm,
                    axis_bounds: bounds,
                    scale_domain: (d_max, d_min),
                    orientation,
                })
            } else {
                // Use Theme Styles directly
                TickResult::from_tick_style(theme_axis.ticks).grid_style(theme_axis.grid)
            };

            if let Some(final_grid_style) = tick_result.grid_style {
                self.draw_grid_line(
                    &layout.bounds(),
                    plot_bounds,
                    &final_grid_style,
                    mesh_buffer,
                    pos_norm,
                );
            }
        }
    }

    pub(crate) fn draw_ticks(
        &self,
        global_style: &Style,
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

        // Pass 1: Extract Theme Defaults
        let theme_axis = &global_style.axis;
        let theme_cursor = &global_style.cursor;

        let (&d_min, &d_max) = self.scale.domain();
        let full_bounds = plot_bounds.union(&bounds);

        // 1. Cursor Logic
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
                .and_then(|val| cursor_renderer.borrow_mut()(val, theme_cursor))
                .map(|result| (cursor_pos, result))
        } else {
            None
        };

        // 2. Tick Logic
        let mut label_candidates = Vec::new();

        for tick in self.ticks().into_iter() {
            let pos_norm = self.normalize(&tick.value);

            let context = TickContext {
                tick,
                tick_style: &theme_axis.ticks,
                grid_style: &theme_axis.grid,
                normalized_position: pos_norm,
                axis_bounds: bounds,
                scale_domain: (d_max, d_min),
                orientation,
            };

            let tick_result = if let Some(renderer) = &self.tick_renderer {
                renderer.borrow_mut()(context)
            } else {
                let mut result = TickResult::from_tick_style(theme_axis.ticks);
                if tick.level == 0 {
                    result = result.label(format!("{:.0}", tick.value.to_f32().unwrap_or(0.0)));
                }
                result
            };

            // Draw Tick Line
            self.draw_tick_line(&tick_result.tick_style, &bounds, mesh_buffer, pos_norm);

            // Collect Labels
            if let Some(label_text) = tick_result.label {
                label_candidates.push(LabelCandidate {
                    tick,
                    normalized_position: pos_norm,
                    text: label_text,
                    style: tick_result.tick_style.text,
                    priority: tick_result.label_priority.unwrap_or(tick.level),
                });
            }
        }

        // 3. Spine Logic
        self.draw_axis_spine(&theme_axis.spine, &bounds, mesh_buffer);

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
        let style = cursor_result.style;
        let padding = style.badge.padding;

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

        let rail_pos = self.calculate_rail_position(&bounds, orientation, self.text_offset);

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

        let gap = style.line_gap.0;
        let width = style.line.width.0;
        let color = style.line.color;
        let half_width = width / 2.0;

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
                    Size::new(width, y_max - y_min),
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
                    Size::new(x_max - x_min, width),
                )
            }
        };

        renderer.fill_quad(
            Quad {
                bounds: line_rect,
                ..Default::default()
            },
            color,
        );

        if let Some(p) = paragraph {
            renderer.fill_quad(
                Quad {
                    bounds: badge_rect,
                    border: style.badge.border,
                    shadow: style.badge.shadow,
                    ..Default::default()
                },
                style.badge.background.unwrap_or(Color::TRANSPARENT.into()),
            );

            let text_pos = Point::new(badge_rect.x + padding.left, badge_rect.y + padding.top);
            renderer.fill_text(
                p.as_text().with_content(p.content().to_string()),
                text_pos,
                style.text.color,
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
        bounds: &Rectangle,
        orientation: Orientation,
        label_candidates: Vec<LabelCandidate<D>>,
        viewport: &Rectangle,
    ) where
        Renderer: plot::Renderer + iced_core::text::Renderer<Font = iced_core::Font>,
    {
        let mut accepted: Vec<PlacedLabelInfo<D>> = Vec::new();
        for candidate in label_candidates {
            if let Some(resolved) = self.resolve_label_candidate::<Renderer>(
                candidate,
                bounds,
                orientation,
                self.text_offset,
            ) {
                let context = LabelDecisionContext {
                    tick: resolved.tick,
                    normalized_position: resolved.normalized_position,
                    bounds: resolved.bounds,
                    orientation,
                    accepted: &accepted,
                };

                if self.label_policy.should_render(context) {
                    renderer.fill_text(
                        resolved
                            .paragraph
                            .as_text()
                            .with_content(resolved.paragraph.content().to_string()),
                        resolved.position,
                        resolved.color,
                        *viewport,
                    );
                    accepted.push(PlacedLabelInfo {
                        tick: resolved.tick,
                        normalized_position: resolved.normalized_position,
                        bounds: resolved.bounds,
                    });
                }
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
        if candidate.text.is_empty() || candidate.normalized_position.is_sign_negative() {
            return None;
        }

        let text_style = candidate.style;
        let paragraph = Plain::new(Text {
            content: candidate.text,
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
                (
                    center - text_bounds.width / 2.0,
                    center + text_bounds.width / 2.0,
                )
            }
            Orientation::Vertical => {
                let center = bounds
                    .height
                    .mul_add(1.0 - candidate.normalized_position, bounds.y);
                (
                    center - text_bounds.height / 2.0,
                    center + text_bounds.height / 2.0,
                )
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
        style: &TickStyle,
        bounds: &Rectangle,
        mesh_buffer: &mut MeshBuffer,
        pos_norm: f32,
    ) {
        let length = style.length.0;
        let width = style.line.width.0;
        let color = style.line.color;

        match self.position {
            Position::Bottom => {
                let x = bounds.width.mul_add(pos_norm, bounds.x);
                draw_vertical_line(
                    mesh_buffer,
                    x,
                    bounds.y,
                    bounds.y + length,
                    width,
                    color,
                    true,
                );
            }
            Position::Top => {
                let x = bounds.width.mul_add(pos_norm, bounds.x);
                draw_vertical_line(
                    mesh_buffer,
                    x,
                    bounds.y + bounds.height - length,
                    bounds.y + bounds.height,
                    width,
                    color,
                    true,
                );
            }
            Position::Right => {
                let y = bounds.height.mul_add(1.0 - pos_norm, bounds.y);
                draw_horizontal_line(
                    mesh_buffer,
                    bounds.x,
                    bounds.x + length,
                    y,
                    width,
                    color,
                    true,
                );
            }
            Position::Left => {
                let y = bounds.height.mul_add(1.0 - pos_norm, bounds.y);
                draw_horizontal_line(
                    mesh_buffer,
                    bounds.x + bounds.width - length,
                    bounds.x + bounds.width,
                    y,
                    width,
                    color,
                    true,
                );
            }
        };
    }

    fn draw_grid_line(
        &self,
        axis_bounds: &Rectangle,
        plot_bounds: &Rectangle,
        style: &GridStyle,
        mesh_buffer: &mut MeshBuffer,
        pos_norm: f32,
    ) {
        let width = style.line.width.0;
        let color = style.line.color;
        let dashed = style.dashed;
        let dash_len = 5.0;
        let gap_len = 5.0;

        match self.orientation() {
            Orientation::Horizontal => {
                let x = axis_bounds.width.mul_add(pos_norm, axis_bounds.x);
                if dashed {
                    draw_vertical_dashed_line(
                        mesh_buffer,
                        x,
                        plot_bounds.y,
                        plot_bounds.y + plot_bounds.height,
                        width,
                        color,
                        dash_len,
                        gap_len,
                        true,
                    );
                } else {
                    draw_vertical_line(
                        mesh_buffer,
                        x,
                        plot_bounds.y,
                        plot_bounds.y + plot_bounds.height,
                        width,
                        color,
                        true,
                    );
                }
            }
            Orientation::Vertical => {
                let y = axis_bounds.height.mul_add(1.0 - pos_norm, axis_bounds.y);
                if dashed {
                    draw_horizontal_dashed_line(
                        mesh_buffer,
                        plot_bounds.x,
                        plot_bounds.x + plot_bounds.width,
                        y,
                        width,
                        color,
                        dash_len,
                        gap_len,
                        true,
                    );
                } else {
                    draw_horizontal_line(
                        mesh_buffer,
                        plot_bounds.x,
                        plot_bounds.x + plot_bounds.width,
                        y,
                        width,
                        color,
                        true,
                    );
                }
            }
        }
    }

    fn draw_axis_spine(&self, style: &LineStyle, bounds: &Rectangle, mesh_buffer: &mut MeshBuffer) {
        let thickness = style.width.0;
        let color = style.color;
        let offset = thickness / 2.0;

        match self.position {
            Position::Bottom => draw_horizontal_line(
                mesh_buffer,
                bounds.x,
                bounds.x + bounds.width,
                bounds.y + offset,
                thickness,
                color,
                true,
            ),
            Position::Top => draw_horizontal_line(
                mesh_buffer,
                bounds.x,
                bounds.x + bounds.width,
                bounds.y + bounds.height - offset,
                thickness,
                color,
                true,
            ),
            Position::Left => draw_vertical_line(
                mesh_buffer,
                bounds.x + bounds.width - offset,
                bounds.y,
                bounds.y + bounds.height,
                thickness,
                color,
                true,
            ),
            Position::Right => draw_vertical_line(
                mesh_buffer,
                bounds.x + offset,
                bounds.y,
                bounds.y + bounds.height,
                thickness,
                color,
                true,
            ),
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
