//! Linear geometric primitives (Line Segments and Arrows).
//!
//! Handles expanding thin mathematical lines into thick, renderable triangles.

use crate::render::cache::MeshData;
use crate::render::cache::mesh::tessellation::manual::basic::draw_dot;
use iced_core::{Color, Point, Vector};
use iced_graphics::{color::pack, mesh::SolidVertex2D};

/// Helper to optionally snap a coordinate to the nearest pixel grid.
///
/// If `snap` is true:
/// - Odd widths (1.0, 3.0) snap to `x.5` (pixel center).
/// - Even widths (2.0, 4.0) snap to `x.0` (pixel boundary).
#[inline(always)]
fn maybe_snap(coord: f32, width: f32, snap: bool) -> f32 {
    if !snap {
        return coord;
    }

    // Check if width is roughly an odd integer
    let width_int = width.round();
    let is_odd = (width_int as i32) % 2 != 0;

    if is_odd {
        (coord - 0.5).round() + 0.5
    } else {
        coord.round()
    }
}

/// Draws a thick line segment between two points.
///
/// Calculates the normal vector perpendicular to the line direction to expand
/// the line width into a rectangle (2 triangles).
#[inline]
pub fn draw_line_segment(
    buffer: &mut MeshData,
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
    let length_sq = dx.mul_add(dx, dy * dy);
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
    buffer: &mut MeshData,
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

/// Draws a generic dashed line between two points.
#[allow(unused)]
pub fn draw_dashed_line_segment(
    buffer: &mut MeshData,
    start: Point,
    end: Point,
    width: f32,
    color: Color,
    dash_length: f32,
    gap_length: f32,
) {
    if dash_length <= 0.0 || gap_length < 0.0 {
        draw_line_segment(buffer, start, end, width, color);
        return;
    }

    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length = dx.hypot(dy);

    if length < 0.0001 {
        return;
    }

    let dir_x = dx / length;
    let dir_y = dy / length;

    let mut current_dist = 0.0;

    #[allow(clippy::while_float)]
    while current_dist < length {
        let segment_end_dist = (current_dist + dash_length).min(length);

        let seg_start = Point::new(
            dir_x.mul_add(current_dist, start.x),
            dir_y.mul_add(current_dist, start.y),
        );
        let seg_end = Point::new(
            dir_x.mul_add(segment_end_dist, start.x),
            dir_y.mul_add(segment_end_dist, start.y),
        );

        draw_line_segment(buffer, seg_start, seg_end, width, color);

        current_dist += dash_length + gap_length;
    }
}

// =============================================================================
// OPTIMIZED AXIS PRIMITIVES
// =============================================================================

/// Draws a perfectly horizontal line.
#[inline]
pub fn draw_horizontal_line(
    buffer: &mut MeshData,
    x_start: f32,
    x_end: f32,
    y: f32,
    width: f32,
    color: Color,
    snap: bool,
) {
    let packed_color = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;

    let y_snapped = maybe_snap(y, width, snap);
    // Note: We generally don't snap start/end X coordinates for length,
    // but if strict grid alignment is desired, they could be snapped too.
    // For now, we only snap the axis position (Y) to ensure sharpness.

    let half_width = width / 2.0;
    let top = y_snapped - half_width;
    let bottom = y_snapped + half_width;

    mesh.vertices.extend_from_slice(&[
        SolidVertex2D {
            position: [x_start, top],
            color: packed_color,
        }, // Top-Left
        SolidVertex2D {
            position: [x_start, bottom],
            color: packed_color,
        }, // Bottom-Left
        SolidVertex2D {
            position: [x_end, top],
            color: packed_color,
        }, // Top-Right
        SolidVertex2D {
            position: [x_end, bottom],
            color: packed_color,
        }, // Bottom-Right
    ]);

    mesh.indices.extend_from_slice(&[
        start_index,
        start_index + 1,
        start_index + 2,
        start_index + 1,
        start_index + 3,
        start_index + 2,
    ]);
}

/// Draws a perfectly vertical line.
#[inline]
pub fn draw_vertical_line(
    buffer: &mut MeshData,
    x: f32,
    y_start: f32,
    y_end: f32,
    width: f32,
    color: Color,
    snap: bool,
) {
    let packed_color = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;

    let x_snapped = maybe_snap(x, width, snap);

    let half_width = width / 2.0;
    let left = x_snapped - half_width;
    let right = x_snapped + half_width;

    mesh.vertices.extend_from_slice(&[
        SolidVertex2D {
            position: [left, y_start],
            color: packed_color,
        }, // Top-Left
        SolidVertex2D {
            position: [right, y_start],
            color: packed_color,
        }, // Top-Right
        SolidVertex2D {
            position: [left, y_end],
            color: packed_color,
        }, // Bottom-Left
        SolidVertex2D {
            position: [right, y_end],
            color: packed_color,
        }, // Bottom-Right
    ]);

    mesh.indices.extend_from_slice(&[
        start_index,
        start_index + 1,
        start_index + 2,
        start_index + 1,
        start_index + 3,
        start_index + 2,
    ]);
}

/// Draws a perfectly horizontal dashed line without vector math.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn draw_horizontal_dashed_line(
    buffer: &mut MeshData,
    x_start: f32,
    x_end: f32,
    y: f32,
    width: f32,
    color: Color,
    dash_length: f32,
    gap_length: f32,
    snap: bool,
) {
    if dash_length <= 0.0 || gap_length < 0.0 {
        draw_horizontal_line(buffer, x_start, x_end, y, width, color, snap);
        return;
    }

    // Ensure we draw left-to-right to simplify math
    let (start, end) = if x_start < x_end {
        (x_start, x_end)
    } else {
        (x_end, x_start)
    };

    let mut current_x = start;

    // Pre-calculate Y geometry to avoid doing it per dash
    let y_snapped = maybe_snap(y, width, snap);
    let half_width = width / 2.0;
    let top = y_snapped - half_width;
    let bottom = y_snapped + half_width;

    let packed_color = pack(color);

    // We can manually push vertices here for maximum speed,
    // rather than calling draw_horizontal_line repeatedly.
    #[allow(clippy::while_float)]
    while current_x < end {
        let seg_end_x = (current_x + dash_length).min(end);

        // Inline the mesh writing for performance
        let mesh = buffer.get_mesh_mut();
        let start_index = mesh.vertices.len() as u32;

        mesh.vertices.extend_from_slice(&[
            SolidVertex2D {
                position: [current_x, top],
                color: packed_color,
            },
            SolidVertex2D {
                position: [current_x, bottom],
                color: packed_color,
            },
            SolidVertex2D {
                position: [seg_end_x, top],
                color: packed_color,
            },
            SolidVertex2D {
                position: [seg_end_x, bottom],
                color: packed_color,
            },
        ]);

        mesh.indices.extend_from_slice(&[
            start_index,
            start_index + 1,
            start_index + 2,
            start_index + 1,
            start_index + 3,
            start_index + 2,
        ]);

        current_x += dash_length + gap_length;
    }
}

