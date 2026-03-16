use crate::Measure;
use crate::interaction::{InteractionQuery, math};
use aksel::{Float, PlotPoint, Transform};
use iced_core::{Point, Rectangle};
use std::fmt::Debug;

/// A trait for performing precise screen-space collision detection.
pub trait HitTest: Debug {
    /// Returns the fast broad-phase bounding box in screen pixels.
    fn bounding_box(&self) -> Rectangle;

    /// Performs the precise narrow-phase check against the user's interaction query.
    fn intersects(&self, query: &InteractionQuery) -> bool;
}

/// A trait for custom geometries that can be resolved into screen-space hit areas.
pub trait ResolvableArea<D: Float>: Debug {
    /// Converts the data-space geometry into a screen-space `HitTest`.
    fn resolve_area(&self, transform: &Transform<D, f32, f32>) -> Box<dyn HitTest>;
}

/// The exact geometric intent for the hit-test.
#[derive(Debug)]
pub enum Area<D> {
    Rect {
        x: D,
        y: D,
        width: Measure<D>,
        height: Measure<D>,
    },
    LineSegment {
        p1: PlotPoint<D>,
        p2: PlotPoint<D>,
        width: f32,
    },
    Ellipse {
        center: PlotPoint<D>,
        radius_x: Measure<D>,
        radius_y: Measure<D>,
    },
    Triangle {
        p1: PlotPoint<D>,
        p2: PlotPoint<D>,
        p3: PlotPoint<D>,
    },
    CenteredTriangle {
        center: PlotPoint<D>,
        width: Measure<D>,
        height: Measure<D>,
    },
    Polygon {
        points: Vec<PlotPoint<D>>,
    },
    RegularPolygon {
        center: PlotPoint<D>,
        radius: Measure<D>,
        vertices: u16,
        rotation_rads: f32,
    },
    Polyline {
        points: Vec<PlotPoint<D>>,
        width: Measure<D>,
    },
    Bezier {
        start: PlotPoint<D>,
        control_1: PlotPoint<D>,
        control_2: Option<PlotPoint<D>>,
        end: PlotPoint<D>,
        width: Measure<D>,
    },
    Spline {
        points: Vec<PlotPoint<D>>,
        width: Measure<D>,
        tension: f32,
    },
    Arc {
        center: PlotPoint<D>,
        radius_outer: Measure<D>,
        radius_inner: Measure<D>,
        start_angle_rads: f32,
        end_angle_rads: f32,
    },
    Label {
        content: String,
        position: PlotPoint<D>,
        size: Measure<D>,
        font: Option<iced_core::Font>,
        horizontal_alignment: iced_core::alignment::Horizontal,
        vertical_alignment: iced_core::alignment::Vertical,
        rotation_rads: f32,
        line_height: f32,
        bounds: crate::shape::Bounds<D>,
        wrapping: iced_core::text::Wrapping,
    },
    /// The escape hatch for custom data-space interactions.
    Custom(Box<dyn ResolvableArea<D>>),
}

