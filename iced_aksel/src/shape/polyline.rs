use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use aksel::{Float, PlotPoint, Transform};
use iced_core::Point;

#[derive(Debug, Clone)]
pub struct Polyline<D> {
    pub points: Vec<PlotPoint<D>>,
    pub stroke: Stroke<D>,
    pub extend_start: bool,
    pub extend_end: bool,
    pub arrow_start: bool,
    pub arrow_end: bool,
    pub arrow_size: f32,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Polyline<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Polyline<D> {
    pub const fn new(points: Vec<PlotPoint<D>>, stroke: Stroke<D>) -> Self {
        Self {
            points,
            stroke,
            extend_start: false,
            extend_end: false,
            arrow_start: false,
            arrow_end: false,
            arrow_size: 3.0,
        }
    }
    pub const fn extend_start(mut self, enable: bool) -> Self {
        self.extend_start = enable;
        self
    }
    pub const fn extend_end(mut self, enable: bool) -> Self {
        self.extend_end = enable;
        self
    }
    pub const fn arrow_start(mut self, enable: bool) -> Self {
        self.arrow_start = enable;
        self
    }
    pub const fn arrow_end(mut self, enable: bool) -> Self {
        self.arrow_end = enable;
        self
    }
    pub const fn arrow_size(mut self, size: f32) -> Self {
        self.arrow_size = size;
        self
    }

    fn tessellate(
        self,
        transform: &Transform<D, f32, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellator,
    ) {
        if self.points.len() < 2 {
            return;
        }

        let width = match self.stroke.thickness {
            Measure::Screen(w) => w,
            Measure::Plot(w) => {
                let p0 = transform.x_to_screen(&D::zero());
                let p1 = transform.x_to_screen(&w);
                (p1 - p0).abs()
            }
        };

        // Fix: Map ScreenRect to iced_core::Rectangle manually
        let b = transform.screen_bounds();
        let clip_bounds = iced_core::Rectangle {
            x: b.x,
            y: b.y,
            width: b.width,
            height: b.height,
        };

        // Stream points lazily
        let points_iter = self
            .points
            .iter()
            .map(|p| Point::new(transform.x_to_screen(&p.x), transform.y_to_screen(&p.y)));

        tess.draw_polyline(
            buffer,
            points_iter,
            &self.stroke,
            width,
            clip_bounds,
            (self.extend_start, self.extend_end),
            (self.arrow_start, self.arrow_end, self.arrow_size),
        );
    }
}
