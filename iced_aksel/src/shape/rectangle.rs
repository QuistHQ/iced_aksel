use crate::{
    Measure, Shape, Stroke,
    interaction::{Area, IntoArea},
    plot,
    render::Primitive,
};
use aksel::{Float, PlotPoint};
use iced_core::{Color, Point, Size};
use std::fmt::Debug;

#[derive(Debug, Clone)]
enum Geometry<D> {
    Corners {
        p1: PlotPoint<D>,
        p2: PlotPoint<D>,
    },
    Centered {
        center: PlotPoint<D>,
        width: Measure<D>,
        height: Measure<D>,
    },
}

#[derive(Debug, Clone)]
pub struct Rectangle<D> {
    geometry: Geometry<D>,
    pub fill: Option<Color>,
    pub stroke: Option<Stroke<D>>,
}

impl<D: Float, R: crate::Renderer> Shape<D, R> for Rectangle<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        let Self {
            geometry,
            fill,
            stroke,
        } = self;

        // 1. Calculate visual screen coordinates
        let (screen_min, screen_max) = match &geometry {
            Geometry::Corners { p1, p2 } => {
                let x1 = ctx.x_to_screen(&p1.x);
                let y1 = ctx.y_to_screen(&p1.y);
                let x2 = ctx.x_to_screen(&p2.x);
                let y2 = ctx.y_to_screen(&p2.y);

                (
                    Point::new(x1.min(x2), y1.min(y2)),
                    Point::new(x1.max(x2), y1.max(y2)),
                )
            }
            Geometry::Centered {
                center,
                width,
                height,
            } => {
                let center_x = ctx.x_to_screen(&center.x);
                let center_y = ctx.y_to_screen(&center.y);

                let width_pixels = width.resolve_x(ctx);
                let height_pixels = height.resolve_y(ctx);

                let half_width = width_pixels / 2.0;
                let half_height = height_pixels / 2.0;

                (
                    Point::new(center_x - half_width, center_y - half_height),
                    Point::new(center_x + half_width, center_y + half_height),
                )
            }
        };

        // 3. Dispatch visual rendering
        let stroke = stroke.map(|s| s.resolve(ctx));

        ctx.add_primitive(Primitive::Rectangle {
            xy1: screen_min,
            xy2: screen_max,
            fill,
            stroke,
        });
    }
}

impl<D: Float> Rectangle<D> {
    pub const fn corners(p1: PlotPoint<D>, p2: PlotPoint<D>) -> Self {
        Self {
            geometry: Geometry::Corners { p1, p2 },
            fill: None,
            stroke: None,
        }
    }

    pub const fn centered(center: PlotPoint<D>, width: Measure<D>, height: Measure<D>) -> Self {
        Self {
            geometry: Geometry::Centered {
                center,
                width,
                height,
            },
            fill: None,
            stroke: None,
        }
    }

    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    pub const fn stroke(mut self, stroke: Stroke<D>) -> Self {
        self.stroke = Some(stroke);
        self
    }
}

impl<'a, D: Float, Renderer: crate::Renderer> IntoArea<'a, D, Renderer> for &Rectangle<D> {
    fn resolve_area(self, ctx: &plot::Context<'a, D, Renderer>) -> Area {
        match self.geometry {
            Geometry::Corners { p1, p2 } => {
                let p1 = ctx.chart_to_screen(&p1);
                let p2 = ctx.chart_to_screen(&p2);
                let left = p1.x.min(p2.x);
                let right = p1.x.max(p2.x);
                let top = p1.y.min(p2.y);
                let bottom = p1.y.max(p2.y);

                let width = right - left;
                let height = bottom - top;

                let top_left = Point::new(left, top);

                Area::Rectangle {
                    top_left,
                    size: Size::new(width, height),
                }
            }
            Geometry::Centered {
                center,
                width,
                height,
            } => {
                let center = ctx.chart_to_screen(&center);
                let width = width.resolve_x(ctx);
                let height = height.resolve_y(ctx);
                let half_width = width / 2.0;
                let half_height = height / 2.0;
                let top_left = Point::new(center.x - half_width, center.y - half_height);

                Area::Rectangle {
                    top_left,
                    size: Size::new(width, height),
                }
            }
        }
    }
}
