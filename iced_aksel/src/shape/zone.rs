use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use aksel::{Float, PlotPoint, Transform};
use iced_core::{Color, Point};

#[derive(Debug, Clone)]
pub struct Zone<D> {
    points: Vec<PlotPoint<D>>,
    fill: Option<Color>,
    stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Zone<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Zone<D> {
    pub const fn new(points: Vec<PlotPoint<D>>) -> Self {
        Self {
            points,
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

    fn tessellate(
        self,
        transform: &Transform<D, f32, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellator,
    ) {
        if self.points.len() < 3 {
            return;
        }

        let screen_points: Vec<Point> = self
            .points
            .iter()
            .map(|p| Point::new(transform.x_to_screen(&p.x), transform.y_to_screen(&p.y)))
            .collect();

        let stroke_info = self.stroke.as_ref().map(|s| {
            let width = match s.thickness {
                Measure::Screen(w) => w,
                Measure::Plot(w) => {
                    let p0 = transform.x_to_screen(&D::zero());
                    let p1 = transform.x_to_screen(&w);
                    (p1 - p0).abs()
                }
            };
            (s, width)
        });

        tess.draw_zone(buffer, &screen_points, self.fill, stroke_info);
    }
}
