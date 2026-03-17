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

// --- ELLIPSE MATH ---
#[inline]
pub fn point_in_ellipse(pt: Point, center: Point, rx: f32, ry: f32, tolerance: f32) -> bool {
    let rx = rx + tolerance;
    let ry = ry + tolerance;
    if rx <= 0.0 || ry <= 0.0 {
        return false;
    }

    let dx = pt.x - center.x;
    let dy = pt.y - center.y;
    (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry) <= 1.0
}

#[inline]
pub fn rect_intersects_ellipse(rect: &Rectangle, center: Point, rx: f32, ry: f32) -> bool {
    if rx <= 0.0 || ry <= 0.0 {
        return false;
    }
    let closest_x = center.x.clamp(rect.x, rect.x + rect.width);
    let closest_y = center.y.clamp(rect.y, rect.y + rect.height);
    let dx = closest_x - center.x;
    let dy = closest_y - center.y;
    (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry) <= 1.0
}

// --- TRIANGLE MATH ---
#[inline]
pub fn point_in_triangle(pt: Point, p1: Point, p2: Point, p3: Point, tolerance: f32) -> bool {
    // Fast half-plane check. We expand the check slightly using the tolerance
    // Note: For a true robust hover tolerance on the edges, we check distance to segments
    if distance_point_to_segment(pt, p1, p2) <= tolerance
        || distance_point_to_segment(pt, p2, p3) <= tolerance
        || distance_point_to_segment(pt, p3, p1) <= tolerance
    {
        return true;
    }

    // Standard barycentric / cross-product check for strictly inside
    let d1 = sign(pt, p1, p2);
    let d2 = sign(pt, p2, p3);
    let d3 = sign(pt, p3, p1);

    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);

    !(has_neg && has_pos)
}

