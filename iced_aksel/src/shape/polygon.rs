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

/// A polygon defined by an arbitrary list of vertices.
///
/// Supports both convex and concave polygons.
/// - **Convex polygons** use optimized manual tessellation (Triangle Fan) for maximum performance.
/// - **Concave polygons** use the robust `earcut` algorithm via `earcutr` to handle complex geometry safely.
#[derive(Debug, Clone)]
pub struct Polygon<D> {
    points: Vec<PlotPoint<D>>,
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
    // =========================================================================
    //  Constructors
    // =========================================================================

    pub const fn new(points: Vec<PlotPoint<D>>) -> Self {
        Self {
            points,
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
        tess: &mut Tessellators,
    ) {
        if self.points.len() < 3 {
            return;
        }

        // 1. Resolve to Screen Coordinates
        // Allocating here is necessary as we need screen-space points for math.
        let screen_points: Vec<Point> = self
            .points
            .iter()
            .map(|p| Point::new(transform.x_to_screen(&p.x), transform.y_to_screen(&p.y)))
            .collect();

        // 2. Resolve Stroke Data (Thickness & Style)
        let maybe_stroke_data = self.resolve_stroke(transform);

        // 3. Determine Geometry Type
        // We check convexity to decide between the "Fast Path" and the "Robust Path".
        let is_convex = self.is_convex(&screen_points);

        // 4. Render Fill
        if let Some(color) = self.fill {
            if is_convex {
                // FAST PATH: Manual Triangle Fan
                // If we have a stroke, we inset the fill slightly to prevent anti-aliasing bleed.
                if maybe_stroke_data.is_some() {
                    let inset_points = self.compute_inset_polygon(&screen_points, 0.5);
                    self.add_triangle_fan(buffer, &inset_points, color);
                } else {
                    self.add_triangle_fan(buffer, &screen_points, color);
                }
            } else {
                // ROBUST PATH: Earcut Algorithm
                // Handles concave shapes, self-intersections, and degeneracies.
                self.tessellate_concave_earcut(buffer, &screen_points, color);
            }
        }

        // 5. Render Stroke
        if let Some((width, stroke)) = maybe_stroke_data {
            match stroke.style {
                StrokeStyle::Solid => {
                    if is_convex {
                        // FAST PATH: Manual Stroke Ring
                        // We calculate the inner edge mathematically and stitch a ring.
                        let inner_points = self.compute_inset_polygon(&screen_points, width);
                        self.add_manual_stroke_ring(
                            buffer,
                            &screen_points,
                            &inner_points,
                            stroke.fill,
                        );
                    } else {
                        // FALLBACK: Lyon
                        // Insetting concave polygons is mathematically perilous.
                        // We delegate to Lyon to handle joins and caps correctly.
                        tess.stroke_polyline(buffer, screen_points, stroke, width, true);
                    }
                }
                StrokeStyle::Dashed | StrokeStyle::Dotted => {
                    // COMPLEX: Always use Lyon for dashes
                    tess.stroke_polyline(buffer, screen_points, stroke, width, true);
                }
            }
        }
    }

    // =========================================================================
    //  Triangulation Implementations
    // =========================================================================

    /// The "Heavy Lifter" for concave polygons.
    /// Uses `earcutr` to triangulate complex shapes efficiently.
    fn tessellate_concave_earcut(&self, buffer: &mut MeshBuffer, points: &[Point], color: Color) {
        // 1. Flatten Data for Earcut
        // Earcut expects a flat interleaved buffer: [x0, y0, x1, y1, ...]
        let flat_coords: Vec<f64> = points
            .iter()
            .flat_map(|p| [p.x as f64, p.y as f64])
            .collect();

        // 2. Run Earcut
        // We pass empty holes (&[]) and dimension 2.
        // We use if-let to safely ignore cases where triangulation fails (e.g., degenerate lines).
        if let Ok(indices) = earcutr::earcut(&flat_coords, &[], 2) {
            let c = pack(color);

            // 3. Map to Iced Mesh
            // Optimization: Create the vertex slice once
            let vertices: Vec<SolidVertex2D> = points
                .iter()
                .map(|p| SolidVertex2D {
                    position: p.to_array(),
                    color: c,
                })
                .collect();

            // Cast indices from usize (Earcut) to u32 (Iced)
            let mesh_indices: Vec<u32> = indices.iter().map(|&i| i as u32).collect();

            buffer.add(&mesh_indices, &vertices);
        }
    }

    /// The "Speed Demon" for convex polygons.
    /// Creates a simple fan of triangles around the first vertex. O(N).
    fn add_triangle_fan(&self, buffer: &mut MeshBuffer, points: &[Point], color: Color) {
        if points.len() < 3 {
            return;
        }

        let c = pack(color);
        let mut vertices = Vec::with_capacity(points.len());
        // We need (N-2) triangles, each having 3 indices
        let mut indices = Vec::with_capacity((points.len() - 2) * 3);

        for p in points {
            vertices.push(SolidVertex2D {
                position: p.to_array(),
                color: c,
            });
        }

        // Fan: Connect Vertex 0 to (i, i+1)
        for i in 1..(points.len() - 1) {
            indices.push(0);
            indices.push(i as u32);
            indices.push((i + 1) as u32);
        }

        buffer.add(&indices, &vertices);
    }

    /// Manually stitches a ring of quads between an outer and inner polygon.
    fn add_manual_stroke_ring(
        &self,
        buffer: &mut MeshBuffer,
        outer: &[Point],
        inner: &[Point],
        color: Color,
    ) {
        if outer.len() != inner.len() || outer.len() < 3 {
            return;
        }

        let c = pack(color);
        let n = outer.len();

        let mut vertices = Vec::with_capacity(n * 2);
        let mut indices = Vec::with_capacity(n * 6);

        // Append all outer vertices then all inner vertices
        // Layout: [O0, O1... On, I0, I1... In]
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

        // Stitch the ring
        for i in 0..n {
            let next = (i + 1) % n;

            // Outer indices
            let o_curr = i as u32;
            let o_next = next as u32;
            // Inner indices (offset by n)
            let i_curr = (i + n) as u32;
            let i_next = (next + n) as u32;

            // Quad formed by two triangles
            // Tri 1
            indices.push(o_curr);
            indices.push(o_next);
            indices.push(i_curr);

            // Tri 2
            indices.push(o_next);
            indices.push(i_next);
            indices.push(i_curr);
        }

        buffer.add(&indices, &vertices);
    }

    // =========================================================================
    //  Math & Helpers
    // =========================================================================

    fn resolve_stroke<'a>(
        &'a self,
        transform: &Transform<D, f32, f32>,
    ) -> Option<(f32, &'a Stroke<D>)> {
        self.stroke.as_ref().and_then(|stroke| {
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
                Some((width, stroke))
            }
        })
    }

    fn is_convex(&self, points: &[Point]) -> bool {
        if points.len() < 4 {
            return true;
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

    fn compute_inset_polygon(&self, points: &[Point], distance: f32) -> Vec<Point> {
        let n = points.len();
        let mut new_points = Vec::with_capacity(n);

        for i in 0..n {
            let prev = points[(i + n - 1) % n];
            let current = points[i];
            let next = points[(i + 1) % n];

            new_points.push(self.compute_inset_vertex(prev, current, next, distance));
        }

        new_points
    }

    fn compute_inset_vertex(
        &self,
        prev: Point,
        current: Point,
        next: Point,
        distance: f32,
    ) -> Point {
        let v1 = (current - prev).normalize();
        let v2 = (next - current).normalize();

        let tangent = (v1 + v2).normalize();
        let miter = Vector::new(-tangent.y, tangent.x);

        let n1 = Vector::new(-v1.y, v1.x);
        let dot = miter.dot(n1);

        let miter_len = if dot.abs() < 1e-4 {
            distance
        } else {
            distance / dot
        };

        let limited_len = miter_len.min(distance * 3.0);
        current + miter * limited_len
    }
}
