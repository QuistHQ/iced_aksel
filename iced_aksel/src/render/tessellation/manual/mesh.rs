//! Raw mesh handling.
//!
//! Allows direct writing of pre-calculated vertex and index buffers (e.g., from Earcut).

use crate::render::MeshBuffer;
use iced_core::{Color, Point};
use iced_graphics::{color::pack, mesh::SolidVertex2D};

/// Writes a pre-triangulated mesh to the buffer.
///
/// * `vertices`: List of point coordinates.
/// * `indices`: List of indices into the vertex list (triplets).
#[inline]
pub fn draw_raw_mesh(buffer: &mut MeshBuffer, vertices: &[Point], indices: &[u32], color: Color) {
    let packed_color = pack(color);
    let mesh = buffer.get_mesh_mut();
    let index_offset = mesh.vertices.len() as u32;

    // Bulk write vertices
    mesh.vertices.reserve(vertices.len());
    for p in vertices {
        mesh.vertices.push(SolidVertex2D {
            position: [p.x, p.y],
            color: packed_color,
        });
    }

    // Bulk write indices, applying the offset
    mesh.indices.reserve(indices.len());
    for &i in indices {
        mesh.indices.push(i + index_offset);
    }
}
