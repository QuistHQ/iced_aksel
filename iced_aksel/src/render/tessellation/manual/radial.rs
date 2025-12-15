use crate::render::MeshBuffer;
use iced_core::Color;
use iced_graphics::{color::pack, mesh::SolidVertex2D};

#[inline]
#[allow(clippy::too_many_arguments)]
pub fn draw_arc_strip(
    buffer: &mut MeshBuffer,
    cx: f32,
    cy: f32,
    r_inner: f32,
    r_outer: f32,
    start_angle: f32,
    end_angle: f32,
    color: Color,
    segments: usize,
) {
    let c = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start = mesh.vertices.len() as u32;

    let sweep = (end_angle - start_angle).abs();
    let step = sweep / segments as f32;
    let dir = if end_angle > start_angle { 1.0 } else { -1.0 };

    for i in 0..=segments {
        let theta = (i as f32 * step).mul_add(dir, start_angle);
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
        let base = start + (i * 2) as u32;
        mesh.indices
            .extend_from_slice(&[base, base + 1, base + 2, base + 1, base + 3, base + 2]);
    }
}
