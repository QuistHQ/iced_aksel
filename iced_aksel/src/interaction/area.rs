use crate::Measure;
use crate::interaction::{InteractionQuery, math};
use aksel::{Float, PlotPoint, Transform};
use iced_core::{Point, Rectangle};
use std::fmt::Debug;

/// A trait for performing precise screen-space collision detection.
pub trait HitTest: Debug + Send + Sync {
    /// Returns the fast broad-phase bounding box in screen pixels.
    fn bounding_box(&self) -> Rectangle;

    /// Performs the precise narrow-phase check against the user's interaction query.
    fn intersects(&self, query: &InteractionQuery) -> bool;
}

/// A trait for custom geometries that can be resolved into screen-space hit areas.
pub trait ResolvableArea<D: Float>: Debug + Send + Sync {
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
    /// The escape hatch for custom data-space interactions.
    Custom(Box<dyn ResolvableArea<D>>),
}

impl<D: Float> Area<D> {
    pub(super) fn resolve(self, transform: &Transform<D, f32, f32>) -> ResolvedArea {
        match self {
            Self::Rect {
                x,
                y,
                width,
                height,
            } => {
                // For Plot measures, we need both corners to handle axis inversions (e.g., Y-axis flip)
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
            Self::Custom(custom_area) => ResolvedArea::Custom(custom_area.resolve_area(transform)),
            _ => todo!("Resolve other areas"),
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
            (Self::Custom(custom_test), q) => custom_test.intersects(q),
        }
    }
}
