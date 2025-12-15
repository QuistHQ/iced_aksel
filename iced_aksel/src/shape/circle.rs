use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};

use aksel::{Float, PlotPoint, Transform};
use iced_core::Color;

#[derive(Debug, Clone)]
pub struct Circle<D> {
    pub center: PlotPoint<D>,
    pub radius: Measure<D>,
    pub fill: Option<Color>,
    pub stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Circle<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Circle<D> {
    // =========================================================================
    //  Constructors
    // =========================================================================

    pub const fn new(center: PlotPoint<D>, radius: Measure<D>) -> Self {
        Self {
            center,
            radius,
            fill: None,
            stroke: None,
        }
    }

    // =========================================================================
    //  Builder Methods
    // =========================================================================

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

    // =========================================================================
    //  Tessellation Logic
    // =========================================================================

    fn tessellate(
        self,
        transform: &Transform<D, f32, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellator,
    ) {
        // 1. Resolve Geometry to Screen Space
        let cx = transform.x_to_screen(&self.center.x);
        let cy = transform.y_to_screen(&self.center.y);

        // Calculate Radius in Pixels.
        let r = match self.radius {
            Measure::Screen(pixels) => pixels,
            Measure::Plot(units) => {
                let x0 = transform.x_to_screen(&D::zero());
                let x1 = transform.x_to_screen(&units);
                let dx = (x1 - x0).abs();

                let y0 = transform.y_to_screen(&D::zero());
                let y1 = transform.y_to_screen(&units);
                let dy = (y1 - y0).abs();

                dx.min(dy)
            }
        };

        // 2. Resolve Stroke Thickness
        let stroke_info = self.stroke.as_ref().and_then(|stroke| {
            let width = match stroke.thickness {
                Measure::Screen(w) => w,
                Measure::Plot(w) => {
                    let x0 = transform.x_to_screen(&D::zero());
                    let x1 = transform.x_to_screen(&w);
                    let dx = (x1 - x0).abs();

                    let y0 = transform.y_to_screen(&D::zero());
                    let y1 = transform.y_to_screen(&w);
                    let dy = (y1 - y0).abs();

                    dx.min(dy)
                }
            };
            if width < 0.1 {
                None
            } else {
                Some((stroke, width))
            }
        });

        // 3. Delegate to the Tessellator Facade
        tess.draw_circle(buffer, cx, cy, r, self.fill, stroke_info);
    }
}
