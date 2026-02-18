use crate::{radii::ResolvedRadii, stroke::ResolvedStroke};
use iced_core::{
    Color, Font, Pixels, Point, Radians, Rectangle, Size, Vector,
    alignment::{Horizontal, Vertical},
    text::{LineHeight, Wrapping},
};
use iced_graphics::geometry::path::Builder;
use iced_graphics::geometry::path::arc::Elliptical;
use std::f32::consts::PI;

/// Controls whether a line is extended to the edges of the plot bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineExtensions {
    /// Extend the line infinitely beyond its start point.
    pub start: bool,
    /// Extend the line infinitely beyond its end point.
    pub end: bool,
}

/// Controls arrowhead rendering at the ends of a line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineArrows {
    /// Draw an arrowhead at the start of the line.
    pub start: bool,
    /// Draw an arrowhead at the end of the line.
    pub end: bool,
    /// Arrowhead size.
    pub size: f32,
}

// Describes a **shared** primitive interface between the Mesh and Path backends.
pub enum Primitive {
    Rectangle {
        xy1: Point,
        xy2: Point,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Ellipse {
        center: Point,
        radii: ResolvedRadii,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Triangle {
        points: [Point; 3],
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Polygon {
        center: Point,
        radius: f32,
        vertices: u16,
        rotation: f32,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Line {
        start: Point,
        end: Point,
        stroke: ResolvedStroke,
        clip_bounds: Rectangle,
        extensions: LineExtensions,
        arrows: LineArrows,
    },
    HorizontalLine {
        y: f32,
        x_start: f32,
        x_end: f32,
        stroke: ResolvedStroke,
        snap: bool,
    },
    VerticalLine {
        x: f32,
        y_start: f32,
        y_end: f32,
        stroke: ResolvedStroke,
        snap: bool,
    },
    PolyLine {
        points: Vec<Point>,
        stroke: ResolvedStroke,
        clip_bounds: Rectangle,
        extensions: LineExtensions,
        arrows: LineArrows,
    },
    BezierCurve {
        start: Point,
        end: Point,
        control_1: Point,
        control_2: Option<Point>,
        stroke: ResolvedStroke,
    },
    Spline {
        points: Vec<Point>,
        stroke: ResolvedStroke,
        tension: f32,
    },
    Arc {
        center: Point,
        radius_inner: f32,
        radius_outer: f32,
        start_angle: f32,
        end_angle: f32,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Area {
        points: Vec<Point>,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Text {
        font: Font,
        content: String,
        position: Point,
        size: Pixels,
        rotation: f32,
        horizontal_alignment: Horizontal,
        vertical_alignment: Vertical,
        fill: Color,
        /// Override the quality tolerance of the text
        quality: Option<f32>,
        line_height: LineHeight,
        bounds: Size,
        wrapping: Wrapping,
    },
}

impl Primitive {
    /// Draws the geometry of this primitive into the provided Iced Path Builder.
    pub fn draw_geometry(&self, builder: &mut iced_graphics::geometry::path::Builder) {
        match self {
            Self::Rectangle { xy1, xy2, .. } => {
                let x = xy1.x.min(xy2.x);
                let y = xy1.y.min(xy2.y);
                let w = (xy1.x - xy2.x).abs();
                let h = (xy1.y - xy2.y).abs();

                builder.rectangle(Point::new(x, y), Size::new(w, h));
            }

            Self::Triangle { points, .. } => {
                builder.move_to(points[0]);
                builder.line_to(points[1]);
                builder.line_to(points[2]);
                builder.close();
            }

            Self::Ellipse { center, radii, .. } => {
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
            }

            Self::Polygon {
                center,
                radius,
                vertices,
                rotation,
                ..
            } => {
                let count = *vertices as usize;
                if count < 3 {
                    return;
                }

                // Convert degrees to radians and adjust so 0 deg = North (Up)
                // Standard trig 0 is East (Right). North is -90 deg (-PI/2).
                let start_angle = (rotation - 90.0).to_radians();
                let step = (std::f32::consts::PI * 2.0) / (*vertices as f32);

                for i in 0..count {
                    let angle = start_angle + (step * i as f32);
                    let px = center.x + radius * angle.cos();
                    let py = center.y + radius * angle.sin();
                    let point = Point::new(px, py);

                    if i == 0 {
                        builder.move_to(point);
                    } else {
                        builder.line_to(point);
                    }
                }
                builder.close();
            }
            Self::Line {
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
                    if let Some((c1, c2)) = clip_infinite_line(p1, p2, *clip_bounds) {
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

                // 2. Draw Line Body
                builder.move_to(p1);
                builder.line_to(p2);

                // 3. Draw Arrows
                // Calculate length squared manually
                let d = p2 - p1;
                let len_sq = d.x.mul_add(d.x, d.y * d.y);

                if len_sq > 0.000001 {
                    let len = len_sq.sqrt();
                    // Normalize manually: vector / length
                    let norm_dir = Vector::new(d.x / len, d.y / len);
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
            }

            // ... (Horizontal/Vertical Lines remain simple) ...

            // -----------------------------------------------------------------
            // PolyLine (Connected Segments)
            // -----------------------------------------------------------------
            Self::PolyLine {
                points,
                stroke,
                clip_bounds,
                extensions,
                arrows,
            } => {
                if points.len() < 2 {
                    return;
                }

                let mut p_first = points[0];
                let mut p_last = points[points.len() - 1];
                let last_idx = points.len() - 1;

                // 1. Handle Infinite Extensions (First/Last segment only)
                if extensions.start {
                    let p_next = points[1];
                    if let Some((edge, _)) = clip_infinite_line(p_first, p_next, *clip_bounds) {
                        p_first = edge;
                    }
                }
                if extensions.end
                    && let Some((_, edge)) =
                        clip_infinite_line(points[last_idx - 1], p_last, *clip_bounds)
                {
                    p_last = edge;
                }

                // 2. Draw Chain
                builder.move_to(p_first);
                for point in points.iter().take(last_idx).skip(1) {
                    builder.line_to(*point);
                }
                builder.line_to(p_last);

                // 3. Draw Arrows
                let arrow_len = stroke.thickness * arrows.size;

                if arrows.start && !extensions.start {
                    let d = points[1] - p_first;
                    let len = d.x.hypot(d.y);

                    if len > 0.001 {
                        let dir = Vector::new(d.x / len, d.y / len);
                        let neg_dir = Vector::new(-dir.x, -dir.y);

                        let (w1, w2) = calculate_arrowhead(p_first, neg_dir, arrow_len);
                        builder.move_to(p_first);
                        builder.line_to(w1);
                        builder.move_to(p_first);
                        builder.line_to(w2);
                    }
                }

                if arrows.end && !extensions.end {
                    let d = p_last - points[last_idx - 1];
                    let len = d.x.hypot(d.y);

                    if len > 0.001 {
                        let dir = Vector::new(d.x / len, d.y / len);
                        let (w1, w2) = calculate_arrowhead(p_last, dir, arrow_len);
                        builder.move_to(p_last);
                        builder.line_to(w1);
                        builder.move_to(p_last);
                        builder.line_to(w2);
                    }
                }
            }

            Self::BezierCurve {
                start,
                end,
                control_1,
                control_2,
                ..
            } => {
                builder.move_to(*start);

                if let Some(c2) = control_2 {
                    // Cubic Bezier (2 Control Points)
                    // Note: 'bezier_curve_to' is the standard name for cubic in Iced/Canvas
                    builder.bezier_curve_to(*control_1, *c2, *end);
                } else {
                    // Quadratic Bezier (1 Control Point)
                    builder.quadratic_curve_to(*control_1, *end);
                }
            }

            // -----------------------------------------------------------------
            // Area (Arbitrary Polygon)
            // -----------------------------------------------------------------
            Self::Area { points, .. } => {
                if points.len() < 3 {
                    return;
                }
                builder.move_to(points[0]);
                for p in points.iter().skip(1) {
                    builder.line_to(*p);
                }
                builder.close();
            }

            // -----------------------------------------------------------------
            // Spline (Smooth Curve)
            // -----------------------------------------------------------------
            Self::Spline {
                points, tension, ..
            } => {
                if points.len() < 2 {
                    return;
                }

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

                    let (c1, c2) = catmull_to_bezier(p0, p1, p2, p3, *tension);
                    builder.bezier_curve_to(c1, c2, p2);
                }
            }

            // -----------------------------------------------------------------
            // Arc (Pie Slice or Donut)
            // -----------------------------------------------------------------
            Self::Arc {
                center,
                radius_inner,
                radius_outer,
                start_angle,
                end_angle,
                ..
            } => {
                // A. Draw Outer Arc (Clockwise)
                let start_outer = Point::new(
                    center.x + radius_outer * start_angle.cos(),
                    center.y + radius_outer * start_angle.sin(),
                );
                builder.move_to(start_outer);

                add_arc_using_beziers(
                    builder,
                    *center,
                    *radius_outer,
                    *start_angle,
                    *end_angle,
                    true, // Clockwise
                );

                // B. Draw Inner Arc (Hole vs Pie)
                if *radius_inner < 0.5 {
                    // Pie Slice: Just go to center
                    builder.line_to(*center);
                } else {
                    // Donut: Line to inner end
                    let end_inner = Point::new(
                        center.x + radius_inner * end_angle.cos(),
                        center.y + radius_inner * end_angle.sin(),
                    );
                    builder.line_to(end_inner);

                    // Draw Inner Arc (Counter-Clockwise) to create the hole!
                    // Note: We pass 'false' for clockwise
                    add_arc_using_beziers(
                        builder,
                        *center,
                        *radius_inner,
                        *end_angle,
                        *start_angle,
                        false,
                    );
                }

                builder.close();
            }

            // Text and other non-path primitives are handled elsewhere
            _ => {}
        }
    }
}

// Helper to calculate arrow wing points
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

// Helper to clip an infinite line to a rectangle
// Returns the two points where the line intersects the box edges
fn clip_infinite_line(p1: Point, p2: Point, bounds: Rectangle) -> Option<(Point, Point)> {
    // Basic Liang-Barsky or similar line clipping algorithm is standard here.
    // For simplicity, we can calculate intersection with all 4 edges.
    //
    // Equation: P = P1 + t * (P2 - P1)
    // We find t for x=min, x=max, y=min, y=max.

    let d = p2 - p1;
    if d.x.abs() < 1e-6 && d.y.abs() < 1e-6 {
        return None;
    } // Point, not line

    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    // Checks (p, q) where p*t <= q
    let mut check = |p: f32, q: f32| -> bool {
        if p == 0.0 {
            return q >= 0.0; // Parallel line
        }
        let t = q / p;
        if p < 0.0 {
            if t > t_min {
                t_min = t;
            }
        } else if t < t_max {
            t_max = t;
        }
        t_min <= t_max
    };

    // Clip against left/right
    if !check(-d.x, p1.x - bounds.x) {
        return None;
    }
    if !check(d.x, bounds.x + bounds.width - p1.x) {
        return None;
    }
    // Clip against top/bottom
    if !check(-d.y, p1.y - bounds.y) {
        return None;
    }
    if !check(d.y, bounds.y + bounds.height - p1.y) {
        return None;
    }

    // If t_min > t_max, line is outside
    if t_min > t_max {
        return None;
    }

    // Infinite lines extend effectively from -inf to +inf,
    // so we clamp to the box range we found.
    let start = Point::new(t_min.mul_add(d.x, p1.x), t_min.mul_add(d.y, p1.y));
    let end = Point::new(t_max.mul_add(d.x, p1.x), t_max.mul_add(d.y, p1.y));

    Some((start, end))
}

// Helper to convert Catmull-Rom points to Cubic Bezier control points
// p0=prev, p1=start, p2=end, p3=next
fn catmull_to_bezier(p0: Point, p1: Point, p2: Point, p3: Point, tension: f32) -> (Point, Point) {
    // Tension: 0.0 = Smooth, 1.0 = Straight
    let t = (1.0 - tension) / 2.0;

    // Tangent at p1
    let tx1 = (p2.x - p0.x) * t;
    let ty1 = (p2.y - p0.y) * t;

    // Tangent at p2
    let tx2 = (p3.x - p1.x) * t;
    let ty2 = (p3.y - p1.y) * t;

    // Control Points
    let c1 = Point::new(p1.x + tx1 / 3.0, p1.y + ty1 / 3.0);
    let c2 = Point::new(p2.x - tx2 / 3.0, p2.y - ty2 / 3.0);

    (c1, c2)
}

// 1. Add this Helper Function (Math for approximating arcs with Beziers)
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

        // Draw
        if i == 0 {
            // Only move to start if we aren't already there (optional, but safer to be explicit for arcs)
            // For the donut logic below, we handle move_to explicitly before calling this.
            // builder.move_to(p0);
        }
        builder.bezier_curve_to(c1, c2, p3);
    }
}
