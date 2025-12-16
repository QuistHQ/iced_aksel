//! Linear geometric primitives (Line Segments and Arrows).
//!
//! Handles expanding thin mathematical lines into thick, renderable triangles.

use crate::render::MeshBuffer;
use iced_core::{Color, Point, Vector};
use iced_graphics::{color::pack, mesh::SolidVertex2D};

/// Draws a thick line segment between two points.
///
/// Calculates the normal vector perpendicular to the line direction to expand
/// the line width into a rectangle (2 triangles).
#[inline]
pub fn draw_line_segment(
    buffer: &mut MeshBuffer,
    start: Point,
    end: Point,
    width: f32,
    color: Color,
) {
    let packed_color = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;

    // Vector from start to end
    let dx = end.x - start.x;
    let dy = end.y - start.y;

    // Compute length for normalization
    let length_sq = dx * dx + dy * dy;
    if length_sq < 0.0001 {
        return;
    } // Prevent division by zero
    let inverse_length = 1.0 / length_sq.sqrt();

    // Perpendicular normal vector scaled by half-width
    // Normal of (dx, dy) is (-dy, dx)
    let offset_x = -dy * inverse_length * (width / 2.0);
    let offset_y = dx * inverse_length * (width / 2.0);

    mesh.vertices.extend_from_slice(&[
        // Start Left
        SolidVertex2D {
            position: [start.x + offset_x, start.y + offset_y],
            color: packed_color,
        },
        // Start Right
        SolidVertex2D {
            position: [start.x - offset_x, start.y - offset_y],
            color: packed_color,
        },
        // End Left
        SolidVertex2D {
            position: [end.x + offset_x, end.y + offset_y],
            color: packed_color,
        },
        // End Right
        SolidVertex2D {
            position: [end.x - offset_x, end.y - offset_y],
            color: packed_color,
        },
    ]);

    // Draw as two triangles (Strip order)
    mesh.indices.extend_from_slice(&[
        start_index,
        start_index + 1,
        start_index + 2,
        start_index + 1,
        start_index + 3,
        start_index + 2,
    ]);
}

/// Draws a triangular arrowhead at a specific point facing a direction.
///
/// The arrowhead is constructed as a simple isosceles triangle.
#[inline]
pub fn draw_arrowhead(
    buffer: &mut MeshBuffer,
    tip: Point,
    direction: Vector, // Must be normalized Vector
    line_width: f32,
    arrow_size_multiplier: f32,
    color: Color,
) {
    let packed_color = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;

    let arrow_length = line_width * arrow_size_multiplier;
    let arrow_half_width = line_width * (arrow_size_multiplier * 0.4);

    // Calculate the base of the arrow
    // Point - Vector = Point
    let base_center = tip - direction * arrow_length;

    // Perpendicular vector for width
    let perp_x = -direction.y * arrow_half_width;
    let perp_y = direction.x * arrow_half_width;

    mesh.vertices.extend_from_slice(&[
        // Tip
        SolidVertex2D {
            position: [tip.x, tip.y],
            color: packed_color,
        },
        // Left Base
        SolidVertex2D {
            position: [base_center.x + perp_x, base_center.y + perp_y],
            color: packed_color,
        },
        // Right Base
        SolidVertex2D {
            position: [base_center.x - perp_x, base_center.y - perp_y],
            color: packed_color,
        },
    ]);

    mesh.indices
        .extend_from_slice(&[start_index, start_index + 1, start_index + 2]);
}
