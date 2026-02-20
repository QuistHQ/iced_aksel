use std::f32::consts::PI;

use super::Primitive;
use super::math::{Bounds, catmull_rom_to_bezier, clip_infinite_line, normalize};

use crate::stroke::{ResolvedStroke, StrokeStyle};
use iced_core::alignment::{Horizontal, Vertical};
use iced_core::text::Shaping;
use iced_core::{Point, Radians, Rectangle, Size, Vector};
use iced_graphics::geometry::path::Builder;
use iced_graphics::geometry::path::arc::Elliptical;
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
                self.buffer
                    .iter()
                    .for_each(|primitive| Self::draw_primitive(primitive, frame))
            });

        self.needs_redraw = false;
        renderer.draw_geometry(geometry);
    }

    fn draw_primitive(primitive: &Primitive, frame: &mut Frame<Renderer>) {
        let (path, fill, stroke) = match primitive {
            Primitive::Rectangle {
                xy1,
                xy2,
                fill,
                stroke,
            } => {
                let path = Path::new(|builder| {
                    let x = xy1.x.min(xy2.x);
                    let y = xy1.y.min(xy2.y);
                    let w = (xy1.x - xy2.x).abs();
                    let h = (xy1.y - xy2.y).abs();

                    builder.rectangle(Point::new(x, y), Size::new(w, h));
                });
                (path, *fill, stroke.as_ref())
            }
            Primitive::Triangle {
                points,
                fill,
                stroke,
            } => {
                let path = Path::new(|builder| {
                    builder.move_to(points[0]);
                    builder.line_to(points[1]);
                    builder.line_to(points[2]);
                    builder.close();
                });
                (path, *fill, stroke.as_ref())
            }
            Primitive::Ellipse {
                center,
                radii,
                fill,
                stroke,
            } => {
                let path = Path::new(|builder| {
                    // Optimization: Use native circle if radii is_uniform
                    if radii.is_uniform() {
                        builder.circle(*center, radii.x);
                    } else {
                        // Stretched Ellipse
                        builder.ellipse(Elliptical {
                            center: *center,
                            radii: Vector::new(radii.x, radii.y),
                            rotation: Radians(0.0),
                            start_angle: Radians(0.0),
                            end_angle: Radians(2.0 * PI),
                        });
                    }
                });

                (path, *fill, stroke.as_ref())
            }
            Primitive::Polygon {
                center,
                radius,
                vertices,
                rotation,
                fill,
                stroke,
            } => {
                let count = *vertices as usize;
                if count < 3 {
                    return;
                }

                let path = Path::new(|builder| {
                    // Convert degrees to radians and adjust so 0 deg = North (Up)
                    // Standard trig 0 is East (Right). North is -90 deg (-PI/2).
                    let start_angle = rotation.0;
                    let step = (std::f32::consts::PI * 2.0) / (*vertices as f32);

                    for i in 0..count {
                        let angle = start_angle + (step * i as f32);
                        let px = radius.0.mul_add(angle.cos(), center.x);
                        let py = radius.0.mul_add(angle.sin(), center.y);
                        let point = Point::new(px, py);

                        if i == 0 {
                            builder.move_to(point);
                        } else {
                            builder.line_to(point);
                        }
                    }
                    builder.close();
                });

                (path, *fill, stroke.as_ref())
            }
            Primitive::Line {
                start,
                end,
                stroke,
                clip_bounds,
                extensions,
                arrows,
            } => {
                let mut p1 = *start;
                let mut p2 = *end;

                // 1. Handle Infinite Extensions
                if extensions.start || extensions.end {
                    if let Some((c1, c2)) = clip_infinite_line(p1, p2, Bounds::new(*clip_bounds, 0.0)) {
                        // Dot product manually: (dx * dx) + (dy * dy)
                        let d_orig = p2 - p1;
                        let d_clip = c2 - c1;
                        let dot = d_orig.x.mul_add(d_clip.x, d_orig.y * d_clip.y);

                        // Align the clip points with the line direction
                        if dot < 0.0 {
                            if extensions.start {
                                p1 = c2;
                            }
                            if extensions.end {
                                p2 = c1;
                            }
                        } else {
                            if extensions.start {
                                p1 = c1;
                            }
                            if extensions.end {
                                p2 = c2;
                            }
                        }
                    } else {
                        return; // Line is completely off-screen
                    }
                }

                let path = Path::new(|builder| {
                    // 2. Draw Line Body
                    builder.move_to(p1);
                    builder.line_to(p2);

                    // 3. Draw Arrows
                    let d = p2 - p1;
                    let norm_dir = normalize(d);

                    if norm_dir.x != 0.0 || norm_dir.y != 0.0 {
                        let arrow_len = stroke.thickness * arrows.size;

                        if arrows.start && !extensions.start {
                            // Negative direction for start arrow
                            let neg_dir = Vector::new(-norm_dir.x, -norm_dir.y);
                            let (w1, w2) = calculate_arrowhead(p1, neg_dir, arrow_len);
                            builder.move_to(p1);
                            builder.line_to(w1);
                            builder.move_to(p1);
                            builder.line_to(w2);
                        }
                        if arrows.end && !extensions.end {
                            let (w1, w2) = calculate_arrowhead(p2, norm_dir, arrow_len);
                            builder.move_to(p2);
                            builder.line_to(w1);
                            builder.move_to(p2);
                            builder.line_to(w2);
                        }
                    }
                });

                (path, Some(stroke.fill), Some(stroke))
            }
            Primitive::HorizontalLine {
                y,
                x_start,
                x_end,
                stroke,
                .. // We can't pixel-snap on tiny-skia
            } => {
                let path = Path::new(|builder| {
                    builder.move_to(Point::new(*x_start, *y));
                    builder.line_to(Point::new(*x_end, *y));
                });
                (path, Some(stroke.fill), Some(stroke))
            }
            Primitive::VerticalLine {
                x,
                y_start,
                y_end,
                stroke,
                .. // We can't pixel-snap on tiny-skia
            } => {
                let path = Path::new(|builder| {
                    builder.move_to(Point::new(*x, *y_start));
                    builder.line_to(Point::new(*x, *y_end));
                });
                (path, Some(stroke.fill), Some(stroke))
            }
            Primitive::PolyLine {
                points,
                stroke,
                clip_bounds,
                extensions,
                arrows,
            } => {
                if points.len() < 2 {
                    return;
                }

                let path = Path::new(|builder| {
                    let mut p_first = points[0];
                    let mut p_last = points[points.len() - 1];
                    let last_idx = points.len() - 1;

                    // 1. Handle Infinite Extensions (First/Last segment only)
                    if extensions.start {
                        let p_next = points[1];
                        if let Some((edge, _)) = clip_infinite_line(p_first, p_next, Bounds::new(*clip_bounds, 0.0)) {
                            p_first = edge;
                        }
                    }
                    if extensions.end
                        && let Some((_, edge)) =
                            clip_infinite_line(points[last_idx - 1], p_last, Bounds::new(*clip_bounds, 0.0))
                    {
                        p_last = edge;
                    }

                    // 2. Draw Chain
                    //
                    // We draw p_first and p_last seperately from the other points, as the first
                    // and last might have been changed due to infinite line extensions.
                    builder.move_to(p_first);
                    points.iter().skip(1).take(last_idx - 1).for_each(|point| builder.line_to(*point));
                    builder.line_to(p_last);

                    // 3. Draw Arrows
                    let arrow_len = stroke.thickness * arrows.size;
                    if arrows.start && !extensions.start {
                        let dir = normalize(points[1] - p_first);
                        if dir.x != 0.0 || dir.y != 0.0 {
                            let neg_dir = Vector::new(-dir.x, -dir.y);
                            let (w1, w2) = calculate_arrowhead(p_first, neg_dir, arrow_len);
                            builder.move_to(p_first);
                            builder.line_to(w1);
                            builder.move_to(p_first);
                            builder.line_to(w2);
                        }
                    }

                    if arrows.end && !extensions.end {
                        let dir = normalize(p_last - points[last_idx - 1]);
                        if dir.x != 0.0 || dir.y != 0.0 {
                            let (w1, w2) = calculate_arrowhead(p_last, dir, arrow_len);
                            builder.move_to(p_last);
                            builder.line_to(w1);
                            builder.move_to(p_last);
                            builder.line_to(w2);
                        }
                    }
                });

                (path, None, Some(stroke))
            }

            Primitive::BezierCurve {
                start,
                end,
                control_1,
                control_2,
                stroke,
            } => {
                let path = Path::new(|builder| {
                    builder.move_to(*start);

                    if let Some(c2) = control_2 {
                        // Cubic Bezier (2 Control Points)
                        // Note: 'bezier_curve_to' is the standard name for cubic in Iced/Canvas
                        builder.bezier_curve_to(*control_1, *c2, *end);
                    } else {
                        // Quadratic Bezier (1 Control Point)
                        builder.quadratic_curve_to(*control_1, *end);
                    }
                });

                (path, None, Some(stroke))
            }

            // -----------------------------------------------------------------
            // Area (Arbitrary Polygon)
            // -----------------------------------------------------------------
            Primitive::Area {
                points,
                fill,
                stroke,
            } => {
                if points.len() < 3 {
                    return;
                }

                let path = Path::new(|builder| {
                    builder.move_to(points[0]);
                    for p in points.iter().skip(1) {
                        builder.line_to(*p);
                    }
                    builder.close();
                });

                (path, *fill, stroke.as_ref())
            }

            // -----------------------------------------------------------------
            // Spline (Smooth Curve)
            // -----------------------------------------------------------------
            Primitive::Spline {
                points,
                tension,
                stroke,
            } => {
                if points.len() < 2 {
                    return;
                }

                let path = Path::new(|builder| {
                    builder.move_to(points[0]);

                    if points.len() == 2 {
                        builder.line_to(points[1]);
                        return;
                    }

                    for i in 0..points.len() - 1 {
                        let p0 = if i == 0 { points[0] } else { points[i - 1] };
                        let p1 = points[i];
                        let p2 = points[i + 1];
                        let p3 = if i + 2 < points.len() {
                            points[i + 2]
                        } else {
                            p2
                        };

                        let (c1, c2) = catmull_rom_to_bezier(p0, p1, p2, p3, *tension);
                        builder.bezier_curve_to(c1, c2, p2);
                    }
                });

                (path, None, Some(stroke))
            }

            // -----------------------------------------------------------------
            // Arc (Pie Slice or Donut)
            // -----------------------------------------------------------------
            Primitive::Arc {
                center,
                radius_inner,
                radius_outer,
                start_angle,
                end_angle,
                fill,
                stroke,
            } => {
                let path = Path::new(|builder| {
                    // A. Draw Outer Arc (Clockwise)
                    let start_outer = Point::new(
                        radius_outer.0.mul_add(start_angle.0.cos(), center.x),
                        radius_outer.0.mul_add(start_angle.0.sin(), center.y),
                    );
                    builder.move_to(start_outer);

                    add_arc_using_beziers(
                        builder,
                        *center,
                        radius_outer.0,
                        start_angle.0,
                        end_angle.0,
                        true, // Clockwise
                    );

                    let radius_inner = radius_inner.map(|r| r.0).unwrap_or(0.0);

                    // B. Draw Inner Arc (Hole vs Pie)
                    if radius_inner < 0.5 {
                        // Pie Slice: Just go to center
                        builder.line_to(*center);
                    } else {
                        // Donut: Line to inner end
                        let end_inner = Point::new(
                            center.x + radius_inner * end_angle.0.cos(),
                            center.y + radius_inner * end_angle.0.sin(),
                        );
                        builder.line_to(end_inner);

                        // Draw Inner Arc (Counter-Clockwise) to create the hole!
                        // Note: We pass 'false' for clockwise
                        add_arc_using_beziers(
                            builder,
                            *center,
                            radius_inner,
                            end_angle.0,
                            start_angle.0,
                            false,
                        );
                    }

                    builder.close();
                });
                (path, *fill, stroke.as_ref())
            }
            Primitive::Text {
                font,
                content,
                position,
                size,
                rotation,
                horizontal_alignment,
                vertical_alignment,
                fill,
                line_height,
                bounds,
                .. // Quality and Wrapping can't be supported on the tiny-skia pipeline
            } => {
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
                // NOTE: Text returns early, as no stroke/fill shall be rendered on Text.
                return;
            }
        };

        // Render Fill
        if let Some(color) = fill {
            frame.fill(&path, color);
        }

        // Render Stroke (Using our helper)
        if let Some(s) = stroke {
            let mut dashed_storage = [0.0; 2];
            let iced_stroke = create_iced_stroke(s, &mut dashed_storage);
            frame.stroke(&path, iced_stroke);
        }
    }
}

