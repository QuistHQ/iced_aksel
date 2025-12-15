use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellators},
    stroke::StrokeStyle,
};

use aksel::{Float, PlotPoint, Transform};
use iced_core::Color;
use iced_graphics::{color::pack, mesh::SolidVertex2D};
use lyon::math::{Point, Vector};

/// A solid area defined by an arbitrary list of coordinate points.
///
/// Use this for highlighting regions, ranges, background zones, or filled areas.
/// **Note:** This shape is optimized for simple, convex polygons.
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
        tess: &mut Tessellators,
    ) {
        if self.points.len() < 3 {
            return;
        }

        let screen_points: Vec<Point> = self
            .points
            .iter()
            .map(|p| Point::new(transform.x_to_screen(&p.x), transform.y_to_screen(&p.y)))
            .collect();

        // Resolve Stroke
        let maybe_stroke = self.stroke.as_ref().map(|s| {
            let width = match s.thickness {
                Measure::Screen(w) => w,
                Measure::Plot(w) => {
                    let p0 = transform.x_to_screen(&D::zero());
                    let p1 = transform.x_to_screen(&w);
                    (p1 - p0).abs()
                }
            };
            (width, s)
        });

        // 1. Render Fill (Simple Fan)
        if let Some(color) = self.fill {
            if maybe_stroke.is_some() {
                // Inset for bleed
                let inset = self.compute_inset_polygon(&screen_points, 0.5);
                self.add_fan(buffer, &inset, color);
            } else {
                self.add_fan(buffer, &screen_points, color);
            }
        }

        // 2. Render Stroke
        if let Some((width, stroke)) = maybe_stroke {
            match stroke.style {
                StrokeStyle::Solid => {
                    // Manual Ring Stitching
                    let inner = self.compute_inset_polygon(&screen_points, width);
                    self.add_ring(buffer, &screen_points, &inner, stroke.fill);
                }
                _ => {
                    // Fallback to Lyon for dashes
                    tess.stroke_polyline(buffer, screen_points, stroke, width, true);
                }
            }
        }
    }

    // --- Math Helpers ---

    fn compute_inset_polygon(&self, points: &[Point], dist: f32) -> Vec<Point> {
        let n = points.len();
        let mut new_points = Vec::with_capacity(n);
        for i in 0..n {
            let prev = points[(i + n - 1) % n];
            let curr = points[i];
            let next = points[(i + 1) % n];
            new_points.push(self.compute_miter_vertex(prev, curr, next, dist));
        }
        new_points
    }

    fn compute_miter_vertex(&self, prev: Point, curr: Point, next: Point, dist: f32) -> Point {
        let v1 = (curr - prev).normalize();
        let v2 = (next - curr).normalize();
        let tangent = (v1 + v2).normalize();
        let miter = Vector::new(-tangent.y, tangent.x); // Normal to tangent
        let n1 = Vector::new(-v1.y, v1.x); // Normal of first segment

        let dot = miter.dot(n1);
        let len = if dot.abs() < 1e-4 { dist } else { dist / dot };
        // Limit miter length to prevent spikes on acute angles
        let limit = len.min(dist * 3.0);

        curr + miter * limit
    }

    fn add_fan(&self, buffer: &mut MeshBuffer, points: &[Point], color: Color) {
        let c = pack(color);
        let mut vertices = Vec::with_capacity(points.len());
        let mut indices = Vec::with_capacity((points.len() - 2) * 3);

        for p in points {
            vertices.push(SolidVertex2D {
                position: p.to_array(),
                color: c,
            });
        }

        // Simple Triangle Fan from vertex 0
        for i in 1..(points.len() - 1) {
            indices.extend_from_slice(&[0, i as u32, (i + 1) as u32]);
        }
        buffer.add(&indices, &vertices);
    }

    fn add_ring(&self, buffer: &mut MeshBuffer, outer: &[Point], inner: &[Point], color: Color) {
        if outer.len() != inner.len() {
            return;
        }
        let c = pack(color);
        let n = outer.len();
        let mut vertices = Vec::with_capacity(n * 2);
        let mut indices = Vec::with_capacity(n * 6);

        for p in outer {
            vertices.push(SolidVertex2D {
                position: p.to_array(),
                color: c,
            });
        }
        for p in inner {
            vertices.push(SolidVertex2D {
                position: p.to_array(),
                color: c,
            });
        }

        for i in 0..n {
            let next = (i + 1) % n;
            let o_curr = i as u32;
            let o_next = next as u32;
            let i_curr = (i + n) as u32;
            let i_next = (next + n) as u32;
            indices.extend_from_slice(&[o_curr, o_next, i_curr, o_next, i_next, i_curr]);
        }
        buffer.add(&indices, &vertices);
    }
}