#[inline]
fn sign(p1: Point, p2: Point, p3: Point) -> f32 {
    (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}

#[inline]
pub fn rect_intersects_triangle(rect: &Rectangle, p1: Point, p2: Point, p3: Point) -> bool {
    // Trivial accept: Are any triangle points inside the rect?
    if rect.contains(p1) || rect.contains(p2) || rect.contains(p3) {
        return true;
    }
    // Are the rect's edges intersecting the triangle's edges?
    line_intersects_rect(p1, p2, rect)
        || line_intersects_rect(p2, p3, rect)
        || line_intersects_rect(p3, p1, rect)
}

// --- POLYGON MATH ---
/// Ray-Casting algorithm (Even-Odd rule) to check if a point is inside a polygon
pub fn point_in_polygon(pt: Point, polygon: &[Point]) -> bool {
    let mut inside = false;
    let mut j = polygon.len().saturating_sub(1);

    for i in 0..polygon.len() {
        let pi = polygon[i];
        let pj = polygon[j];

        if (pi.y > pt.y) != (pj.y > pt.y) {
            let intersect_x = (pj.x - pi.x) * (pt.y - pi.y) / (pj.y - pi.y) + pi.x;
            if pt.x < intersect_x {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

pub fn rect_intersects_polygon(rect: &Rectangle, polygon: &[Point]) -> bool {
    if polygon.is_empty() {
        return false;
    }
    // 1. Are any polygon vertices inside the rect?
    if polygon.iter().any(|&p| rect.contains(p)) {
        return true;
    }

    // 2. Do any polygon edges intersect the rect?
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        if line_intersects_rect(polygon[j], polygon[i], rect) {
            return true;
        }
        j = i;
    }

    // 3. Is the rect entirely inside the polygon? (Just check one corner)
    point_in_polygon(Point::new(rect.x, rect.y), polygon)
}

// --- POLYLINE MATH ---
pub fn point_in_polyline(pt: Point, polyline: &[Point], stroke_width: f32, tolerance: f32) -> bool {
    let max_dist = (stroke_width / 2.0) + tolerance;

    for i in 0..polyline.len().saturating_sub(1) {
        if distance_point_to_segment(pt, polyline[i], polyline[i + 1]) <= max_dist {
            return true;
        }
    }
    false
}

pub fn rect_intersects_polyline(rect: &Rectangle, polyline: &[Point]) -> bool {
    if polyline.is_empty() {
        return false;
    }
    // 1. Are any vertices inside the rect?
    if polyline.iter().any(|&p| rect.contains(p)) {
        return true;
    }

    // 2. Do any edges intersect the rect?
    for i in 0..polyline.len().saturating_sub(1) {
        if line_intersects_rect(polyline[i], polyline[i + 1], rect) {
            return true;
        }
    }
    false
}

// --- ARC / SECTOR MATH ---
pub fn point_in_arc(
    pt: Point,
    center: Point,
    inner_r: f32,
    outer_r: f32,
    start_angle: f32,
    end_angle: f32,
) -> bool {
    let dx = pt.x - center.x;
    let dy = pt.y - center.y;
    let dist_sq = dx * dx + dy * dy;

    // 1. Check distance (Is it within the donut/circle?)
    if dist_sq < (inner_r * inner_r) || dist_sq > (outer_r * outer_r) {
        return false;
    }

    // 2. Check angle (Is it within the pie slice?)
    let mut angle = dy.atan2(dx);
    if angle < 0.0 {
        angle += std::f32::consts::TAU;
    }

    let mut start = start_angle % std::f32::consts::TAU;
    if start < 0.0 {
        start += std::f32::consts::TAU;
    }

    let mut end = end_angle % std::f32::consts::TAU;
    if end < 0.0 {
        end += std::f32::consts::TAU;
    }

    if start < end {
        angle >= start && angle <= end
    } else {
        // The arc crosses the 0/360 degree boundary
        angle >= start || angle <= end
    }
}

pub fn rect_intersects_arc(rect: &Rectangle, center: Point, outer_r: f32) -> bool {
    // Precise rect-to-arc math is extremely heavy.
    // For marquee selection (broad interactions), we approximate by checking
    // if the rect intersects the arc's outer bounding circle.
    let closest_x = center.x.clamp(rect.x, rect.x + rect.width);
    let closest_y = center.y.clamp(rect.y, rect.y + rect.height);

    let dx = closest_x - center.x;
    let dy = closest_y - center.y;

    (dx * dx + dy * dy) <= (outer_r * outer_r)
}

#[cfg(test)]
mod tests {
    use crate::interaction::math::{
        point_in_ellipse, point_in_polygon, point_in_polyline, point_in_triangle,
    };
    use iced_core::{Point, Rectangle};

    #[test]
    fn test_point_in_triangle() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(10.0, 0.0);
        let p3 = Point::new(5.0, 10.0);

        // Clearly inside
        assert!(point_in_triangle(Point::new(5.0, 5.0), p1, p2, p3, 0.0));
        // Clearly outside
        assert!(!point_in_triangle(Point::new(10.0, 10.0), p1, p2, p3, 0.0));
        // Outside, but within hover tolerance of the edge
        assert!(point_in_triangle(Point::new(5.0, 11.0), p1, p2, p3, 2.0));
    }

    #[test]
    fn test_point_in_polygon_ray_casting() {
        // A concave "Pac-Man" shape pointing right
        // Bounding box is 0 to 10 on both axes. The "mouth" cuts inward to (5, 5).
        let polygon = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(5.0, 5.0), // The concave dent
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ];

        // Inside the solid body
        assert!(point_in_polygon(Point::new(2.0, 5.0), &polygon));
        // Inside the top jaw
        assert!(point_in_polygon(Point::new(6.0, 8.0), &polygon));
        // Outside the shape, inside the concave "mouth" (This proves Ray-Casting works!)
        assert!(!point_in_polygon(Point::new(8.0, 5.0), &polygon));
        // Completely outside
        assert!(!point_in_polygon(Point::new(15.0, 5.0), &polygon));
    }

    #[test]
    fn test_point_in_polyline() {
        let line = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
        ];

        let stroke_width = 2.0;
        let tolerance = 1.0; // Total allowed distance from center = 2.0

        // Exactly on the line
        assert!(point_in_polyline(
            Point::new(5.0, 0.0),
            &line,
            stroke_width,
            tolerance
        ));
        // Hovering just near the line segment
        assert!(point_in_polyline(
            Point::new(5.0, 1.5),
            &line,
            stroke_width,
            tolerance
        ));
        // Hovering near the "elbow" joint
        assert!(point_in_polyline(
            Point::new(11.0, 1.0),
            &line,
            stroke_width,
            tolerance
        ));
        // Too far away
        assert!(!point_in_polyline(
            Point::new(5.0, 5.0),
            &line,
            stroke_width,
            tolerance
        ));
    }

    #[test]
    fn test_point_in_ellipse() {
        let center = Point::new(5.0, 5.0);
        let rx = 4.0;
        let ry = 2.0;

        // Dead center
        assert!(point_in_ellipse(Point::new(5.0, 5.0), center, rx, ry, 0.0));
        // Inside
        assert!(point_in_ellipse(Point::new(7.0, 6.0), center, rx, ry, 0.0));
        // Outside the mathematical ellipse, but inside the bounding box
        assert!(!point_in_ellipse(Point::new(8.5, 6.5), center, rx, ry, 0.0));
        // Outside, but caught by the hover tolerance
        assert!(point_in_ellipse(Point::new(8.5, 6.5), center, rx, ry, 1.0));
    }
}
