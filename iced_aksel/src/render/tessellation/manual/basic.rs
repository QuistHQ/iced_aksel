use crate::render::MeshBuffer;
use iced_core::Color;
use iced_graphics::{color::pack, mesh::SolidVertex2D};

#[inline]
pub fn draw_fill_rect(
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
    ]);

    #[rustfmt::skip]
    mesh.indices.extend_from_slice(&[
        start, start + 1, start + 2,
        start, start + 2, start + 3,
    ]);
}

#[inline]
#[allow(clippy::too_many_arguments)]
pub fn draw_stroke_rect(
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
        start, start + 1, start + 4,   start + 1, start + 4, start + 5,
        start + 1, start + 2, start + 5,   start + 2, start + 5, start + 6,
        start + 2, start + 3, start + 6,   start + 3, start + 6, start + 7,
        start + 3, start, start + 7,   start, start + 7, start + 4,
    ]);
}

#[inline]
pub fn draw_fill_circle(
    buffer: &mut MeshBuffer,
    cx: f32,
    cy: f32,
    radius: f32,
    color: Color,
    segments: usize,
) {
    let c = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start = mesh.vertices.len() as u32;
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

#[inline]
pub fn draw_stroke_circle(
    buffer: &mut MeshBuffer,
    cx: f32,
    cy: f32,
    r_inner: f32,
    r_outer: f32,
    color: Color,
    segments: usize,
) {
    let c = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start = mesh.vertices.len() as u32;
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
