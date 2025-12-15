use crate::render::MeshBuffer;
use iced_core::{Color, Point};
use iced_graphics::{color::pack, mesh::SolidVertex2D};

#[inline]
pub fn draw_fill_triangle(buffer: &mut MeshBuffer, p1: Point, p2: Point, p3: Point, color: Color) {
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

#[inline]
pub fn draw_stroke_triangle(
    buffer: &mut MeshBuffer,
    outer: [Point; 3],
    inner: [Point; 3],
    color: Color,
) {
    let c = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start = mesh.vertices.len() as u32;

    mesh.vertices.extend_from_slice(&[
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

    #[rustfmt::skip]
    mesh.indices.extend_from_slice(&[
        start, start + 1, start + 4,   start, start + 4, start + 3,
        start + 1, start + 2, start + 5,   start + 1, start + 5, start + 4,
        start + 2, start, start + 3,   start + 2, start + 3, start + 5,
    ]);
}

#[inline]
pub fn draw_fan(buffer: &mut MeshBuffer, points: &[Point], color: Color) {
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

    for i in 1..(points.len() - 1) {
        mesh.indices
            .extend_from_slice(&[start, start + i as u32, start + (i + 1) as u32]);
    }
}

#[inline]
pub fn draw_ring(buffer: &mut MeshBuffer, outer: &[Point], inner: &[Point], color: Color) {
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
        let o_curr = start + i as u32;
        let o_next = start + next as u32;
        let i_curr = start + (i + n) as u32;
        let i_next = start + (next + n) as u32;

        mesh.indices
            .extend_from_slice(&[o_curr, o_next, i_curr, o_next, i_next, i_curr]);
    }
}
