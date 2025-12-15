use crate::{
    Float, Measure, PlotPoint, Shape, Stroke, Transform,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use iced_core::Point;

#[derive(Debug, Clone)]
pub struct Line<D> {
    pub p1: PlotPoint<D>,
    pub p2: PlotPoint<D>,
    pub stroke: Stroke<D>,
    pub extend_start: bool,
    pub extend_end: bool,
    pub arrow_start: bool,
    pub arrow_end: bool,
    pub arrow_size: f32,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Line<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Line<D> {
    pub const fn new(p1: PlotPoint<D>, p2: PlotPoint<D>, stroke: Stroke<D>) -> Self {
        Self {
            p1,
            p2,
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
    pub const fn infinite(mut self) -> Self {
        self.extend_start = true;
        self.extend_end = true;
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
    pub const fn arrows(mut self, enable: bool) -> Self {
        self.arrow_start = enable;
        self.arrow_end = enable;
        self
    }
    pub const fn arrow_size(mut self, multiplier: f32) -> Self {
        self.arrow_size = multiplier;
        self
    }

    fn tessellate(
        self,
        transform: &Transform<D, f32, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellator,
    ) {
        let raw_start = Point::new(
            transform.x_to_screen(&self.p1.x),
            transform.y_to_screen(&self.p1.y),
        );
        let raw_end = Point::new(
            transform.x_to_screen(&self.p2.x),
            transform.y_to_screen(&self.p2.y),
        );

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

        tess.draw_line(
            buffer,
            raw_start,
            raw_end,
            &self.stroke,
            width,
            clip_bounds,
            (self.extend_start, self.extend_end),
            (self.arrow_start, self.arrow_end, self.arrow_size),
        );
    }
}
