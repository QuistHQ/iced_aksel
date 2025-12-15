use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellator},
};
use aksel::{Float, PlotPoint, Transform, scale};
use iced_core::Color;

#[derive(Debug, Clone)]
pub struct Rectangle<D> {
    center: PlotPoint<D>,
    width: Measure<D>,
    height: Measure<D>,
    fill: Option<Color>,
    stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Rectangle<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        // We now request the 'tess' (Tessellator) from the context
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Rectangle<D> {
    // =========================================================================
    //  Constructors
    // =========================================================================

    pub const fn new(center: PlotPoint<D>, width: Measure<D>, height: Measure<D>) -> Self {
        Self {
            center,
            width,
            height,
            fill: None,
            stroke: None,
        }
    }

    pub fn from_corners(p1: PlotPoint<D>, p2: PlotPoint<D>) -> Self {
        let (x_min, x_max) = scale::util::sorted_pair(p1.x, p2.x);
        let (y_min, y_max) = scale::util::sorted_pair(p1.y, p2.y);
        let two = D::one() + D::one();

        Self {
            center: PlotPoint {
                x: (x_min + x_max) / two,
                y: (y_min + y_max) / two,
            },
            width: Measure::Plot(x_max - x_min),
            height: Measure::Plot(y_max - y_min),
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

    #[inline]
    pub const fn no_fill(mut self) -> Self {
        self.fill = None;
        self
    }

    // =========================================================================
    //  Tessellation Logic
    // =========================================================================

    /// Calculates the screen-space boundaries (min_x, max_x, min_y, max_y).
    fn resolve_bounds(&self, transform: &Transform<D, f32, f32>) -> (f32, f32, f32, f32) {
        let half_const = D::from(0.5).unwrap();

        // Resolve Width
        let (x_min, x_max) = match &self.width {
            Measure::Screen(px) => {
                let c = transform.x_to_screen(&self.center.x);
                let half = px * 0.5;
                (c - half, c + half)
            }
            Measure::Plot(width) => {
                let half_w = *width * half_const;
                let p1 = transform.x_to_screen(&(self.center.x - half_w));
                let p2 = transform.x_to_screen(&(self.center.x + half_w));
                if p1 < p2 { (p1, p2) } else { (p2, p1) }
            }
        };

        // Resolve Height
        let (y_min, y_max) = match &self.height {
            Measure::Screen(px) => {
                let c = transform.y_to_screen(&self.center.y);
                let half = px * 0.5;
                (c - half, c + half)
            }
            Measure::Plot(height) => {
                let half_h = *height * half_const;
                let p1 = transform.y_to_screen(&(self.center.y - half_h));
                let p2 = transform.y_to_screen(&(self.center.y + half_h));
                if p1 < p2 { (p1, p2) } else { (p2, p1) }
            }
        };

        (x_min, x_max, y_min, y_max)
    }

    fn tessellate(
        self,
        transform: &Transform<D, f32, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellator,
    ) {
        let (x_min, x_max, y_min, y_max) = self.resolve_bounds(transform);

        // 1. Resolve Stroke Thickness
        let stroke_info = self.stroke.as_ref().and_then(|stroke| {
            let (th_x, th_y) = match stroke.thickness {
                Measure::Screen(px) => (px, px),
                Measure::Plot(units) => (
                    (transform.x_to_screen(&units) - transform.x_to_screen(&D::zero())).abs(),
                    (transform.y_to_screen(&units) - transform.y_to_screen(&D::zero())).abs(),
                ),
            };

            if th_x < 0.1 && th_y < 0.1 {
                None
            } else {
                Some((stroke, th_x, th_y))
            }
        });

        // 2. Delegate to the Tessellator Facade
        // The tessellator now handles the "Manual" vs "Lyon" strategy selection,
        // consumption checks, and bleed fixing.
        tess.draw_rectangle(buffer, x_min, y_min, x_max, y_max, self.fill, stroke_info);
    }
}
