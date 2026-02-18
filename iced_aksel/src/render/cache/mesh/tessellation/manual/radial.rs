//! Radial primitives (Arcs and Sectors).
//!
//! Handles curved shapes defined by angular ranges.

use crate::render::cache::MeshData;
use iced_core::Color;
use iced_graphics::{color::pack, mesh::SolidVertex2D};

/// Writes a strip of triangles representing an arc or donut sector.
///
/// The strip allows variable thickness (inner radius vs outer radius).
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn draw_arc_strip(
    buffer: &mut MeshData,
    center_x: f32,
    center_y: f32,
    radius_inner: f32,
    radius_outer: f32,
    start_angle: f32,
    end_angle: f32,
    color: Color,
    segments: usize,
) {
    let packed_color = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;

    let total_sweep = end_angle - start_angle;
    let step_angle = total_sweep / segments as f32;

    for i in 0..=segments {
        let theta = (i as f32).mul_add(step_angle, start_angle);
        let (sin, cos) = theta.sin_cos();

        // Inner Vertex
        mesh.vertices.push(SolidVertex2D {
            position: [
                cos.mul_add(radius_inner, center_x),
                sin.mul_add(radius_inner, center_y),
            ],
            color: packed_color,
        });

        // Outer Vertex
        mesh.vertices.push(SolidVertex2D {
            position: [
                cos.mul_add(radius_outer, center_x),
                sin.mul_add(radius_outer, center_y),
            ],
            color: packed_color,
        });
    }

    // Connect as Triangle Strip
    super::push_ring_strip_indices(mesh, start_index, segments, false);
}
