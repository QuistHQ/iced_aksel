use iced_core::{Point, Rectangle};

#[inline]
pub fn rect_intersects_rect(a: &Rectangle, b: &Rectangle) -> bool {
    a.x <= b.x + b.width && a.x + a.width >= b.x && a.y <= b.y + b.height && a.y + a.height >= b.y
}

/// Calculates the shortest distance from a point to a finite line segment.
pub fn distance_point_to_segment(pt: Point, v: Point, w: Point) -> f32 {
    let l2 = (w.x - v.x).powi(2) + (w.y - v.y).powi(2);

    // Edge case: Line segment is actually just a single point (v == w)
    if l2 == 0.0 {
        return ((pt.x - v.x).powi(2) + (pt.y - v.y).powi(2)).sqrt();
    }

    // Find the projection of `pt` onto the mathematical line.
    // `t` is the normalized distance along the line segment.
    let t = ((pt.x - v.x) * (w.x - v.x) + (pt.y - v.y) * (w.y - v.y)) / l2;

    // Clamp `t` between 0 and 1 so we don't project past the ends of the segment!
    let t = t.clamp(0.0, 1.0);

    let projection = Point::new(v.x + t * (w.x - v.x), v.y + t * (w.y - v.y));

    // Return distance from point to the projected point on the segment
    ((pt.x - projection.x).powi(2) + (pt.y - projection.y).powi(2)).sqrt()
}

/// Checks if a line segment intersects a given axis-aligned rectangle.
pub fn line_intersects_rect(p1: Point, p2: Point, rect: &Rectangle) -> bool {
    // 1. Trivial accept: Are either of the line's endpoints inside the rect?
    if rect.contains(p1) || rect.contains(p2) {
        return true;
    }

    // 2. Line segment intersection with the 4 edges of the rectangle.
    // Rect corners:
    let top_left = Point::new(rect.x, rect.y);
    let top_right = Point::new(rect.x + rect.width, rect.y);
    let bottom_left = Point::new(rect.x, rect.y + rect.height);
    let bottom_right = Point::new(rect.x + rect.width, rect.y + rect.height);

    segments_intersect(p1, p2, top_left, top_right) ||       // Top edge
        segments_intersect(p1, p2, top_right, bottom_right) ||   // Right edge
        segments_intersect(p1, p2, bottom_right, bottom_left) || // Bottom edge
        segments_intersect(p1, p2, bottom_left, top_left) // Left edge
}

/// Standard cross-product based line segment intersection check.
fn segments_intersect(a: Point, b: Point, c: Point, d: Point) -> bool {
    let ccw = |p1: Point, p2: Point, p3: Point| {
        (p3.y - p1.y) * (p2.x - p1.x) > (p2.y - p1.y) * (p3.x - p1.x)
    };
    ccw(a, c, d) != ccw(b, c, d) && ccw(a, b, c) != ccw(a, b, d)
}
