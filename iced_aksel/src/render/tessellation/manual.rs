use crate::render::MeshBuffer;
use iced_core::{Color, Point};
use iced_graphics::{color::pack, mesh::SolidVertex2D};

/// The "Fast Path" tessellator.
///
/// This component generates vertices directly for simple primitives (Quads, Circles, Triangles)
/// without the overhead of path building or generic tessellation algorithms.
#[derive(Default)]
pub struct ManualTessellator;

impl ManualTessellator {
    // =========================================================================
    //  Rectangle Primitives
    // =========================================================================

    /// Draws a solid rectangle (2 triangles).
    #[inline]
    pub fn draw_fill_rect(
        &mut self,
        buffer: &mut MeshBuffer,
        x_min: f32,
        y_min: f32,
        x_max: f32,
        y_max: f32,
        color: Color,
    ) {
        let c = pack(color);
        let mesh = buffer.get_mesh_mut();
        let start = mesh.vertices.len() as u32;

        mesh.vertices.extend_from_slice(&[
            SolidVertex2D {
                position: [x_min, y_min],
                color: c,
            }, // BL
            SolidVertex2D {
                position: [x_max, y_min],
                color: c,
            }, // BR
            SolidVertex2D {
                position: [x_max, y_max],
                color: c,
            }, // TR
            SolidVertex2D {
                position: [x_min, y_max],
                color: c,
            }, // TL
        ]);

        #[rustfmt::skip]
        mesh.indices.extend_from_slice(&[
            start, start + 1, start + 2,
            start, start + 2, start + 3,
        ]);
    }

    /// Draws a hollow rectangular frame (8 vertices).
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn draw_stroke_rect(
        &mut self,
        buffer: &mut MeshBuffer,
        x_min: f32,
        y_min: f32,
        x_max: f32,
        y_max: f32,
        th_x: f32,
        th_y: f32,
        color: Color,
    ) {
        let c = pack(color);
        let mesh = buffer.get_mesh_mut();
        let start = mesh.vertices.len() as u32;

        let ix_min = x_min + th_x;
        let ix_max = x_max - th_x;
        let iy_min = y_min + th_y;
        let iy_max = y_max - th_y;

        mesh.vertices.extend_from_slice(&[
            // Outer Ring (0-3)
            SolidVertex2D {
                position: [x_min, y_min],
                color: c,
            },
            SolidVertex2D {
                position: [x_max, y_min],
                color: c,
            },
            SolidVertex2D {
                position: [x_max, y_max],
                color: c,
            },
            SolidVertex2D {
                position: [x_min, y_max],
                color: c,
            },
            // Inner Ring (4-7)
            SolidVertex2D {
                position: [ix_min, iy_min],
                color: c,
            },
            SolidVertex2D {
                position: [ix_max, iy_min],
                color: c,
            },
            SolidVertex2D {
                position: [ix_max, iy_max],
                color: c,
            },
            SolidVertex2D {
                position: [ix_min, iy_max],
                color: c,
            },
        ]);

        #[rustfmt::skip]
        mesh.indices.extend_from_slice(&[
            start, start + 1, start + 4,   start + 1, start + 4, start + 5, // Bottom
            start + 1, start + 2, start + 5,   start + 2, start + 5, start + 6, // Right
            start + 2, start + 3, start + 6,   start + 3, start + 6, start + 7, // Top
            start + 3, start, start + 7,   start, start + 7, start + 4, // Left
        ]);
    }

    // =========================================================================
    //  Circle Primitives
    // =========================================================================

    /// Draws a solid circle using a Triangle Fan.
    #[inline]
    pub fn draw_fill_circle(
        &mut self,
        buffer: &mut MeshBuffer,
        cx: f32,
        cy: f32,
        radius: f32,
        color: Color,
    ) {
        let c = pack(color);
        let mesh = buffer.get_mesh_mut();
        let start = mesh.vertices.len() as u32;

        let segments = (radius * 2.0).clamp(24.0, 128.0) as usize;
        let step = std::f32::consts::TAU / segments as f32;

        mesh.vertices.push(SolidVertex2D {
            position: [cx, cy],
            color: c,
        });

        for i in 0..segments {
            let theta = i as f32 * step;
            let (sin, cos) = theta.sin_cos();
            mesh.vertices.push(SolidVertex2D {
                position: [cos.mul_add(radius, cx), sin.mul_add(radius, cy)],
                color: c,
            });
        }

        for i in 0..segments {
            let current = (i + 1) as u32;
            let next = if i == segments - 1 { 1 } else { current + 1 };
            mesh.indices
                .extend_from_slice(&[start, start + current, start + next]);
        }
    }

