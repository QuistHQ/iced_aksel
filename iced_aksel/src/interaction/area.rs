use crate::interaction::{InteractionQuery, math};
use crate::plot;
use crate::radii::{ResolvedRadii, ResolvedRadius};
use aksel::Float;
use iced_core::{Pixels, Point, Radians, Rectangle, Size};
use std::fmt::Debug;

/// A trait for performing precise screen-space collision detection.
///
/// # Examples
///
/// ```rust
/// use iced_aksel::interaction::area::HitTest;
/// use iced_core::Rectangle;
///
/// #[derive(Debug)]
/// struct CustomShape {
///     bounds: Rectangle,
/// }
///
/// impl HitTest for CustomShape {
///     fn bounding_box(&self) -> Rectangle {
///         self.bounds
///     }
///
///     fn intersects(&self, query: &iced_aksel::interaction::InteractionQuery) -> bool {
///         self.bounds.intersects(&query.bounds())
///     }
/// }
/// ```
pub trait HitTest: Debug {
    /// Returns the fast broad-phase bounding box in screen pixels.
    fn bounding_box(&self) -> Rectangle;

    /// Performs the precise narrow-phase check against the user's interaction query.
    fn intersects(&self, query: &InteractionQuery) -> bool;
}

/// A trait for any geometry-like types that can be resolved into screen-space hit areas.
pub trait IntoArea<'a, D: Float, Renderer: crate::Renderer> {
    /// Resolves the type into an area.
    ///
    /// The plot-context provided can be used to resolve screen-space coordinates from plot-coordinates
    fn resolve_area(self, ctx: &plot::Context<'a, D, Renderer>) -> Area;
}

impl<'a, D: Float + 'a, Renderer: crate::Renderer + 'a> IntoArea<'a, D, Renderer> for Area {
    fn resolve_area(self, _: &plot::Context<'a, D, Renderer>) -> Area {
        self
    }
}

/// An area describing an area in screen-space coordinates
#[derive(Debug)]
pub enum Area {
    /// A rectangle
    Rectangle {
        /// The top-left corner of the rectangle
        top_left: Point,
        /// The size (width/height) of the rectangle
        size: Size,
    },
    /// A line segment
    LineSegment {
        /// First endpoint
        p1: Point,
        /// Second endpoint
        p2: Point,
        /// Width of the line stroke
        stroke_width: Pixels,
    },
    /// An ellipse
    Ellipse {
        /// Center point of the ellipse
        center: Point,
        /// Horizontal and vertical radii
        radii: ResolvedRadii,
    },
    /// A triangle
    Triangle {
        /// First vertex
        p1: Point,
        /// Second vertex
        p2: Point,
        /// Third vertex
        p3: Point,
    },
    /// A filled polygon
    Polygon {
        /// Vertices of the polygon
        points: Vec<Point>,
    },
    /// A regular polygon with equal sides and angles
    RegularPolygon {
        /// Center point of the polygon
        center: Point,
        /// Distance from center to vertices
        radius: ResolvedRadius,
        /// Number of vertices
        vertices: u16,
        /// Rotation angle from the default orientation
        rotation: Radians,
    },
    /// A series of connected line segments
    Polyline {
        /// Points defining the polyline
        points: Vec<Point>,
        /// Width of the line stroke
        stroke_width: Pixels,
    },
    /// An arc or ring segment
    Arc {
        /// Center point of the arc
        center: Point,
        /// Outer radius
        radius_outer: ResolvedRadius,
        /// Inner radius (0 for a pie slice)
        radius_inner: ResolvedRadius,
        /// Starting angle in radians
        start_angle: Radians,
        /// Ending angle in radians
        end_angle: Radians,
    },
    /// The escape hatch for custom screen-space hit testing.
    Custom(Box<dyn HitTest>),
}

impl Area {
    /// Returns the axis-aligned bounding box containing this area
    pub fn bounding_box(&self) -> Rectangle {
        match self {
            Self::Rectangle { top_left, size } => {
                Rectangle::new(Point::new(top_left.x, top_left.y), *size)
            }
            Self::LineSegment {
                p1,
                p2,
                stroke_width,
            } => {
                let padding = stroke_width.0 / 2.0;
                let min_x = p1.x.min(p2.x) - padding;
                let max_x = p1.x.max(p2.x) + padding;
                let min_y = p1.y.min(p2.y) - padding;
                let max_y = p1.y.max(p2.y) + padding;

                Rectangle {
                    x: min_x,
                    y: min_y,
                    width: max_x - min_x,
                    height: max_y - min_y,
                }
            }
            Self::Ellipse { center, radii } => Rectangle {
                x: center.x - radii.x,
                y: center.y - radii.y,
                width: radii.x * 2.0,
                height: radii.y * 2.0,
            },
            Self::Triangle { p1, p2, p3 } => {
                let min_x = p1.x.min(p2.x).min(p3.x);
                let max_x = p1.x.max(p2.x).max(p3.x);
                let min_y = p1.y.min(p2.y).min(p3.y);
                let max_y = p1.y.max(p2.y).max(p3.y);
                Rectangle {
                    x: min_x,
                    y: min_y,
                    width: max_x - min_x,
                    height: max_y - min_y,
                }
            }
            Self::Polygon { points } => {
                if points.is_empty() {
                    return Rectangle::default();
                }
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                for p in points {
                    min_x = min_x.min(p.x);
                    min_y = min_y.min(p.y);
                    max_x = max_x.max(p.x);
                    max_y = max_y.max(p.y);
                }
                Rectangle {
                    x: min_x,
                    y: min_y,
                    width: max_x - min_x,
                    height: max_y - min_y,
                }
            }
            Self::RegularPolygon { center, radius, .. } => Rectangle {
                x: center.x - radius.0,
                y: center.y - radius.0,
                width: radius.0 * 2.0,
                height: radius.0 * 2.0,
            },
            Self::Polyline {
                points,
                stroke_width,
            } => {
                if points.is_empty() {
                    return Rectangle::default();
                }
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                for p in points {
                    min_x = min_x.min(p.x);
                    min_y = min_y.min(p.y);
                    max_x = max_x.max(p.x);
                    max_y = max_y.max(p.y);
                }
                let padding = stroke_width.0 / 2.0;
                Rectangle {
                    x: min_x - padding,
                    y: min_y - padding,
                    width: (max_x - min_x) + stroke_width.0,
                    height: (max_y - min_y) + stroke_width.0,
                }
            }
            Self::Arc {
                center,
                radius_outer,
                ..
            } => Rectangle {
                // A conservative bounding box covering the full circle
                x: center.x - radius_outer.0,
                y: center.y - radius_outer.0,
                width: radius_outer.0 * 2.0,
                height: radius_outer.0 * 2.0,
            },
            Self::Custom(custom_test) => custom_test.bounding_box(),
        }
    }

