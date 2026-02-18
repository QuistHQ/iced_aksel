//! Geometric utility functions for mesh tessellation.
//!
//! Mesh-specific helpers for polygon inset, convexity checks, ring generation, and segment clipping.
//! Shared utilities (normalize, Bounds, clip_infinite_line, catmull_rom_to_bezier) are re-exported
//! from the crate-level shared math module.

pub use crate::render::cache::math::{
    Bounds, catmull_rom_to_bezier, clip_infinite_line, normalize,
};
use iced_core::{Point, Vector};

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
        let cross_z = dx1.mul_add(dy2, -(dy1 * dx2));

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
        let theta = (i as f32).mul_add(step, start_angle);
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
    let corner_sin = v1.x.mul_add(v2.y, -(v1.y * v2.x)).abs();

    // SAFETY CHECK 1: Parallel lines (Angle ~ 0 or 180)
    // Just shift perpendicularly, don't try to compute a corner.
    if corner_sin < 0.001 {
        return curr + n1 * distance;
    }

    // 4. Calculate the "Miter" vector (the direction of the corner bisection)
    let combined = v1 - v2;
    let len_sq = combined.x.mul_add(combined.x, combined.y * combined.y);
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
    let cross = v1.x.mul_add(v2.y, -(v1.y * v2.x));
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
        Point::new(
            delta_x.mul_add(t_enter, start.x),
            delta_y.mul_add(t_enter, start.y),
        ),
        Point::new(
            delta_x.mul_add(t_exit, start.x),
            delta_y.mul_add(t_exit, start.y),
        ),
    ))
}