/// Draws a perfectly vertical dashed line without vector math.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn draw_vertical_dashed_line(
    buffer: &mut MeshData,
    x: f32,
    y_start: f32,
    y_end: f32,
    width: f32,
    color: Color,
    dash_length: f32,
    gap_length: f32,
    snap: bool,
) {
    if dash_length <= 0.0 || gap_length < 0.0 {
        draw_vertical_line(buffer, x, y_start, y_end, width, color, snap);
        return;
    }

    // Ensure we draw top-to-bottom
    let (start, end) = if y_start < y_end {
        (y_start, y_end)
    } else {
        (y_end, y_start)
    };

    let mut current_y = start;

    // Pre-calculate X geometry
    let x_snapped = maybe_snap(x, width, snap);
    let half_width = width / 2.0;
    let left = x_snapped - half_width;
    let right = x_snapped + half_width;

    let packed_color = pack(color);

    #[allow(clippy::while_float)]
    while current_y < end {
        let seg_end_y = (current_y + dash_length).min(end);

        let mesh = buffer.get_mesh_mut();
        let start_index = mesh.vertices.len() as u32;

        mesh.vertices.extend_from_slice(&[
            SolidVertex2D {
                position: [left, current_y],
                color: packed_color,
            },
            SolidVertex2D {
                position: [right, current_y],
                color: packed_color,
            },
            SolidVertex2D {
                position: [left, seg_end_y],
                color: packed_color,
            },
            SolidVertex2D {
                position: [right, seg_end_y],
                color: packed_color,
            },
        ]);

        mesh.indices.extend_from_slice(&[
            start_index,
            start_index + 1,
            start_index + 2,
            start_index + 1,
            start_index + 3,
            start_index + 2,
        ]);

        current_y += dash_length + gap_length;
    }
}