    /// Draws a solid ring using a Triangle Strip.
    #[inline]
    pub fn draw_stroke_circle(
        &mut self,
        buffer: &mut MeshBuffer,
        cx: f32,
        cy: f32,
        r_inner: f32,
        r_outer: f32,
        color: Color,
    ) {
        let c = pack(color);
        let mesh = buffer.get_mesh_mut();
        let start = mesh.vertices.len() as u32;

        let segments = (r_outer * 2.0).clamp(24.0, 128.0) as usize;
        let step = std::f32::consts::TAU / segments as f32;

        for i in 0..segments {
            let theta = i as f32 * step;
            let (sin, cos) = theta.sin_cos();
            mesh.vertices.push(SolidVertex2D {
                position: [cos.mul_add(r_inner, cx), sin.mul_add(r_inner, cy)],
                color: c,
            });
            mesh.vertices.push(SolidVertex2D {
                position: [cos.mul_add(r_outer, cx), sin.mul_add(r_outer, cy)],
                color: c,
            });
        }

        for i in 0..segments {
            let i = i as u32;
            let next_i = (i + 1) % segments as u32;
            let inner_curr = start + i * 2;
            let outer_curr = start + i * 2 + 1;
            let inner_next = start + next_i * 2;
            let outer_next = start + next_i * 2 + 1;

            mesh.indices.extend_from_slice(&[
                inner_curr, outer_curr, outer_next, inner_curr, outer_next, inner_next,
            ]);
        }
    }

    // =========================================================================
    //  Triangle & Polygon Primitives
    // =========================================================================

    /// Draws a solid triangle.
    #[inline]
    pub fn draw_fill_triangle(
        &mut self,
        buffer: &mut MeshBuffer,
        p1: Point,
        p2: Point,
        p3: Point,
        color: Color,
    ) {
        let c = pack(color);
        let mesh = buffer.get_mesh_mut();
        let start = mesh.vertices.len() as u32;

        mesh.vertices.extend_from_slice(&[
            SolidVertex2D {
                position: [p1.x, p1.y],
                color: c,
            },
            SolidVertex2D {
                position: [p2.x, p2.y],
                color: c,
            },
            SolidVertex2D {
                position: [p3.x, p3.y],
                color: c,
            },
        ]);

        mesh.indices
            .extend_from_slice(&[start, start + 1, start + 2]);
    }

    /// Draws a stroked triangle (frame) given outer and inner vertices.
    #[inline]
    pub fn draw_stroke_triangle(
        &mut self,
        buffer: &mut MeshBuffer,
        outer: [Point; 3],
        inner: [Point; 3],
        color: Color,
    ) {
        let c = pack(color);
        let mesh = buffer.get_mesh_mut();
        let start = mesh.vertices.len() as u32;

        mesh.vertices.extend_from_slice(&[
            // Outer 0, 1, 2
            SolidVertex2D {
                position: [outer[0].x, outer[0].y],
                color: c,
            },
            SolidVertex2D {
                position: [outer[1].x, outer[1].y],
                color: c,
            },
            SolidVertex2D {
                position: [outer[2].x, outer[2].y],
                color: c,
            },
            // Inner 3, 4, 5
            SolidVertex2D {
                position: [inner[0].x, inner[0].y],
                color: c,
            },
            SolidVertex2D {
                position: [inner[1].x, inner[1].y],
                color: c,
            },
            SolidVertex2D {
                position: [inner[2].x, inner[2].y],
                color: c,
            },
        ]);

        // Stitch 3 trapezoids (quads)
        #[rustfmt::skip]
        mesh.indices.extend_from_slice(&[
            start, start + 1, start + 4,   start, start + 4, start + 3, // Side 1
            start + 1, start + 2, start + 5,   start + 1, start + 5, start + 4, // Side 2
            start + 2, start, start + 3,   start + 2, start + 3, start + 5, // Side 3
        ]);
    }

    /// Draws a generic fan (for polygons).
    #[inline]
    pub fn draw_fan(&mut self, buffer: &mut MeshBuffer, points: &[Point], color: Color) {
        if points.len() < 3 {
            return;
        }
        let c = pack(color);
        let mesh = buffer.get_mesh_mut();
        let start = mesh.vertices.len() as u32;

        for p in points {
            mesh.vertices.push(SolidVertex2D {
                position: [p.x, p.y],
                color: c,
            });
        }

        // Fan indices: 0 -> i -> i+1
        for i in 1..(points.len() - 1) {
            mesh.indices
                .extend_from_slice(&[start, start + i as u32, start + (i + 1) as u32]);
        }
    }

    /// Draws a generic ring (for polygon borders).
    #[inline]
    pub fn draw_ring(
        &mut self,
        buffer: &mut MeshBuffer,
        outer: &[Point],
        inner: &[Point],
        color: Color,
    ) {
        if outer.len() != inner.len() || outer.len() < 3 {
            return;
        }
        let c = pack(color);
        let mesh = buffer.get_mesh_mut();
        let start = mesh.vertices.len() as u32;
        let n = outer.len();

        for p in outer {
            mesh.vertices.push(SolidVertex2D {
                position: [p.x, p.y],
                color: c,
            });
        }
        for p in inner {
            mesh.vertices.push(SolidVertex2D {
                position: [p.x, p.y],
                color: c,
            });
        }

        for i in 0..n {
            let next = (i + 1) % n;
            // Vertex layout: [Outer... | Inner...]
            let o_curr = start + i as u32;
            let o_next = start + next as u32;
            let i_curr = start + (i + n) as u32;
            let i_next = start + (next + n) as u32;

            mesh.indices
                .extend_from_slice(&[o_curr, o_next, i_curr, o_next, i_next, i_curr]);
        }
    }
}
