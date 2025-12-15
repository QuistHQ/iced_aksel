use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use aksel::{Float, PlotPoint, Transform};
use iced_core::Color;

#[derive(Debug, Clone)]
pub struct Arc<D> {
    pub center: PlotPoint<D>,
    pub radius: Measure<D>,
    pub inner_radius: Measure<D>,
    pub start_angle: f32, // Radians
    pub end_angle: f32,   // Radians
    pub fill: Option<Color>,
    pub stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Arc<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Arc<D> {
    pub const fn new(
        center: PlotPoint<D>,
        radius: Measure<D>,
        start_angle: f32,
        end_angle: f32,
    ) -> Self {
        Self {
            center,
            radius,
            inner_radius: Measure::Screen(0.0),
            start_angle,
            end_angle,
            fill: None,
            stroke: None,
        }
    }

    pub const fn inner_radius(mut self, radius: Measure<D>) -> Self {
        self.inner_radius = radius;
        self
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
        let cx = transform.x_to_screen(&self.center.x);
        let cy = transform.y_to_screen(&self.center.y);

        let outer_r = self.resolve_length(transform, self.radius);
        let inner_r = self.resolve_length(transform, self.inner_radius);

        let stroke_info = self.stroke.as_ref().and_then(|stroke| {
            let width = self.resolve_length(transform, stroke.thickness);
            if width < 0.1 {
                None
            } else {
                Some((stroke, width))
            }
        });

        tess.draw_arc(
            buffer,
            cx,
            cy,
            inner_r,
            outer_r,
            self.start_angle,
            self.end_angle,
            self.fill,
            stroke_info,
        );
    }

    fn resolve_length(&self, transform: &Transform<D, f32, f32>, len: Measure<D>) -> f32 {
        match len {
            Measure::Screen(px) => px,
            Measure::Plot(units) => {
                let p0_x = transform.x_to_screen(&D::zero());
                let p1_x = transform.x_to_screen(&units);
                let size_x = (p1_x - p0_x).abs();

                let p0_y = transform.y_to_screen(&D::zero());
                let p1_y = transform.y_to_screen(&units);
                let size_y = (p1_y - p0_y).abs();

                size_x.min(size_y)
            }
        }
    }
}