/// Draws a perfectly horizontal dotted line using circular dots.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn draw_horizontal_dotted_line(
    buffer: &mut MeshData,
    x_start: f32,
    x_end: f32,
    y: f32,
    width: f32,
    color: Color,
    gap_length: f32,
    snap: bool,
) {
    // Prevent inifnite loops
    if gap_length < 0.0 || width <= 0.0 {
        return;
    }

    let radius = width / 2.0;
    let spacing = width + gap_length; // Space taken by the dot + the gap

    let (start, end) = if x_start < x_end {
        (x_start, x_end)
    } else {
        (x_end, x_start)
    };

    let y_snapped = maybe_snap(y, width, snap);

    // Start the first dot shifted by its radius so the edge of the dot touches x_start
    let mut current_x = start + radius;

    #[allow(clippy::while_float)]
    while current_x <= end - radius {
        draw_dot(buffer, Point::new(current_x, y_snapped), radius, color);
        current_x += spacing;
    }
}

/// Draws a perfectly vertical dotted line using circular dots.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn draw_vertical_dotted_line(
    buffer: &mut MeshData,
    x: f32,
    y_start: f32,
    y_end: f32,
    width: f32,
    color: Color,
    gap_length: f32,
    snap: bool,
) {
    let radius = width / 2.0;
    let spacing = width + gap_length;

    let (start, end) = if y_start < y_end {
        (y_start, y_end)
    } else {
        (y_end, y_start)
    };

    let x_snapped = maybe_snap(x, width, snap);
    let mut current_y = start + radius;

    #[allow(clippy::while_float)]
    while current_y <= end - radius {
        draw_dot(buffer, Point::new(x_snapped, current_y), radius, color);
        current_y += spacing;
    }
}

/// Draws a path of connected segments as a series of perfectly circular dots.
///
/// It maintains the exact arc-length spacing across corners to ensure smooth
/// continuous dotted patterns regardless of segment lengths.
pub fn draw_dotted_path(
    buffer: &mut MeshData,
    points: &[Point],
    width: f32,
    gap: f32,
    color: Color,
) {
    if points.is_empty() || gap < 0.0 || width <= 0.0 {
        return;
    }

    let radius = width / 2.0;
    let spacing = width + gap;

    // Draw the very first dot exactly at the start point
    draw_dot(buffer, points[0], radius, color);
    let mut dist_since_last_dot = 0.0;

    // Iterate over every connected segment
    for window in points.windows(2) {
        let p_start = window[0];
        let p_end = window[1];

        let dx = p_end.x - p_start.x;
        let dy = p_end.y - p_start.y;
        let len = dx.hypot(dy);

        // Skip microscopic segments to prevent divide-by-zero
        if len < 0.0001 {
            continue;
        }

        let dir_x = dx / len;
        let dir_y = dy / len;

        // Calculate how far we need to travel along THIS segment to drop the next dot
        let mut advance = spacing - dist_since_last_dot;

        #[allow(clippy::while_float)]
        while advance <= len {
            let dot_center = Point::new(
                dir_x.mul_add(advance, p_start.x),
                dir_y.mul_add(advance, p_start.y),
            );
            draw_dot(buffer, dot_center, radius, color);
            advance += spacing;
        }

        // Save the remaining distance on this segment for the next segment in the polyline
        dist_since_last_dot = len - (advance - spacing);
    }
}