impl<D: Float> Area<D> {
    pub(crate) fn resolve<R: iced_core::text::Renderer<Font = iced_core::Font>>(
        self,
        transform: &Transform<D, f32, f32>,
        renderer: &R,
    ) -> ResolvedArea {
        match self {
            Self::Rect {
                x,
                y,
                width,
                height,
            } => {
                let width_data = if let Measure::Plot(w) = width {
                    w
                } else {
                    D::zero()
                };
                let height_data = if let Measure::Plot(h) = height {
                    h
                } else {
                    D::zero()
                };

                let p1 = transform.chart_to_screen(&PlotPoint::new(x, y));
                let p2 =
                    transform.chart_to_screen(&PlotPoint::new(x + width_data, y + height_data));

                ResolvedArea::Rect(Rectangle {
                    x: p1.x.min(p2.x),
                    y: p1.y.min(p2.y),
                    width: width.resolve_x(transform),
                    height: height.resolve_y(transform),
                })
            }
            Self::LineSegment { p1, p2, width } => {
                let sp1 = transform.chart_to_screen(&p1);
                let sp2 = transform.chart_to_screen(&p2);
                ResolvedArea::LineSegment {
                    p1: Point::new(sp1.x, sp1.y),
                    p2: Point::new(sp2.x, sp2.y),
                    stroke_width_px: width,
                }
            }
            Self::Ellipse {
                center,
                radius_x,
                radius_y,
            } => {
                let sc = transform.chart_to_screen(&center);
                ResolvedArea::Ellipse {
                    center: Point::new(sc.x, sc.y),
                    rx: radius_x.resolve_x(transform),
                    ry: radius_y.resolve_y(transform),
                }
            }
            Self::Triangle { p1, p2, p3 } => {
                let sp1 = transform.chart_to_screen(&p1);
                let sp2 = transform.chart_to_screen(&p2);
                let sp3 = transform.chart_to_screen(&p3);
                ResolvedArea::Triangle {
                    p1: Point::new(sp1.x, sp1.y),
                    p2: Point::new(sp2.x, sp2.y),
                    p3: Point::new(sp3.x, sp3.y),
                }
            }
            Self::CenteredTriangle {
                center,
                width,
                height,
            } => {
                let sc = transform.chart_to_screen(&center);
                let center_x = sc.x;
                let center_y = sc.y;

                let w = width.resolve_x(transform);
                let h = height.resolve_y(transform);

                let half_w = w / 2.0;
                let half_h = h / 2.0;

                ResolvedArea::Triangle {
                    p1: Point::new(center_x, center_y - half_h),
                    p2: Point::new(center_x + half_w, center_y + half_h),
                    p3: Point::new(center_x - half_w, center_y + half_h),
                }
            }
            Self::Polygon { points } => ResolvedArea::Polygon {
                points: points
                    .into_iter()
                    .map(|p| {
                        let sp = transform.chart_to_screen(&p);
                        Point::new(sp.x, sp.y)
                    })
                    .collect(),
            },
            Self::RegularPolygon {
                center,
                radius,
                vertices,
                rotation_rads,
            } => {
                let sc = transform.chart_to_screen(&center);
                ResolvedArea::RegularPolygon {
                    center: Point::new(sc.x, sc.y),
                    radius_px: radius.resolve_x(transform).min(radius.resolve_y(transform)),
                    vertices,
                    rotation_rads,
                }
            }
            Self::Polyline { points, width } => ResolvedArea::Polyline {
                points: points
                    .into_iter()
                    .map(|p| {
                        let sp = transform.chart_to_screen(&p);
                        Point::new(sp.x, sp.y)
                    })
                    .collect(),
                stroke_width_px: width.resolve_x(transform).min(width.resolve_y(transform)),
            },
            Self::Bezier {
                start,
                control_1,
                control_2,
                end,
                width,
            } => {
                let p0 = transform.chart_to_screen(&start);
                let p1 = transform.chart_to_screen(&control_1);
                let p3 = transform.chart_to_screen(&end);

                let stroke_width_px = width.resolve_x(transform).min(width.resolve_y(transform));
                let segments = 30; // 30 segments is plenty for accurate UI hit-testing
                let mut points = Vec::with_capacity(segments + 1);

                if let Some(c2) = control_2 {
                    // Cubic Bezier Evaluation
                    let p2 = transform.chart_to_screen(&c2);
                    for i in 0..=segments {
                        let t = i as f32 / segments as f32;
                        let t_inv = 1.0 - t;

                        let x = t_inv.powi(3) * p0.x
                            + 3.0 * t_inv.powi(2) * t * p1.x
                            + 3.0 * t_inv * t.powi(2) * p2.x
                            + t.powi(3) * p3.x;

                        let y = t_inv.powi(3) * p0.y
                            + 3.0 * t_inv.powi(2) * t * p1.y
                            + 3.0 * t_inv * t.powi(2) * p2.y
                            + t.powi(3) * p3.y;

                        points.push(Point::new(x, y));
                    }
                } else {
                    // Quadratic Bezier Evaluation
                    for i in 0..=segments {
                        let t = i as f32 / segments as f32;
                        let t_inv = 1.0 - t;

                        let x = t_inv.powi(2) * p0.x + 2.0 * t_inv * t * p1.x + t.powi(2) * p3.x;
                        let y = t_inv.powi(2) * p0.y + 2.0 * t_inv * t * p1.y + t.powi(2) * p3.y;

                        points.push(Point::new(x, y));
                    }
                }

                // MAGIC: We return it as a Polyline, so no new hit-test math is needed!
                ResolvedArea::Polyline {
                    points,
                    stroke_width_px,
                }
            }

            Self::Spline {
                points,
                width,
                tension,
            } => {
                let stroke_width_px = width.resolve_x(transform).min(width.resolve_y(transform));

                if points.len() < 2 {
                    return ResolvedArea::Polyline {
                        points: vec![],
                        stroke_width_px,
                    };
                }

                // Map to screen points
                let sp: Vec<Point> = points
                    .iter()
                    .map(|p| {
                        let sc = transform.chart_to_screen(p);
                        Point::new(sc.x, sc.y)
                    })
                    .collect();

                let mut flattened = Vec::new();
                let segments_per_curve = 15;

                // Catmull-Rom Spline flattening
                for i in 0..sp.len().saturating_sub(1) {
                    let p0 = if i == 0 { sp[0] } else { sp[i - 1] };
                    let p1 = sp[i];
                    let p2 = sp[i + 1];
                    let p3 = if i + 2 < sp.len() { sp[i + 2] } else { p2 };

                    for step in 0..=segments_per_curve {
                        let t = step as f32 / segments_per_curve as f32;
                        let t2 = t * t;
                        let t3 = t2 * t;

                        // Catmull-Rom math incorporating the user's tension parameter
                        let alpha = 1.0 - tension;

                        let x = 0.5
                            * ((2.0 * p1.x)
                                + (-p0.x + p2.x) * t * alpha
                                + (2.0 * p0.x - 5.0 * p1.x + 4.0 * p2.x - p3.x) * t2 * alpha
                                + (-p0.x + 3.0 * p1.x - 3.0 * p2.x + p3.x) * t3 * alpha);

                        let y = 0.5
                            * ((2.0 * p1.y)
                                + (-p0.y + p2.y) * t * alpha
                                + (2.0 * p0.y - 5.0 * p1.y + 4.0 * p2.y - p3.y) * t2 * alpha
                                + (-p0.y + 3.0 * p1.y - 3.0 * p2.y + p3.y) * t3 * alpha);

                        flattened.push(Point::new(x, y));
                    }
                }

                ResolvedArea::Polyline {
                    points: flattened,
                    stroke_width_px,
                }
            }
            Self::Arc {
                center,
                radius_outer,
                radius_inner,
                start_angle_rads,
                end_angle_rads,
            } => {
                let sc = transform.chart_to_screen(&center);
                ResolvedArea::Arc {
                    center: Point::new(sc.x, sc.y),
                    radius_outer: radius_outer
                        .resolve_x(transform)
                        .min(radius_outer.resolve_y(transform)),
                    radius_inner: radius_inner
                        .resolve_x(transform)
                        .min(radius_inner.resolve_y(transform)),
                    start_angle: start_angle_rads,
                    end_angle: end_angle_rads,
                }
            }
            Self::Label {
                content,
                position,
                size,
                font,
                horizontal_alignment,
                vertical_alignment,
                rotation_rads,
                line_height,
                bounds,
                wrapping,
            } => {
                // Bring the 0.14 Paragraph trait into scope
                use iced_core::text::Paragraph as _;

                let sc = transform.chart_to_screen(&position);
                let screen_pos = Point::new(sc.x as f32, sc.y as f32); // Force f32

                let font_size_px = size.resolve_y(transform);
                let bounds_size = bounds.resolve(transform, &position);
                let font = font.unwrap_or_else(|| renderer.default_font());

                // 1. Measure the text using the Iced 0.14 Paragraph API
                let paragraph =
                    <R as iced_core::text::Renderer>::Paragraph::with_text(iced_core::text::Text {
                        content: content.as_str(),
                        bounds: bounds_size,
                        size: iced_core::Pixels(font_size_px),
                        line_height: iced_core::text::LineHeight::Relative(line_height),
                        font,
                        align_x: horizontal_alignment.into(),
                        align_y: vertical_alignment.into(),
                        shaping: iced_core::text::Shaping::Basic,
                        wrapping,
                    });

                let text_size = paragraph.min_bounds();

                let text_size = paragraph.min_bounds();

                // 2. Explicitly type the offsets as f32
                let dx: f32 = match horizontal_alignment {
                    iced_core::alignment::Horizontal::Left => 0.0,
                    iced_core::alignment::Horizontal::Center => -text_size.width / 2.0,
                    iced_core::alignment::Horizontal::Right => -text_size.width,
                };
                let dy: f32 = match vertical_alignment {
                    iced_core::alignment::Vertical::Top => 0.0,
                    iced_core::alignment::Vertical::Center => -text_size.height / 2.0,
                    iced_core::alignment::Vertical::Bottom => -text_size.height,
                };

                let corners: [Point<f32>; 4] = [
                    Point::new(dx, dy),
                    Point::new(dx + text_size.width, dy),
                    Point::new(dx + text_size.width, dy + text_size.height),
                    Point::new(dx, dy + text_size.height),
                ];

                let cos_r: f32 = rotation_rads.cos();
                let sin_r: f32 = rotation_rads.sin();

                let mut rotated_corners = Vec::with_capacity(4);
                for c in corners {
                    let rx: f32 = c.x * cos_r - c.y * sin_r;
                    let ry: f32 = c.x * sin_r + c.y * cos_r;
                    rotated_corners.push(Point::new(screen_pos.x + rx, screen_pos.y + ry));
                }

                ResolvedArea::Polygon {
                    points: rotated_corners,
                }
            }
            Self::Custom(custom) => ResolvedArea::Custom(custom.resolve_area(transform)),
        }
    }
}
#[derive(Debug)]
pub enum ResolvedArea {
    Rect(Rectangle),
    LineSegment {
        p1: Point,
        p2: Point,
        stroke_width_px: f32,
    },
    Ellipse {
        center: Point,
        rx: f32,
        ry: f32,
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
        radius_px: f32,
        vertices: u16,
        rotation_rads: f32,
    },
    Polyline {
        points: Vec<Point>,
        stroke_width_px: f32,
    },
    Arc {
        center: Point,
        radius_outer: f32,
        radius_inner: f32,
        start_angle: f32,
        end_angle: f32,
    },
    /// The escape hatch for custom screen-space hit testing.
    Custom(Box<dyn HitTest>),
}

