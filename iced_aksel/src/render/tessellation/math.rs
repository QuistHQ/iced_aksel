//! Geometric utility functions.
//!
//! Shared math helpers for generating shapes, calculating intersections, and normalizing vectors.

use iced_core::{Point, Vector};

/// Represents an Axis-Aligned Bounding Box (AABB) defined by min/max coordinates.
///
/// This is preferred over `Rectangle` for intersection math because it explicitly
/// stores the boundaries, avoiding repeated `x + width` calculations.
#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl Bounds {
    /// Creates new bounds from an iced Rectangle, optionally expanding it by a margin.
    pub fn new(rect: iced_core::Rectangle, margin: f32) -> Self {
        Self {
            min_x: rect.x - margin,
            min_y: rect.y - margin,
            max_x: rect.x + rect.width + margin,
            max_y: rect.y + rect.height + margin,
        }
    }
}

/// Normalizes a 2D vector (makes its length 1.0).
///
/// If the vector length is near zero, returns a zero vector.
#[inline]
pub fn normalize(v: Vector) -> Vector {
    let len_sq = v.x * v.x + v.y * v.y;
    if len_sq < 1e-8 {
        Vector::new(0.0, 0.0)
    } else {
        let len = len_sq.sqrt();
        Vector::new(v.x / len, v.y / len)
    }
}

/// Checks if a polygon defined by a set of points is convex.
///
/// Uses the cross-product method to check if all turns are in the same direction.
pub fn is_convex(points: &[Point]) -> bool {
    if points.len() < 4 {
        return true; // Triangles are always convex
    }
    let n = points.len();
    let mut sign = 0.0;

    for i in 0..n {
        let p1 = points[i];
        let p2 = points[(i + 1) % n];
        let p3 = points[(i + 2) % n];

        let dx1 = p2.x - p1.x;
        let dy1 = p2.y - p1.y;
        let dx2 = p3.x - p2.x;
        let dy2 = p3.y - p2.y;

        // 2D Cross Product (Z component)
        let cross_z = dx1 * dy2 - dy1 * dx2;

        if cross_z.abs() < 1e-5 {
            continue;
        } // Collinear points ignore

        if sign == 0.0 {
            sign = cross_z;
        } else if cross_z * sign < 0.0 {
            return false; // Sign flipped, meaning concavity detected
        }
    }
    true
}

/// Generates a ring of points representing a regular polygon.
///
/// * `center`: The center of the polygon.
/// * `radius`: Distance from center to vertex.
/// * `vertices`: Number of sides.
/// * `rotation`: Rotation in degrees (converted to radians internally).
pub fn generate_ring(center: Point, radius: f32, vertices: u16, rotation: f32) -> Vec<Point> {
    let mut points = Vec::with_capacity(vertices as usize);
    // Convert degrees to radians and adjust so 0 degrees is North (standard chart behavior)
    let start_angle = (rotation - 90.0).to_radians();
    let step = std::f32::consts::TAU / vertices as f32;

    for i in 0..vertices {
        let theta = start_angle + (i as f32 * step);
        let (sin, cos) = theta.sin_cos();
        // Point + Vector (implicitly constructed)
        points.push(Point::new(
            cos.mul_add(radius, center.x),
            sin.mul_add(radius, center.y),
        ));
    }
    points
}

/// Computes a new vertex inset (moved inwards) from a corner of a polygon.
///
/// Used for shrinking a polygon to create a stroke border.
/// **Includes Miter Limiting to prevent sharp angles from exploding.**
///
/// * `prev`, `curr`, `next`: The three points defining the corner at `curr`.
/// * `distance`: How far to move inwards.
pub fn compute_inset_vertex(prev: Point, curr: Point, next: Point, distance: f32) -> Point {
    // 1. Calculate normalized direction vectors for the two edges
    // (Point - Point = Vector)
    let v1 = normalize(curr - prev);
    let v2 = normalize(next - curr);

    // 2. Calculate the perpendicular normal for the first edge (90 deg rotation)
    let n1 = Vector::new(-v1.y, v1.x);

    // 3. Calculate the sine of the angle between the two vectors.
    // If this is close to 0, the lines are parallel or the angle is 180.
    let corner_sin = (v1.x * v2.y - v1.y * v2.x).abs();

    // SAFETY CHECK 1: Parallel lines (Angle ~ 0 or 180)
    // Just shift perpendicularly, don't try to compute a corner.
    if corner_sin < 0.001 {
        return curr + n1 * distance;
    }

    // 4. Calculate the "Miter" vector (the direction of the corner bisection)
    let combined = v1 - v2;
    let len_sq = combined.x * combined.x + combined.y * combined.y;
    let len = len_sq.sqrt();

    // SAFETY CHECK 2: Degenerate geometry (points on top of each other)
    if len < 1e-4 {
        return curr + n1 * distance;
    }

    // 5. Calculate Miter Length
    // Standard formula: length = distance / sin(theta/2)
    // Derived here via vector algebra
    let miter_len = 2.0 * distance / (len * corner_sin);

    // 6. THE FIX: Miter Clamping
    // If the required offset is more than 5x the stroke width, we clamp it.
    // This prevents the "explosion" where the vertex shoots off to infinity.
    let miter_limit = distance * 5.0;
    let clamped_len = miter_len.min(miter_limit);

    let miter = combined * clamped_len;

    // 7. Apply offset based on winding order (Convexity check)
    let cross = v1.x * v2.y - v1.y * v2.x;
    if cross < 0.0 {
        curr - miter
    } else {
        curr + miter
    }
}

