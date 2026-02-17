use super::Primitive;

use crate::stroke::{ResolvedStroke, StrokeStyle};
use iced_core::alignment::{Horizontal, Vertical};
use iced_core::text::Shaping;
use iced_core::{Point, Rectangle, Vector};
use iced_graphics::geometry::{
    Cache, Frame, LineCap, LineDash, LineJoin, Path, Stroke, Style, Text,
};

const PRE_ALLOC_PATHS: usize = 5000;

pub struct PathCache<Renderer: crate::Renderer> {
    buffer: Vec<Primitive>,
    cache: Cache<Renderer>,
    needs_redraw: bool,
}

impl<Renderer: crate::render::Renderer> PathCache<Renderer> {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(PRE_ALLOC_PATHS),
            cache: Cache::new(),
            needs_redraw: true,
        }
    }

    pub const fn paths_count(&self) -> usize {
        self.buffer.len()
    }

    /// Renders a primitive into this path buffer.
    ///
    /// This converts the primitive into tiny-skia compatible paths.
    pub fn add_primitive(&mut self, primitive: Primitive) {
        self.buffer.push(primitive)
    }

    /// Clear the buffer, triggering a redraw
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cache.clear();
        self.needs_redraw = true;
    }

    pub const fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    pub(crate) fn draw(&mut self, renderer: &mut Renderer, clip_bounds: &Rectangle) {
        let geometry = self
            .cache
            .draw_with_bounds(renderer, *clip_bounds, |frame| {
                if self.needs_redraw {
                    self.needs_redraw = false;
                    self.buffer
                        .iter()
                        .for_each(|primitive| Self::draw_primitive(primitive, frame))
                }
            });

        renderer.draw_geometry(geometry);
    }

    fn draw_primitive(primitive: &Primitive, frame: &mut Frame<Renderer>) {
        // -------------------------------------------------------------------------
        // 1. Handle Text (The "Odd One Out")
        // -------------------------------------------------------------------------
        if let Primitive::Text {
            content,
            position,
            size,
            line_height,
            bounds,
            horizontal_alignment,
            vertical_alignment,
            fill,
            font,
            rotation,
            ..
        } = primitive
        {
            frame.with_save(|frame| {
                // 1. Calculate Bounds & Clip Rect
                // We must offset the clip rectangle based on alignment so it matches the text placement.
                let (max_width, clip_rect) = if bounds.width.is_infinite() {
                    (f32::INFINITY, None)
                } else {
                    let x_origin = match horizontal_alignment {
                        Horizontal::Left => position.x,
                        Horizontal::Center => position.x - (bounds.width / 2.0),
                        Horizontal::Right => position.x - bounds.width,
                    };

                    let y_origin = match vertical_alignment {
                        Vertical::Top => position.y,
                        Vertical::Center => position.y - (bounds.height / 2.0),
                        Vertical::Bottom => position.y - bounds.height,
                    };

                    (
                        bounds.width,
                        Some(Rectangle::new(Point::new(x_origin, y_origin), *bounds)),
                    )
                };

                // 2. Rotation (Rotate around the generic anchor point)
                // Note: If you want to rotate the *box* around the specific alignment point,
                // this logic is correct.
                if *rotation != 0.0 {
                    frame.translate(Vector::new(position.x, position.y));
                    frame.rotate(*rotation);
                    frame.translate(Vector::new(-position.x, -position.y));
                }

                // 3. Draw Text
                let draw_text = |frame: &mut Frame<Renderer>| {
                    frame.fill_text(Text {
                        content: content.clone(),
                        position: *position, // Draw at anchor
                        color: *fill,
                        size: *size,
                        // Ensure this is Absolute to prevent the "Crazy Line Height"
                        line_height: *line_height,
                        font: *font,
                        align_x: (*horizontal_alignment).into(),
                        align_y: *vertical_alignment,
                        shaping: Shaping::Advanced,
                        max_width,
                    });
                };

                // 4. Clip & Execute
                if let Some(rect) = clip_rect {
                    frame.with_clip(rect, draw_text);
                } else {
                    draw_text(frame);
                }
            });
            // NOTE: Early return if text
            return;
        }

        // -------------------------------------------------------------------------
        // 2. The Shape Pipeline (Rect, Triangle, Ellipse)
        // -------------------------------------------------------------------------
        // Since we returned above, 'primitive' here is guaranteed to be a Shape.

        // A. Extract Styles
        let (fill, stroke) = match &primitive {
            Primitive::Rectangle { fill, stroke, .. } => (*fill, stroke.as_ref()),
            Primitive::Triangle { fill, stroke, .. } => (*fill, stroke.as_ref()),
            Primitive::Ellipse { fill, stroke, .. } => (*fill, stroke.as_ref()),
            Primitive::Polygon { fill, stroke, .. } => (*fill, stroke.as_ref()),
            Primitive::Line { stroke, .. } => (None, Some(stroke)),
            Primitive::HorizontalLine { stroke, .. } => (None, Some(stroke)),
            Primitive::VerticalLine { stroke, .. } => (None, Some(stroke)),
            Primitive::PolyLine { stroke, .. } => (None, Some(stroke)),
            Primitive::BezierCurve { stroke, .. } => (None, Some(stroke)),
            Primitive::Area { fill, stroke, .. } => (*fill, stroke.as_ref()),
            Primitive::Arc { fill, stroke, .. } => (*fill, stroke.as_ref()),
            Primitive::Spline { stroke, .. } => (None, Some(stroke)),
            Primitive::Text { .. } => (None, None),
        };

        // B. Build Geometry (Using the method on Primitive we added)
        let path = Path::new(|b| primitive.draw_geometry(b));

        // C. Render Fill
        if let Some(color) = fill {
            frame.fill(&path, color);
        }

        // D. Render Stroke (Using our helper)
        if let Some(s) = stroke {
            let mut dashed_storage = [0.0; 2];
            let iced_stroke = create_iced_stroke(s, &mut dashed_storage);
            frame.stroke(&path, iced_stroke);
        }
    }
}

// --- Helper Function ---
// This prevents code duplication between Rectangle, Triangle, Circle, etc.
fn create_iced_stroke<'a>(
    s: &ResolvedStroke,
    storage: &'a mut [f32; 2],
) -> iced_graphics::geometry::Stroke<'a> {
    let (segments, line_cap) = match s.style {
        StrokeStyle::Solid => (&[] as &[f32], LineCap::Butt),
        StrokeStyle::Dashed { dash, gap } => {
            storage[0] = dash * s.thickness;
            storage[1] = gap * s.thickness;
            (&storage[..], LineCap::Butt)
        }
        StrokeStyle::Dotted { gap } => {
            storage[0] = 0.0;
            storage[1] = gap * s.thickness;
            (&storage[..], LineCap::Round)
        }
    };

    Stroke {
        style: Style::Solid(s.fill),
        width: s.thickness,
        line_cap,
        line_join: LineJoin::Miter,
        line_dash: LineDash {
            segments,
            offset: 0,
        },
    }
}
