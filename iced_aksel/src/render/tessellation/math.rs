use iced_core::{Point, Vector};

/// Liang-Barsky line clipping algorithm.
pub fn clip_line_liang_barsky(
    p1: Point,
    p2: Point,
    clip_rect: (f32, f32, f32, f32),
) -> Option<(f32, f32)> {
    let (xmin, ymin, xmax, ymax) = clip_rect;
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let mut t0 = 0.0;
    let mut t1 = 1.0;
    let p = [-dx, dx, -dy, dy];
    let q = [p1.x - xmin, xmax - p1.x, p1.y - ymin, ymax - p1.y];

    for i in 0..4 {
        if p[i].abs() < 1e-6 && q[i] < 0.0 {
                return None;
        }
            let t = q[i] / p[i];
            if p[i] < 0.0 {
                if t > t1 {
                    return None;
                }
                if t > t0 {
                    t0 = t;
                }
                
                continue
            } 
            if t < t0 {
                return None;
            }
            if t < t1 {
                t1 = t;
            }
    }

    if t0 <= t1 { Some((t0, t1)) } else { None }
}

/// Normalizes a vector. Returns (0,0) if length is close to zero.
pub fn normalize(v: Vector) -> Vector {
    let len = (v.x * v.x + v.y * v.y).sqrt();
    if len < 1e-4 {
        Vector::new(0.0, 0.0)
    } else {
        Vector::new(v.x / len, v.y / len)
    }
}

/// Computes the miter join vertex for a corner.
pub fn compute_inset_vertex(prev: Point, current: Point, next: Point, distance: f32) -> Point {
    let v1 = normalize(current - prev);
    let v2 = normalize(next - current);
    let tangent = normalize(v1 + v2);
    let miter = Vector::new(-tangent.y, tangent.x);
    let n1 = Vector::new(-v1.y, v1.x);
    let dot = miter.x * n1.x + miter.y * n1.y;
    let miter_len = distance / dot;
    let limited_len = miter_len.min(distance * 5.0);
    current + miter * limited_len
}

/// Generates a ring of points for a regular polygon.
pub fn generate_ring(center: Point, radius: f32, vertices: u16, rotation: f32) -> Vec<Point> {
    let mut points = Vec::with_capacity(vertices as usize);
    let angle_step = 360.0 / vertices as f32;
    let start_angle = rotation - 90.0;
    for i in 0..vertices {
        let theta = (start_angle + i as f32 * angle_step).to_radians();
        let (sin, cos) = theta.sin_cos();
        points.push(Point::new(center.x + radius * cos, center.y + radius * sin));
    }
    points
}

/// Checks if a polygon is convex.
pub fn is_convex(points: &[Point]) -> bool {
    if points.len() < 4 {
        return true;
    }
    if points.len() > 20 {
        return false;
    } // Heuristic limit
    let mut sign = 0.0;
    let n = points.len();
    for i in 0..n {
        let p1 = points[i];
        let p2 = points[(i + 1) % n];
        let p3 = points[(i + 2) % n];
        let v1 = p2 - p1;
        let v2 = p3 - p2;
        let cross = v1.x * v2.y - v1.y * v2.x;
        if cross.abs() < 1e-5 {
            continue;
        }
        if sign == 0.0 {
            sign = cross;
        } else if cross * sign < 0.0 {
            return false;
        }
    }
    true
}

/// Computes an inset polygon for a convex shape.
pub fn compute_inset_polygon(points: &[Point], dist: f32) -> Vec<Point> {
    let n = points.len();
    let mut new_points = Vec::with_capacity(n);
    for i in 0..n {
        let prev = points[(i + n - 1) % n];
        let curr = points[i];
        let next = points[(i + 1) % n];
        new_points.push(compute_inset_vertex(prev, curr, next, dist));
    }
    new_points
}