    /// Tests whether this area intersects with the given interaction query
    pub fn intersects(&self, query: &InteractionQuery) -> bool {
        match (self, query) {
            // ======================
            // Rectangle
            // ======================
            (Self::Rectangle { top_left, size }, query) => {
                let rect = Rectangle::new(Point::new(top_left.x, top_left.y), *size);
                math::rect_intersects_rect(&rect, &query.bounds())
            }
            (
                Self::LineSegment {
                    p1,
                    p2,
                    stroke_width,
                },
                query,
            ) => match query {
                InteractionQuery::Point {
                    position,
                    tolerance,
                } => {
                    let distance = math::distance_point_to_segment(*position, *p1, *p2);
                    distance <= (stroke_width.0 / 2.0) + tolerance.0
                }
                InteractionQuery::Bounds(bounds) => math::line_intersects_rect(*p1, *p2, bounds),
            },
            // ======================
            // Ellipse
            // ======================
            (Self::Ellipse { center, radii }, query) => match query {
                InteractionQuery::Point {
                    position,
                    tolerance,
                } => math::point_in_ellipse(
                    *position,
                    Point::new(center.x, center.y),
                    radii.x,
                    radii.y,
                    tolerance.0,
                ),
                InteractionQuery::Bounds(bounds) => {
                    math::rect_intersects_ellipse(bounds, *center, radii.x, radii.y)
                }
            },
            // ======================
            // Triangle
            // ======================
            (Self::Triangle { p1, p2, p3 }, query) => match query {
                InteractionQuery::Point {
                    position,
                    tolerance,
                } => math::point_in_triangle(*position, *p1, *p2, *p3, tolerance.0),
                InteractionQuery::Bounds(bounds) => {
                    math::rect_intersects_triangle(bounds, *p1, *p2, *p3)
                }
            },
            // ======================
            // Polygon
            // ======================
            (Self::Polygon { points }, query) => match query {
                InteractionQuery::Point { position, .. } => {
                    // Polygons don't strictly need tolerance for filled areas, just Ray-Casting
                    math::point_in_polygon(*position, points)
                }
                InteractionQuery::Bounds(bounds) => math::rect_intersects_polygon(bounds, points),
            },
            // ======================
            // Polyline
            // ======================
            (
                Self::Polyline {
                    points,
                    stroke_width,
                },
                query,
            ) => match query {
                InteractionQuery::Point {
                    position,
                    tolerance,
                } => math::point_in_polyline(*position, points, stroke_width.0, tolerance.0),
                InteractionQuery::Bounds(bounds) => math::rect_intersects_polyline(bounds, points),
            },
            // ======================
            // Regular Polygon
            // ======================
            (
                Self::RegularPolygon {
                    center,
                    radius,
                    vertices,
                    rotation,
                },
                query,
            ) => {
                let mut pts = Vec::with_capacity(*vertices as usize);
                let angle_step = std::f32::consts::TAU / (*vertices as f32);
                for i in 0..*vertices {
                    let angle = (i as f32).mul_add(angle_step, rotation.0);
                    pts.push(Point::new(
                        radius.0.mul_add(angle.cos(), center.x),
                        radius.0.mul_add(angle.sin(), center.y),
                    ));
                }
                match query {
                    InteractionQuery::Point { position, .. } => {
                        math::point_in_polygon(*position, &pts)
                    }
                    InteractionQuery::Bounds(bounds) => math::rect_intersects_polygon(bounds, &pts),
                }
            }
            // ======================
            // Arc
            // ======================
            (
                Self::Arc {
                    center,
                    radius_outer,
                    radius_inner,
                    start_angle,
                    end_angle,
                },
                query,
            ) => match query {
                InteractionQuery::Point { position, .. } => math::point_in_arc(
                    *position,
                    *center,
                    radius_inner.0,
                    radius_outer.0,
                    start_angle.0,
                    end_angle.0,
                ),
                InteractionQuery::Bounds(bounds) => {
                    math::rect_intersects_arc(bounds, *center, radius_outer.0)
                }
            },
            (Self::Custom(custom_test), q) => custom_test.intersects(q),
        }
    }
}
