//! Shared math utilities for both mesh and path rendering pipelines.

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
    let len_sq = v.x.mul_add(v.x, v.y * v.y);
    if len_sq < 1e-8 {
        Vector::new(0.0, 0.0)
    } else {
        let len = len_sq.sqrt();
        Vector::new(v.x / len, v.y / len)
    }
}

/// Calculates the intersection of an **Infinite Line** with a bounding box.
///
/// Returns `Some((entry_point, exit_point))` if the line is visible.
pub fn clip_infinite_line(start: Point, end: Point, bounds: Bounds) -> Option<(Point, Point)> {
    let delta_x = end.x - start.x;
    let delta_y = end.y - start.y;

    // Check for vertical lines
    if delta_x.abs() < 1e-6 {
        if start.x < bounds.min_x || start.x > bounds.max_x {
            return None;
        }
        return Some((
            Point::new(start.x, bounds.min_y),
            Point::new(start.x, bounds.max_y),
        ));
    }
    // Check for horizontal lines
    if delta_y.abs() < 1e-6 {
        if start.y < bounds.min_y || start.y > bounds.max_y {
            return None;
        }
        return Some((
            Point::new(bounds.min_x, start.y),
            Point::new(bounds.max_x, start.y),
        ));
    }

    // Calculate intersection parameter 't' for all 4 boundaries
    let t_at_min_x = (bounds.min_x - start.x) / delta_x;
    let t_at_max_x = (bounds.max_x - start.x) / delta_x;
    let t_at_min_y = (bounds.min_y - start.y) / delta_y;
    let t_at_max_y = (bounds.max_y - start.y) / delta_y;

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
            delta_x.mul_add(t_entry, start.x),
            delta_y.mul_add(t_entry, start.y),
        ),
        Point::new(
            delta_x.mul_add(t_exit, start.x),
            delta_y.mul_add(t_exit, start.y),
        ),
    ))
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
