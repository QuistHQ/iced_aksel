use crate::interaction::{InteractionQuery, math};
use crate::plot;
use crate::radii::{ResolvedRadii, ResolvedRadius};
use aksel::{Float, Transform};
use iced_core::{Font, Pixels, Point, Radians, Rectangle, Size};
use std::fmt::Debug;
use std::ops::Deref;

/// A trait for performing precise screen-space collision detection.
pub trait HitTest: Debug {
    /// Returns the fast broad-phase bounding box in screen pixels.
    fn bounding_box(&self) -> Rectangle;

    /// Performs the precise narrow-phase check against the user's interaction query.
    fn intersects(&self, query: &InteractionQuery) -> bool;
}

/// Internal context for Areas.
pub struct AreaContext<'a, D: Float, Renderer: crate::Renderer = iced_renderer::Renderer> {
    pub(crate) transform: &'a Transform<'a, D, f32, f32>,
    pub(crate) renderer: &'a Renderer,
}

impl<'a, 'b, D, Renderer> From<&'b plot::Context<'a, D, Renderer>> for AreaContext<'b, D, Renderer>
where
    D: Float,
    Renderer: crate::Renderer,
{
    fn from(value: &'b plot::Context<'a, D, Renderer>) -> Self {
        Self {
            transform: value.transform,
            renderer: value.renderer,
        }
    }
}

impl<'a, D: Float, Renderer: crate::Renderer> Deref for AreaContext<'a, D, Renderer> {
    type Target = Transform<'a, D, f32, f32>;

    fn deref(&self) -> &Self::Target {
        self.transform
    }
}

impl<'a, D: Float, Renderer: crate::Renderer> AreaContext<'a, D, Renderer> {
    /// Returns the default font of the underlying renderer
    #[inline(always)]
    pub fn default_font(&self) -> Font {
        self.renderer.default_font()
    }

    pub fn measure_text(&self, text: iced_core::text::Text<&str>) -> iced_core::Size {
        use iced_core::text::Paragraph as _;
        <Renderer as iced_core::text::Renderer>::Paragraph::with_text(text).min_bounds()
    }

    /// Returns the screen bounds bounds of the plot
    pub const fn clip_bounds(&self) -> Rectangle {
        let bounds = self.transform.screen_bounds();
        Rectangle {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bounds.height,
        }
    }
}

/// A trait for any geometries that can be resolved into screen-space hit areas.
///
/// The trait allows for None to be returned to allow not rendering when some requirements aren't
/// met (e.g. Area is too small).
pub trait IntoArea<'a, D: Float, Renderer: crate::Renderer> {
    fn resolve_area(self, ctx: &AreaContext<'a, D, Renderer>) -> Option<Area>;
}

impl<'a, D: Float, Renderer: crate::Renderer> IntoArea<'a, D, Renderer> for Area {
    fn resolve_area(self, _: &AreaContext<'a, D, Renderer>) -> Option<Area> {
        Some(self)
    }
}

#[derive(Debug)]
pub enum Area {
    Rectangle {
        top_left: Point,
        size: Size,
    },
    LineSegment {
        p1: Point,
        p2: Point,
        stroke_width: Pixels,
    },
    Ellipse {
        center: Point,
        radii: ResolvedRadii,
    },
    Triangle {
        p1: Point,
        p2: Point,
        p3: Point,
    },
    Polygon {
        points: Vec<Point>,
    },
    RegularPolygon {
        center: Point,
        radius: ResolvedRadius,
        vertices: u16,
        rotation: Radians,
    },
    Polyline {
        points: Vec<Point>,
        stroke_width: Pixels,
    },
    Arc {
        center: Point,
        radius_outer: ResolvedRadius,
        radius_inner: ResolvedRadius,
        start_angle: Radians,
        end_angle: Radians,
    },
    /// The escape hatch for custom screen-space hit testing.
    Custom(Box<dyn HitTest>),
}

impl Area {
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
                    let angle = rotation.0 + (i as f32) * angle_step;
                    pts.push(Point::new(
                        center.x + radius.0 * angle.cos(),
                        center.y + radius.0 * angle.sin(),
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
