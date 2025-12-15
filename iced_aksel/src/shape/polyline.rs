use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellators},
};

use aksel::{Float, PlotPoint, Transform};
use iced_core::Color;
use iced_graphics::{color::pack, mesh::SolidVertex2D};
use lyon::math::{Point, Vector};

/// A connected series of line segments.
///
/// Supports infinite extension on the first/last segments, optional arrowheads,
/// and proper miter joins via Lyon tessellation.
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
    // =========================================================================
    //  Constructors
    // =========================================================================

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

    // =========================================================================
    //  Builder Methods
    // =========================================================================

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

    // =========================================================================
    //  Tessellation Logic
    // =========================================================================

    fn tessellate(
        self,
        transform: &Transform<D, f32, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellators,
    ) {
        if self.points.len() < 2 {
            return;
        }

        // 1. Resolve Stroke Thickness
        let width = self.resolve_thickness(transform);
        if width < 0.1 {
            return;
        }

        // 2. Branch: Fast vs Complex
        if self.is_simple_line() {
            self.tessellate_fast(transform, buffer, tess, width);
        } else {
            self.tessellate_complex(transform, buffer, tess, width);
        }
    }

    // --- Path A: The Fast Path (Zero Allocation) ---
    // Used for standard data series (99% of cases).
    // Streams points directly from the plot data to the tessellator.
    fn tessellate_fast(
        &self,
        transform: &Transform<D, f32, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellators,
        width: f32,
    ) {
        // Creates the iterator, does NOT collect into Vec
        let iterator = self.project_points(transform);

        tess.stroke_polyline(
            buffer,
            iterator,
            &self.stroke,
            width,
            false, // Open path
        );
    }

    // --- Path B: The Complex Path (Modifies Geometry) ---
    // Used for arrows and infinite extensions.
    // Must allocate a Vec because we need random access to modify start/end points.
    fn tessellate_complex(
        self,
        transform: &Transform<D, f32, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellators,
        width: f32,
    ) {
        // 1. Allocate and Collect (Necessary for modification)
        let mut screen_points: Vec<Point> = self.project_points(transform).collect();

        // 2. Prepare Clipping for Extensions
        let bounds = transform.screen_bounds();
        let clip_margin = width * self.arrow_size.max(2.0);
        let clip_rect = (
            bounds.x - clip_margin,
            bounds.y - clip_margin,
            bounds.x + bounds.width + clip_margin,
            bounds.y + bounds.height + clip_margin,
        );

        let last_idx = screen_points.len() - 1;
        let p0 = screen_points[0];
        let pn = screen_points[last_idx];

        // 3. Modify Start (Extension or Arrow Retraction)
        if self.extend_start {
            let p1 = screen_points[1];
            if let Some((t0, _)) = clip_line(p0, p1, clip_rect) {
                if t0 < 0.0 {
                    screen_points[0] = p0 + (p1 - p0) * t0;
                }
            }
        } else if self.arrow_start {
            let p1 = screen_points[1];
            let dir = (p1 - p0).normalize();
            screen_points[0] = p0 + dir * (width * self.arrow_size);
        }

        // 4. Modify End (Extension or Arrow Retraction)
        if self.extend_end {
            let pn_minus_1 = screen_points[last_idx - 1];
            if let Some((_, t1)) = clip_line(pn_minus_1, pn, clip_rect) {
                if t1 > 1.0 {
                    screen_points[last_idx] = pn_minus_1 + (pn - pn_minus_1) * t1;
                }
            }
        } else if self.arrow_end {
            let pn_minus_1 = screen_points[last_idx - 1];
            let dir = (pn - pn_minus_1).normalize();
            screen_points[last_idx] = pn - dir * (width * self.arrow_size);
        }

        // 5. Render the Body
        tess.stroke_polyline(
            buffer,
            screen_points, // Pass the modified Vec
            &self.stroke,
            width,
            false,
        );

        // 6. Render Arrowheads (Manual Manual)
        // We draw these at the *original* p0/pn coordinates
        if self.arrow_start && !self.extend_start {
            let p1 = self.project_single(transform, 1); // Helper to get just one point
            let dir = (p1 - p0).normalize();
            self.add_arrowhead(buffer, p0, -dir, width, self.stroke.fill);
        }

        if self.arrow_end && !self.extend_end {
            let pn_minus_1 = self.project_single(transform, self.points.len() - 2);
            let dir = (pn - pn_minus_1).normalize();
            self.add_arrowhead(buffer, pn, dir, width, self.stroke.fill);
        }
    }

    // =========================================================================
    //  Helpers
    // =========================================================================

    /// Returns true if we can use the optimized zero-allocation path.
    #[inline(always)]
    fn is_simple_line(&self) -> bool {
        !self.arrow_start && !self.arrow_end && !self.extend_start && !self.extend_end
    }

    /// Resolves the stroke width in screen pixels.
    fn resolve_thickness(&self, transform: &Transform<D, f32, f32>) -> f32 {
        match self.stroke.thickness {
            Measure::Screen(w) => w,
            Measure::Plot(w) => {
                let p0 = transform.x_to_screen(&D::zero());
                let p1 = transform.x_to_screen(&w);
                (p1 - p0).abs()
            }
        }
    }

    /// Creates a lazy iterator that projects points from Plot Space to Screen Space.
    /// This avoids allocating a `Vec<Point>` unless absolutely necessary.
    fn project_points<'a>(
        &'a self,
        transform: &'a Transform<D, f32, f32>,
    ) -> impl Iterator<Item = Point> + 'a {
        self.points
            .iter()
            .map(move |p| Point::new(transform.x_to_screen(&p.x), transform.y_to_screen(&p.y)))
    }

    /// Helper to get a specific projected point without creating a full iterator.
    fn project_single(&self, transform: &Transform<D, f32, f32>, index: usize) -> Point {
        let p = &self.points[index];
        Point::new(transform.x_to_screen(&p.x), transform.y_to_screen(&p.y))
    }

    fn add_arrowhead(
        &self,
        buffer: &mut MeshBuffer,
        tip: Point,
        direction: Vector,
        width: f32,
        color: Color,
    ) {
        let c = pack(color);
        let arrow_len = width * self.arrow_size;
        let arrow_width = width * self.arrow_size * 0.8;

        let base_center = tip - direction * arrow_len;
        let normal = Vector::new(-direction.y, direction.x) * (arrow_width / 2.0);
        let wing1 = base_center + normal;
        let wing2 = base_center - normal;

        // Simple Triangle
        buffer.add(
            &[0, 1, 2],
            &[
                SolidVertex2D {
                    position: tip.to_array(),
                    color: c,
                },
                SolidVertex2D {
                    position: wing1.to_array(),
                    color: c,
                },
                SolidVertex2D {
                    position: wing2.to_array(),
                    color: c,
                },
            ],
        );
    }
}

// =========================================================================
//  Math Utilities
// =========================================================================

fn clip_line(p1: Point, p2: Point, clip_rect: (f32, f32, f32, f32)) -> Option<(f32, f32)> {
    let (xmin, ymin, xmax, ymax) = clip_rect;
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;

    let mut t0 = -100_000.0;
    let mut t1 = 100_000.0;

    let p = [-dx, dx, -dy, dy];
    let q = [p1.x - xmin, xmax - p1.x, p1.y - ymin, ymax - p1.y];

    for i in 0..4 {
        if p[i].abs() < 1e-6 {
            if q[i] < 0.0 {
                return None;
            }
        } else {
            let t = q[i] / p[i];
            if p[i] < 0.0 {
                if t > t1 {
                    return None;
                }
                if t > t0 {
                    t0 = t;
                }
            } else {
                if t < t0 {
                    return None;
                }
                if t < t1 {
                    t1 = t;
                }
            }
        }
    }

    if t0 <= t1 { Some((t0, t1)) } else { None }
}