impl ResolvedArea {
    pub fn bounding_box(&self) -> Rectangle {
        match self {
            Self::Rect(rect) => *rect,
            Self::LineSegment {
                p1,
                p2,
                stroke_width_px,
            } => {
                let padding = *stroke_width_px / 2.0;
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
            Self::Ellipse { center, rx, ry } => Rectangle {
                x: center.x - rx,
                y: center.y - ry,
                width: rx * 2.0,
                height: ry * 2.0,
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
            Self::RegularPolygon {
                center, radius_px, ..
            } => Rectangle {
                x: center.x - radius_px,
                y: center.y - radius_px,
                width: radius_px * 2.0,
                height: radius_px * 2.0,
            },
            Self::Polyline {
                points,
                stroke_width_px,
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
                let padding = *stroke_width_px / 2.0;
                Rectangle {
                    x: min_x - padding,
                    y: min_y - padding,
                    width: (max_x - min_x) + stroke_width_px,
                    height: (max_y - min_y) + stroke_width_px,
                }
            }
            Self::Arc {
                center,
                radius_outer,
                ..
            } => Rectangle {
                // A conservative bounding box covering the full circle
                x: center.x - radius_outer,
                y: center.y - radius_outer,
                width: radius_outer * 2.0,
                height: radius_outer * 2.0,
            },
            Self::Custom(custom_test) => custom_test.bounding_box(),
        }
    }

    pub fn intersects(&self, query: &InteractionQuery) -> bool {
        match (self, query) {
            (
                Self::Rect(rect),
                InteractionQuery::Point {
                    position,
                    tolerance_px,
                },
            ) => {
                let expanded = Rectangle {
                    x: rect.x - tolerance_px,
                    y: rect.y - tolerance_px,
                    width: rect.width + (tolerance_px * 2.0),
                    height: rect.height + (tolerance_px * 2.0),
                };
                expanded.contains(*position)
            }
            (Self::Rect(rect), InteractionQuery::Bounds(bounds)) => {
                math::rect_intersects_rect(rect, bounds)
            }
            (
                Self::LineSegment {
                    p1,
                    p2,
                    stroke_width_px,
                },
                InteractionQuery::Point {
                    position,
                    tolerance_px,
                },
            ) => {
                let distance = math::distance_point_to_segment(*position, *p1, *p2);
                distance <= (stroke_width_px / 2.0) + tolerance_px
            }
            (Self::LineSegment { p1, p2, .. }, InteractionQuery::Bounds(bounds)) => {
                math::line_intersects_rect(*p1, *p2, bounds)
            }
            (
                Self::Ellipse { center, rx, ry },
                InteractionQuery::Point {
                    position,
                    tolerance_px,
                },
            ) => math::point_in_ellipse(*position, *center, *rx, *ry, *tolerance_px),
            (Self::Ellipse { center, rx, ry }, InteractionQuery::Bounds(bounds)) => {
                math::rect_intersects_ellipse(bounds, *center, *rx, *ry)
            }
            (
                Self::Triangle { p1, p2, p3 },
                InteractionQuery::Point {
                    position,
                    tolerance_px,
                },
            ) => math::point_in_triangle(*position, *p1, *p2, *p3, *tolerance_px),
            (Self::Triangle { p1, p2, p3 }, InteractionQuery::Bounds(bounds)) => {
                math::rect_intersects_triangle(bounds, *p1, *p2, *p3)
            }
            (Self::Polygon { points }, InteractionQuery::Point { position, .. }) => {
                // Polygons don't strictly need tolerance for filled areas, just Ray-Casting
                math::point_in_polygon(*position, points)
            }
            (Self::Polygon { points }, InteractionQuery::Bounds(bounds)) => {
                math::rect_intersects_polygon(bounds, points)
            }
            (
                Self::Polyline {
                    points,
                    stroke_width_px,
                },
                InteractionQuery::Point {
                    position,
                    tolerance_px,
                },
            ) => math::point_in_polyline(*position, points, *stroke_width_px, *tolerance_px),
            // --- REGULAR POLYGON ---
            (
                Self::RegularPolygon {
                    center,
                    radius_px,
                    vertices,
                    rotation_rads,
                },
                InteractionQuery::Point { position, .. },
            ) => {
                let mut pts = Vec::with_capacity(*vertices as usize);
                let angle_step = std::f32::consts::TAU / (*vertices as f32);
                for i in 0..*vertices {
                    let angle = *rotation_rads + (i as f32) * angle_step;
                    pts.push(Point::new(
                        center.x + radius_px * angle.cos(),
                        center.y + radius_px * angle.sin(),
                    ));
                }
                math::point_in_polygon(*position, &pts)
            }
            (
                Self::RegularPolygon {
                    center,
                    radius_px,
                    vertices,
                    rotation_rads,
                },
                InteractionQuery::Bounds(bounds),
            ) => {
                let mut pts = Vec::with_capacity(*vertices as usize);
                let angle_step = std::f32::consts::TAU / (*vertices as f32);
                for i in 0..*vertices {
                    let angle = *rotation_rads + (i as f32) * angle_step;
                    pts.push(Point::new(
                        center.x + radius_px * angle.cos(),
                        center.y + radius_px * angle.sin(),
                    ));
                }
                math::rect_intersects_polygon(bounds, &pts)
            }
            (Self::Polyline { points, .. }, InteractionQuery::Bounds(bounds)) => {
                math::rect_intersects_polyline(bounds, points)
            }
            (
                Self::Arc {
                    center,
                    radius_outer,
                    radius_inner,
                    start_angle,
                    end_angle,
                },
                InteractionQuery::Point { position, .. },
            ) => math::point_in_arc(
                *position,
                *center,
                *radius_inner,
                *radius_outer,
                *start_angle,
                *end_angle,
            ),
            (
                Self::Arc {
                    center,
                    radius_outer,
                    ..
                },
                InteractionQuery::Bounds(bounds),
            ) => math::rect_intersects_arc(bounds, *center, *radius_outer),
            (Self::Custom(custom_test), q) => custom_test.intersects(q),
        }
    }
}