/// Helper to calculate arrow wing points
fn calculate_arrowhead(tip: Point, direction: Vector, size: f32) -> (Point, Point) {
    // Arrow is usually an isosceles triangle pointing at 'tip'.
    // We rotate the reverse direction vector by +/- 30 degrees (roughly 0.5 radians).
    let angle: f32 = 0.5;
    let cos = angle.cos();
    let sin = angle.sin();

    // Reverse direction scaled by size
    let rx = -direction.x * size;
    let ry = -direction.y * size;

    // Rotate +angle
    let p1 = Point::new(
        tip.x + rx.mul_add(cos, -(ry * sin)),
        tip.y + rx.mul_add(sin, ry * cos),
    );

    // Rotate -angle
    let p2 = Point::new(
        tip.x + rx.mul_add(cos, ry * sin),  // sin(-a) = -sin(a)
        tip.y + rx.mul_add(-sin, ry * cos), // cos(-a) = cos(a)
    );

    (p1, p2)
}

/// Arc Helper Function (Math for approximating arcs with Beziers)
fn add_arc_using_beziers(
    builder: &mut Builder,
    center: Point,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    clockwise: bool,
) {
    // We split the arc into smaller segments (max 90 degrees) for accuracy
    let mut total_sweep = end_angle - start_angle;

    // Normalize sweep based on direction
    if clockwise {
        if total_sweep < 0.0 {
            total_sweep += 2.0 * std::f32::consts::PI;
        }
    } else if total_sweep > 0.0 {
        total_sweep -= 2.0 * std::f32::consts::PI;
    }

    let num_segments = (total_sweep.abs() / (std::f32::consts::PI / 2.0)).ceil() as usize;

    // No segments, so we don't draw
    if num_segments == 0 {
        return;
    }

    let step = total_sweep / num_segments as f32;

    for i in 0..num_segments {
        let current_start = start_angle + (step * i as f32);
        let current_end = current_start + step;

        // Math: Control points for a circular arc
        // k = (4/3) * tan(theta / 4)
        let half_angle = step / 2.0;
        let k = radius * (4.0 / 3.0) * (half_angle / 2.0).tan();

        // Start/End points of this segment
        let p0 = Point::new(
            radius.mul_add(current_start.cos(), center.x),
            radius.mul_add(current_start.sin(), center.y),
        );
        let p3 = Point::new(
            radius.mul_add(current_end.cos(), center.x),
            radius.mul_add(current_end.sin(), center.y),
        );

        // Control Points (Tangent to the circle)
        // Tangent vector is (-y, x) relative to radius vector
        let c1 = Point::new(
            p0.x - k * current_start.sin(),
            p0.y + k * current_start.cos(),
        );
        let c2 = Point::new(p3.x + k * current_end.sin(), p3.y - k * current_end.cos());

        builder.bezier_curve_to(c1, c2, p3);
    }
}

/// Stroke helper: This prevents code duplication between Rectangle, Triangle, Circle, etc.
fn create_iced_stroke<'a>(
    s: &ResolvedStroke,
    storage: &'a mut [f32; 2],
) -> iced_graphics::geometry::Stroke<'a> {
    let (segments, line_cap) = match s.style {
        StrokeStyle::Solid => (&[] as &[f32], LineCap::Butt),
        StrokeStyle::Dashed { dash, gap } => {
            storage[0] = dash;
            storage[1] = gap;
            (&storage[..], LineCap::Butt)
        }
        StrokeStyle::Dotted { gap } => {
            storage[0] = 0.0;
            storage[1] = gap;
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