/// Computes a new polygon that is smaller (inset) than the original.
///
/// * `points`: The original polygon vertices.
/// * `distance`: The inset distance.
pub fn compute_inset_polygon(points: &[Point], distance: f32) -> Vec<Point> {
    let len = points.len();
    let mut new_points = Vec::with_capacity(len);

    for i in 0..len {
        let prev = points[(i + len - 1) % len];
        let curr = points[i];
        let next = points[(i + 1) % len];

        new_points.push(compute_inset_vertex(prev, curr, next, distance));
    }
    new_points
}

/// Calculates the Cubic Bézier control points for a Catmull-Rom spline segment.
///
/// Given four points (p0, p1, p2, p3), this calculates the curve segment between p1 and p2.
///
/// * `tension`: Controls the "tightness" of the curve.
///   * `0.0` = Catmull-Rom (Standard smooth).
///   * `1.0` = Linear (Straight lines).
///
/// Returns `(ControlPoint1, ControlPoint2)`.
pub fn catmull_rom_to_bezier(
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
    tension: f32,
) -> (Point, Point) {
    let alpha = 0.5 * (1.0 - tension);

    // Tangent vectors
    let tangent1 = (p2 - p0) * alpha;
    let tangent2 = (p3 - p1) * alpha;

    // Convert tangents to Bézier control points
    let c1 = p1 + tangent1 * (1.0 / 3.0);
    let c2 = p2 - tangent2 * (1.0 / 3.0);

    (c1, c2)
}

/// Calculates the intersection of an **Infinite Line** with a bounding box.
///
/// Returns `Some((entry_point, exit_point))` if the line is visible.
pub fn clip_infinite_line(
    line_start: Point,
    line_end: Point,
    bounds: Bounds,
) -> Option<(Point, Point)> {
    let delta_x = line_end.x - line_start.x;
    let delta_y = line_end.y - line_start.y;

    // Check for vertical lines
    if delta_x.abs() < 1e-6 {
        if line_start.x < bounds.min_x || line_start.x > bounds.max_x {
            return None;
        }
        return Some((
            Point::new(line_start.x, bounds.min_y),
            Point::new(line_start.x, bounds.max_y),
        ));
    }
    // Check for horizontal lines
    if delta_y.abs() < 1e-6 {
        if line_start.y < bounds.min_y || line_start.y > bounds.max_y {
            return None;
        }
        return Some((
            Point::new(bounds.min_x, line_start.y),
            Point::new(bounds.max_x, line_start.y),
        ));
    }

    // Calculate intersection parameter 't' for all 4 boundaries
    let t_at_min_x = (bounds.min_x - line_start.x) / delta_x;
    let t_at_max_x = (bounds.max_x - line_start.x) / delta_x;
    let t_at_min_y = (bounds.min_y - line_start.y) / delta_y;
    let t_at_max_y = (bounds.max_y - line_start.y) / delta_y;

    // Find the range of 't' that is inside the X boundaries
    let (t_enter_x, t_exit_x) = if delta_x > 0.0 {
        (t_at_min_x, t_at_max_x)
    } else {
        (t_at_max_x, t_at_min_x)
    };

    // Find the range of 't' that is inside the Y boundaries
    let (t_enter_y, t_exit_y) = if delta_y > 0.0 {
        (t_at_min_y, t_at_max_y)
    } else {
        (t_at_max_y, t_at_min_y)
    };

    let t_entry = t_enter_x.max(t_enter_y);
    let t_exit = t_exit_x.min(t_exit_y);

    if t_entry > t_exit {
        return None;
    }

    Some((
        Point::new(
            line_start.x + delta_x * t_entry,
            line_start.y + delta_y * t_entry,
        ),
        Point::new(
            line_start.x + delta_x * t_exit,
            line_start.y + delta_y * t_exit,
        ),
    ))
}

/// Standard Liang-Barsky clipping for a finite line segment.
pub fn clip_segment(start: Point, end: Point, bounds: Bounds) -> Option<(Point, Point)> {
    let delta_x = end.x - start.x;
    let delta_y = end.y - start.y;

    let p_components = [-delta_x, delta_x, -delta_y, delta_y];
    let q_distances = [
        start.x - bounds.min_x,
        bounds.max_x - start.x,
        start.y - bounds.min_y,
        bounds.max_y - start.y,
    ];

    let mut t_enter = 0.0;
    let mut t_exit = 1.0;

    for i in 0..4 {
        let p = p_components[i];
        let q = q_distances[i];

        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                if t > t_exit {
                    return None;
                }
                if t > t_enter {
                    t_enter = t;
                }
            } else {
                if t < t_enter {
                    return None;
                }
                if t < t_exit {
                    t_exit = t;
                }
            }
        }
    }

    if t_enter > t_exit {
        return None;
    }

    Some((
        Point::new(start.x + delta_x * t_enter, start.y + delta_y * t_enter),
        Point::new(start.x + delta_x * t_exit, start.y + delta_y * t_exit),
    ))
}
