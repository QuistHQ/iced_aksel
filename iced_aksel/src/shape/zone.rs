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
///
/// **Note:** This shape supports Concave geometry via the `earcutr` algorithm.
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

        // 1. Resolve to Screen Coordinates
        let screen_points: Vec<Point> = self
            .points
            .iter()
            .map(|p| Point::new(transform.x_to_screen(&p.x), transform.y_to_screen(&p.y)))
            .collect();

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

        // 2. Render Fill
        if let Some(color) = self.fill {
            // OPTIMIZATION CHECK:
            // If the shape is strictly convex (like a simple quad), we can use the fast Fan.
            // If it's the Spectrum (which has hundreds of points and is likely concave), we MUST use Earcut.
            if self.is_convex(&screen_points) {
                if maybe_stroke.is_some() {
                    let inset = self.compute_inset_polygon(&screen_points, 0.5);
                    self.add_fan(buffer, &inset, color);
                } else {
                    self.add_fan(buffer, &screen_points, color);
                }
            } else {
                // ROBUST PATH: Earcut
                self.tessellate_earcut(buffer, &screen_points, color);
            }
        }

        // 3. Render Stroke
        if let Some((width, stroke)) = maybe_stroke {
            match stroke.style {
                StrokeStyle::Solid => {
                    // Only use manual stroke optimization if Convex.
                    // Lyon is better at stroking complex concave lines (miter limits etc).
                    if self.is_convex(&screen_points) {
                        let inner = self.compute_inset_polygon(&screen_points, width);
                        self.add_ring(buffer, &screen_points, &inner, stroke.fill);
                    } else {
                        tess.stroke_polyline(buffer, screen_points, stroke, width, true);
                    }
                }
                _ => {
                    tess.stroke_polyline(buffer, screen_points, stroke, width, true);
                }
            }
        }
    }

    // --- Triangulation Strategies ---

    fn tessellate_earcut(&self, buffer: &mut MeshBuffer, points: &[Point], color: Color) {
        // Flatten for Earcut: [x0, y0, x1, y1, ...]
        let flat_coords: Vec<f64> = points
            .iter()
            .flat_map(|p| [p.x as f64, p.y as f64])
            .collect();

        // Run Earcut
        if let Ok(indices) = earcutr::earcut(&flat_coords, &[], 2) {
            let c = pack(color);
            let vertices: Vec<SolidVertex2D> = points
                .iter()
                .map(|p| SolidVertex2D {
                    position: p.to_array(),
                    color: c,
                })
                .collect();

            let mesh_indices: Vec<u32> = indices.iter().map(|&i| i as u32).collect();
            buffer.add(&mesh_indices, &vertices);
        }
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

        for i in 1..(points.len() - 1) {
            indices.extend_from_slice(&[0, i as u32, (i + 1) as u32]);
        }
        buffer.add(&indices, &vertices);
    }

    // --- Math Helpers ---

    fn is_convex(&self, points: &[Point]) -> bool {
        if points.len() < 4 {
            return true;
        }
        // Optimization: For large shapes like Spectrum (N > 50),
        // checking convexity is O(N) which is fast, BUT Earcut is O(N log N).
        // It might be safer to just default to Earcut for large N to avoid edge cases.
        if points.len() > 20 {
            return false;
        }

        let mut sign = 0.0;
        let n = points.len();
        for i in 0..n {
            let p1 = points[i];
            let p2 = points[(i + 1) % n];
            let p3 = points[(i + 2) % n];
            let v1 = p2 - p1;
            let v2 = p3 - p2;
            let cross = v1.x.mul_add(v2.y, -(v1.y * v2.x));
            if cross.abs() < 1e-5 {
                continue;
            }
            if sign == 0.0 {
                sign = cross;
            } else if cross * sign < 0.0 {
                return false;
            }
        }
        true
    }

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
        let miter = Vector::new(-tangent.y, tangent.x);
        let n1 = Vector::new(-v1.y, v1.x);
        let dot = miter.dot(n1);
        let len = if dot.abs() < 1e-4 { dist } else { dist / dot };
        let limit = len.min(dist * 3.0);
        curr + miter * limit
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
