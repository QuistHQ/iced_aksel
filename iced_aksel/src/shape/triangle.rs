use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use aksel::{Float, PlotPoint, Transform};
use iced_core::{Color, Point};

#[derive(Debug, Clone)]
enum Geometry<D> {
    Vertices([PlotPoint<D>; 3]),
    Equilateral {
        center: PlotPoint<D>,
        radius: Measure<D>,
    },
}

#[derive(Debug, Clone)]
pub struct Triangle<D> {
    geometry: Geometry<D>,
    pub fill: Option<Color>,
    pub stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Triangle<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Triangle<D> {
    pub const fn new(p1: PlotPoint<D>, p2: PlotPoint<D>, p3: PlotPoint<D>) -> Self {
        Self {
            geometry: Geometry::Vertices([p1, p2, p3]),
            fill: None,
            stroke: None,
        }
    }

    pub const fn equilateral(center: PlotPoint<D>, radius: Measure<D>) -> Self {
        Self {
            geometry: Geometry::Equilateral { center, radius },
            fill: None,
            stroke: None,
        }
    }

    #[inline]
    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    #[inline]
    pub const fn stroke(mut self, stroke: Stroke<D>) -> Self {
        self.stroke = Some(stroke);
        self
    }

    fn tessellate(
        self,
        transform: &Transform<D, f32, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellator,
    ) {
        let (p1, p2, p3) = match self.geometry {
            Geometry::Vertices(pts) => (
                Point::new(
                    transform.x_to_screen(&pts[0].x),
                    transform.y_to_screen(&pts[0].y),
                ),
                Point::new(
                    transform.x_to_screen(&pts[1].x),
                    transform.y_to_screen(&pts[1].y),
                ),
                Point::new(
                    transform.x_to_screen(&pts[2].x),
                    transform.y_to_screen(&pts[2].y),
                ),
            ),
            Geometry::Equilateral { center, radius } => {
                let cx = transform.x_to_screen(&center.x);
                let cy = transform.y_to_screen(&center.y);
                let r_px = match radius {
                    Measure::Screen(px) => px,
                    Measure::Plot(units) => {
                        let p0 = transform.x_to_screen(&D::zero());
                        let p1 = transform.x_to_screen(&units);
                        (p1 - p0).abs()
                    }
                };
                let a1 = std::f32::consts::FRAC_PI_2;
                let a2 = std::f32::consts::PI * 7.0 / 6.0;
                let a3 = std::f32::consts::PI * 11.0 / 6.0;
                (
                    Point::new(a1.cos().mul_add(r_px, cx), a1.sin().mul_add(-r_px, cy)),
                    Point::new(a2.cos().mul_add(r_px, cx), a2.sin().mul_add(-r_px, cy)),
                    Point::new(a3.cos().mul_add(r_px, cx), a3.sin().mul_add(-r_px, cy)),
                )
            }
        };

        let stroke_info = self.stroke.as_ref().and_then(|stroke| {
            let width = match stroke.thickness {
                Measure::Screen(w) => w,
                Measure::Plot(w) => {
                    let p0 = transform.x_to_screen(&D::zero());
                    let p1 = transform.x_to_screen(&w);
                    (p1 - p0).abs()
                }
            };
            if width < 0.1 {
                None
            } else {
                Some((stroke, width))
            }
        });

        tess.draw_triangle(buffer, p1, p2, p3, self.fill, stroke_info);
    }
}
