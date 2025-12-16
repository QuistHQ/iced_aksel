//! Geometric utility functions.
//!
//! Shared math helpers for generating shapes, calculating intersections, and normalizing vectors.

use iced_core::{Point, Vector};

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
///
/// * `prev`, `curr`, `next`: The three points defining the corner at `curr`.
/// * `distance`: How far to move inwards.
pub fn compute_inset_vertex(prev: Point, curr: Point, next: Point, distance: f32) -> Point {
    // Vectors for the two edges
    let v1 = normalize(curr - prev);
    let v2 = normalize(next - curr);

    // Perpendicular vectors (normals) pointing inwards
    // Rotate 90 degrees CCW: (x, y) -> (-y, x)
    let n1 = Vector::new(-v1.y, v1.x);

    // Safety check for parallel lines
    let corner_sin = (v1.x * v2.y - v1.y * v2.x).abs();
    if corner_sin < 0.001 {
        // Parallel lines, just shift perpendicular
        return curr + n1 * distance;
    }

    // Standard miter offset formula
    let combined = v1 - v2;
    let len_sq = combined.x * combined.x + combined.y * combined.y;
    let len = len_sq.sqrt();

    if len < 1e-4 {
        return curr + n1 * distance;
    }

    let miter = combined * (2.0 * distance / (len * corner_sin));

    // Determine sign based on convexity (cross product) to ensure we go "in"
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

/// Clipping Algorithm: Liang-Barsky.
///
/// Calculates the intersection of a line segment `p1`->`p2` with a rectangle `rect`.
/// Returns `Some((t0, t1))` where `t` are scalar values along the line segment (0.0 to 1.0).
pub fn clip_line_liang_barsky(
    p1: Point,
    p2: Point,
    rect: (f32, f32, f32, f32), // x_min, y_min, x_max, y_max
) -> Option<(f32, f32)> {
    let (x_min, y_min, x_max, y_max) = rect;
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;

    // p array: directional components
    // q array: distance to boundaries
    let p = [-dx, dx, -dy, dy];
    let q = [p1.x - x_min, x_max - p1.x, p1.y - y_min, y_max - p1.y];

    let mut t0 = 0.0;
    let mut t1 = 1.0;

    for i in 0..4 {
        if p[i] == 0.0 {
            // Parallel line outside?
            if q[i] < 0.0 {
                return None;
            }
        } else {
            let t = q[i] / p[i];
            if p[i] < 0.0 {
                if t > t1 {
                    return None;
                }
                if t > t0 {
                    t0 = t;
                }
            } else {
                if t < t0 {
                    return None;
                }
                if t < t1 {
                    t1 = t;
                }
            }
        }
    }

    if t0 <= t1 { Some((t0, t1)) } else { None }
}
