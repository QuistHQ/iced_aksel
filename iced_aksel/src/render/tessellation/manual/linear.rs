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

/// Draws a dashed line between two points.
///
/// # Arguments
/// * `buffer` - The mesh buffer to draw into
/// * `start` - Starting point of the line
/// * `end` - Ending point of the line
/// * `width` - Thickness of the line in pixels
/// * `color` - Color of the line
/// * `dash_length` - Length of each dash segment in pixels
/// * `gap_length` - Length of gaps between dashes in pixels
pub fn draw_dashed_line(
    buffer: &mut MeshBuffer,
    start: Point,
    end: Point,
    width: f32,
    color: Color,
    dash_length: f32,
    gap_length: f32,
) {
    // Guard against invalid parameters that could cause infinite loop
    if dash_length <= 0.0 || gap_length < 0.0 {
        // Fall back to solid line if dash parameters are invalid
        draw_line_segment(buffer, start, end, width, color);
        return;
    }

    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length = (dx * dx + dy * dy).sqrt();

    // Prevent division by zero
    if length < 0.0001 {
        return;
    }

    let dir_x = dx / length;
    let dir_y = dy / length;

    let mut current_dist = 0.0;

    while current_dist < length {
        let segment_end_dist = (current_dist + dash_length).min(length);

        let seg_start = Point::new(
            start.x + dir_x * current_dist,
            start.y + dir_y * current_dist,
        );
        let seg_end = Point::new(
            start.x + dir_x * segment_end_dist,
            start.y + dir_y * segment_end_dist,
        );

        draw_line_segment(buffer, seg_start, seg_end, width, color);

        current_dist += dash_length + gap_length;
    }
}