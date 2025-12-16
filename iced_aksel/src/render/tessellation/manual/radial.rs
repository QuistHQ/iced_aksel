//! Radial primitives (Arcs and Sectors).
//!
//! Handles curved shapes defined by angular ranges.

use crate::render::MeshBuffer;
use iced_core::Color;
use iced_graphics::{color::pack, mesh::SolidVertex2D};

/// Writes a strip of triangles representing an arc or donut sector.
///
/// The strip allows variable thickness (inner radius vs outer radius).
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn draw_arc_strip(
    buffer: &mut MeshBuffer,
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
        let theta = start_angle + (i as f32 * step_angle);
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
    for i in 0..segments {
        let current = i as u32;
        let next = current + 1;

        let inner_curr = start_index + current * 2;
        let outer_curr = start_index + current * 2 + 1;
        let inner_next = start_index + next * 2;
        let outer_next = start_index + next * 2 + 1;

        mesh.indices.extend_from_slice(&[
            inner_curr, outer_curr, outer_next, inner_curr, outer_next, inner_next,
        ]);
    }
}
