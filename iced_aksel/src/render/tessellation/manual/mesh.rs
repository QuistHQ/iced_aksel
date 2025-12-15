use crate::render::MeshBuffer;
use iced_core::{Color, Point};
use iced_graphics::{color::pack, mesh::SolidVertex2D};

#[inline]
pub fn draw_mesh(buffer: &mut MeshBuffer, vertices: &[Point], indices: &[u32], color: Color) {
    let c = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start_offset = mesh.vertices.len() as u32;

    for p in vertices {
        mesh.vertices.push(SolidVertex2D {
            position: [p.x, p.y],
            color: c,
        });
    }

    for i in indices {
        mesh.indices.push(start_offset + i);
    }
}
