use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use aksel::{Float, PlotPoint, Transform};
use iced_core::{Color, Point};

#[derive(Debug, Clone)]
pub struct Polygon<D> {
    center: PlotPoint<D>,
    radius: Measure<D>,
    vertices: u16,
    rotation: f32,
    fill: Option<Color>,
    stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Polygon<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Polygon<D> {
    pub const fn new(center: PlotPoint<D>, radius: Measure<D>, vertices: u16) -> Self {
        Self {
            center,
            radius,
            vertices,
            rotation: 0.0,
            fill: None,
            stroke: None,
        }
    }
    pub const fn rotation(mut self, degrees: f32) -> Self {
        self.rotation = degrees;
        self
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
        if self.vertices < 3 {
            return;
        }

        let center = Point::new(
            transform.x_to_screen(&self.center.x),
            transform.y_to_screen(&self.center.y),
        );

        let radius_px = match self.radius {
            Measure::Screen(px) => px,
            Measure::Plot(u) => {
                let p0 = transform.x_to_screen(&D::zero());
                let p1 = transform.x_to_screen(&u);
                (p1 - p0).abs()
            }
        };

        if radius_px < 0.5 {
            return;
        }

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

        tess.draw_polygon(
            buffer,
            center,
            radius_px,
            self.vertices,
            self.rotation,
            self.fill,
            stroke_info,
        );
    }
}
