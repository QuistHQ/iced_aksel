//! Axis configuration and rendering.

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
    text::{Wrapping, paragraph::Plain},
    widget::text::Alignment,
};
use iced_graphics::{color, mesh::SolidVertex2D};

use crate::{
    plot,
    render::MeshBuffer,
    style::{AxisStyle, GridStyle, Style, TextStyle, TickStyle},
};

mod grid;
mod label;
mod position;
mod tick;

pub use grid::*;
pub use label::*;
pub use position::*;
pub use tick::*;

type TickRendererFn<D> = Rc<RefCell<dyn FnMut(TickContext<D>) -> TickResult>>;
type CursorRendererFn<D> = Rc<RefCell<dyn FnMut(D) -> Option<String>>>;

/// An axis that maps data values to screen coordinates.
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
    pub(crate) cursor_formatter: Option<CursorRendererFn<D>>,
    #[derivative(Debug = "ignore")]
    label_policy: LabelPolicy<D>,
}

impl<D: Float> Axis<D> {
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

            scale: Box::new(scale),
            tick_renderer: Some(tick_renderer),
            cursor_formatter: None,
            label_policy: LabelPolicy::default(),
        }
    }

    pub fn with_thickness<P: Into<Pixels>>(mut self, thickness: P) -> Self {
        self.thickness = thickness.into();
        self
    }

    pub fn with_tick_renderer<F>(mut self, renderer: F) -> Self
    where
        F: FnMut(TickContext<D>) -> TickResult + 'static,
    {
        self.tick_renderer = Some(Rc::new(RefCell::new(renderer)));
        self
    }

    pub const fn without_grid(mut self) -> Self {
        self.render_grid = false;
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
        F: FnMut(D) -> Option<String> + 'static,
    {
        self.cursor_formatter = Some(Rc::new(RefCell::new(renderer)));
        self
    }

    pub const fn invisible(mut self) -> Self {
        self.invisible = true;
        self
    }

    pub fn set_tick_renderer<F>(&mut self, renderer: F)
    where
        F: Fn(TickContext<D>) -> TickResult + 'static,
    {
        self.tick_renderer = Some(Rc::new(RefCell::new(renderer)));
    }

    pub const fn set_visibility(&mut self, visible: bool) {
        self.invisible = !visible;
    }

    pub fn set_thickness<P: Into<Pixels>>(&mut self, thickness: P) {
        self.thickness = thickness.into();
    }

    pub const fn is_visible(&self) -> bool {
        !self.invisible
    }

    pub fn domain(&self) -> (&D, &D) {
        self.scale.domain()
    }

    pub const fn position(&self) -> &Position {
        &self.position
    }

    pub fn orientation(&self) -> Orientation {
        Orientation::from(&self.position)
    }

    pub const fn thickness(&self) -> Pixels {
        if self.invisible {
            return Pixels(0.0);
        }
        self.thickness
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
        Renderer: plot::Renderer + iced_core::text::Renderer<Font = iced_core::Font>,
    {
        if self.invisible && !self.render_grid {
            return;
        }

        let theme = style.axis;
        let bounds = layout.bounds();
        let full_bounds = plot_bounds.union(&bounds);
        let orientation = Orientation::from(self.position());
        let (&d_min, &d_max) = self.scale.domain();

        // --- Step 1: Measure Cursor (Dry Run) ---
        let cursor_state = if self.render_cursor
            && let Some(cursor_pos) = cursor.position_over(full_bounds)
            && let Some(cursor_renderer) = &self.cursor_formatter
        {
            let value_to_render = match orientation {
                Orientation::Horizontal => (cursor_pos.x - plot_bounds.x) / plot_bounds.width,
                Orientation::Vertical => {
                    1.0 - ((cursor_pos.y - plot_bounds.y) / plot_bounds.height)
                }
            };

            self.denormalize_opt(value_to_render)
                .and_then(|val| cursor_renderer.borrow_mut()(val))
                .map(|content| {
                    let paragraph = Plain::<Renderer::Paragraph>::new(Text {
                        content,
                        bounds: bounds.size(),
                        size: theme.cursor.text.size,
                        line_height: theme.cursor.text.line_height,
                        font: theme.cursor.text.font,
                        align_x: Alignment::Left,
                        align_y: Vertical::Top,
                        shaping: theme.cursor.text.shaping,
                        wrapping: Wrapping::None,
                    });

                    (cursor_pos, paragraph)
                })
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
                grid_line,
                label,
                label_priority,
            }) = tick_result
            else {
                continue;
            };

            // Draw Grid using Global Grid Style + local line info
            // FIX: Use plot_bounds here, NOT axis bounds!
            if self.render_grid
                && let Some(line) = grid_line
            {
                self.draw_grid_line(&style.grid, plot_bounds, line, mesh_buffer, pos_norm);
            }

            if self.invisible {
                continue;
            }

            if let Some(label) = label {
                label_candidates.push(LabelCandidate {
                    tick,
                    normalized_position: pos_norm,
                    label,
                    priority: label_priority.unwrap_or(tick.level),
                });
            }

            // Draw Tick using Axis Tick Style + local line info
            if let Some(line) = tick_line {
                self.draw_tick_line(&theme.ticks, line, &bounds, mesh_buffer, pos_norm);
            }
        }

        if self.invisible {
            return;
        }

        label_candidates.sort_by_key(|candidate| candidate.priority);

        self.layout_labels(
            renderer,
            &theme,
            &bounds,
            orientation,
            label_candidates,
            viewport,
        );

        if let Some((cursor_pos, paragraph)) = cursor_state {
            self.draw_cursor_overlay(
                renderer,
                cursor_pos,
                paragraph,
                bounds,
                viewport,
                orientation,
                theme,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_cursor_overlay<Renderer>(
        &self,
        renderer: &mut Renderer,
        cursor_pos: Point,
        paragraph: Plain<Renderer::Paragraph>,
        bounds: Rectangle,
        viewport: &Rectangle,
        orientation: Orientation,
        theme: AxisStyle,
    ) where
        Renderer: plot::Renderer + iced_core::text::Renderer<Font = iced_core::Font>,
    {
        let rail_pos = self.calculate_rail_position(&bounds, orientation, theme.text_offset);
        let min_bounds = paragraph.min_bounds();
        let padding = theme.cursor.badge.padding;
        let badge_width = min_bounds.width + padding.left + padding.right;
        let badge_height = min_bounds.height + padding.top + padding.bottom;

        let badge_rect = match orientation {
            Orientation::Horizontal => {
                let x = cursor_pos.x - (badge_width / 2.0);
                let y = match self.position {
                    Position::Top => rail_pos - padding.bottom - min_bounds.height - padding.top,
                    _ => rail_pos,
                };
                Rectangle::new(Point::new(x, y), Size::new(badge_width, badge_height))
            }
            Orientation::Vertical => {
                let y = cursor_pos.y - (badge_height / 2.0);
                let x = match self.position {
                    Position::Right => rail_pos,
                    _ => rail_pos - padding.right - min_bounds.width - padding.left,
                };
                Rectangle::new(Point::new(x, y), Size::new(badge_width, badge_height))
            }
        };

        let gap = theme.cursor.line_gap.0;
        let line_width = theme.cursor.width.0;

        let (cursor_line_rect, clip_bounds) = match orientation {
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

                let rect = Rectangle {
                    x: cursor_pos.x - (line_width / 2.0),
                    y: y_start.min(y_end),
                    width: line_width,
                    height: (y_end - y_start).abs(),
                };
                let clip = Rectangle {
                    width: bounds.width.max(viewport.width),
                    x: bounds.x.min(viewport.x),
                    ..bounds
                };
                (rect, clip)
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

                let rect = Rectangle {
                    x: x_start.min(x_end),
                    y: cursor_pos.y - (line_width / 2.0),
                    width: (x_end - x_start).abs(),
                    height: line_width,
                };
                let clip = Rectangle {
                    height: bounds.height.max(viewport.height),
                    y: bounds.y.min(viewport.y),
                    ..bounds
                };
                (rect, clip)
            }
        };

        renderer.start_layer(clip_bounds);
        renderer.fill_quad(
            Quad {
                bounds: cursor_line_rect,
                ..Default::default()
            },
            theme.cursor.color,
        );

        renderer.fill_quad(
            Quad {
                bounds: badge_rect,
                border: theme.cursor.badge.border,
                shadow: theme.cursor.badge.shadow,
                ..Default::default()
            },
            theme.cursor.badge.background,
        );

        let text_pos = Point::new(badge_rect.x + padding.left, badge_rect.y + padding.top);

        renderer.fill_text(
            paragraph
                .as_text()
                .with_content(paragraph.content().to_string()),
            text_pos,
            theme.cursor.text.color,
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
        theme: &AxisStyle,
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
                renderer,
                candidate,
                bounds,
                orientation,
                &theme.label,
                theme.text_offset,
            ) else {
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
                    theme.label.color,
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
        text_style: &TextStyle,
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
        })
    }

    fn draw_tick_line(
        &self,
        style: &TickStyle,
        line: TickLine,
        bounds: &Rectangle,
        mesh_buffer: &mut MeshBuffer,
        pos_norm: f32,
    ) {
        let (x0, y0, x1, y1) = match self.position {
            Position::Bottom => {
                let x = bounds.width.mul_add(pos_norm, bounds.x).round();
                (x, bounds.y, x + line.thickness.0, bounds.y + line.length.0)
            }
            Position::Top => {
                let x = bounds.width.mul_add(pos_norm, bounds.x).round();
                (
                    x,
                    bounds.y + bounds.height - line.length.0,
                    x + line.thickness.0,
                    bounds.y + bounds.height,
                )
            }
            Position::Right => {
                let y = bounds.height.mul_add(1.0 - pos_norm, bounds.y).round();
                (bounds.x, y, bounds.x + line.length.0, y + line.thickness.0)
            }
            Position::Left => {
                let y = bounds.height.mul_add(1.0 - pos_norm, bounds.y).round();
                (
                    bounds.x + bounds.width - line.length.0,
                    y,
                    bounds.x + bounds.width,
                    y + line.thickness.0,
                )
            }
        };

        let color = color::pack(style.color);
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
        style: &GridStyle,
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

        let packed = color::pack(style.color);
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
