//! Basic geometric primitives (Rectangles and Ellipses).
//!
//! This module handles shapes that can be trivially defined by a bounding box or center/radius.

use crate::render::MeshBuffer;
use iced_core::Color;
use iced_graphics::{color::pack, mesh::SolidVertex2D};

/// Writes a filled axis-aligned rectangle.
///
/// Uses 4 vertices and 2 triangles (Quad).
#[inline]
pub fn draw_fill_rect(
    buffer: &mut MeshBuffer,
    mut x_min: f32,
    mut y_min: f32,
    mut x_max: f32,
    mut y_max: f32,
    color: Color,
    snap: bool,
) {
    let packed_color = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;

    if snap {
        x_min = x_min.round();
        y_min = y_min.round();
        x_max = x_max.round();
        y_max = y_max.round();
    }

    mesh.vertices.extend_from_slice(&[
        SolidVertex2D {
            position: [x_min, y_min],
            color: packed_color,
        }, // Top-Left
        SolidVertex2D {
            position: [x_max, y_min],
            color: packed_color,
        }, // Top-Right
        SolidVertex2D {
            position: [x_max, y_max],
            color: packed_color,
        }, // Bottom-Right
        SolidVertex2D {
            position: [x_min, y_max],
            color: packed_color,
        }, // Bottom-Left
    ]);

    // Standard Quad Indices (0-1-2, 0-2-3)
    #[rustfmt::skip]
    mesh.indices.extend_from_slice(&[
        start_index, start_index + 1, start_index + 2,
        start_index, start_index + 2, start_index + 3,
    ]);
}

/// Writes a hollow rectangle (border).
///
/// Constructed as a "picture frame" using 8 vertices and 8 triangles to form the thick border.
/// This prevents overdraw artifacts that occur if we were to just draw 4 overlapping line segments.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn draw_stroke_rect(
    buffer: &mut MeshBuffer,
    x_min: f32,
    y_min: f32,
    x_max: f32,
    y_max: f32,
    thickness_x: f32,
    thickness_y: f32,
    color: Color,
) {
    let packed_color = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;

    // Inner rectangle coordinates
    let inner_x_min = x_min + thickness_x;
    let inner_x_max = x_max - thickness_x;
    let inner_y_min = y_min + thickness_y;
    let inner_y_max = y_max - thickness_y;

    mesh.vertices.extend_from_slice(&[
        // Outer Loop (0-3)
        SolidVertex2D {
            position: [x_min, y_min],
            color: packed_color,
        },
        SolidVertex2D {
            position: [x_max, y_min],
            color: packed_color,
        },
        SolidVertex2D {
            position: [x_max, y_max],
            color: packed_color,
        },
        SolidVertex2D {
            position: [x_min, y_max],
            color: packed_color,
        },
        // Inner Loop (4-7)
        SolidVertex2D {
            position: [inner_x_min, inner_y_min],
            color: packed_color,
        },
        SolidVertex2D {
            position: [inner_x_max, inner_y_min],
            color: packed_color,
        },
        SolidVertex2D {
            position: [inner_x_max, inner_y_max],
            color: packed_color,
        },
        SolidVertex2D {
            position: [inner_x_min, inner_y_max],
            color: packed_color,
        },
    ]);

    // Connect outer loop to inner loop using a triangle strip approach
    #[rustfmt::skip]
    mesh.indices.extend_from_slice(&[
        start_index, start_index + 1, start_index + 4,   start_index + 1, start_index + 4, start_index + 5, // Top Edge
        start_index + 1, start_index + 2, start_index + 5,   start_index + 2, start_index + 5, start_index + 6, // Right Edge
        start_index + 2, start_index + 3, start_index + 6,   start_index + 3, start_index + 6, start_index + 7, // Bottom Edge
        start_index + 3, start_index, start_index + 7,   start_index, start_index + 7, start_index + 4, // Left Edge
    ]);
}

/// Writes a filled circle or ellipse using a Triangle Fan.
///
/// A center vertex is created, and surrounding vertices are generated via trigonometry.
#[inline]
pub fn draw_fill_circle(
    buffer: &mut MeshBuffer,
    center_x: f32,
    center_y: f32,
    radius_x: f32,
    radius_y: f32,
    color: Color,
    segments: usize,
) {
    let packed_color = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;
    let step_angle = std::f32::consts::TAU / segments as f32;

    // Center Vertex (Hub of the fan)
    mesh.vertices.push(SolidVertex2D {
        position: [center_x, center_y],
        color: packed_color,
    });

    // Rim Vertices
    for i in 0..segments {
        let theta = i as f32 * step_angle;
        let (sin, cos) = theta.sin_cos();
        mesh.vertices.push(SolidVertex2D {
            position: [
                cos.mul_add(radius_x, center_x),
                sin.mul_add(radius_y, center_y),
            ],
            color: packed_color,
        });
    }

    // Connect indices (Hub -> Current -> Next)
    for i in 0..segments {
        let current = (i + 1) as u32;
        let next = if i == segments - 1 { 1 } else { current + 1 };
        mesh.indices
            .extend_from_slice(&[start_index, start_index + current, start_index + next]);
    }
}

/// Writes a hollow circle or ellipse (ring) using a Triangle Strip.
///
/// Creates an inner ring and an outer ring of vertices and laces them together.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn draw_stroke_circle(
    buffer: &mut MeshBuffer,
    center_x: f32,
    center_y: f32,
    radius_x_inner: f32,
    radius_y_inner: f32,
    radius_x_outer: f32,
    radius_y_outer: f32,
    color: Color,
    segments: usize,
) {
    let packed_color = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;
    let step_angle = std::f32::consts::TAU / segments as f32;

    for i in 0..segments {
        let theta = i as f32 * step_angle;
        let (sin, cos) = theta.sin_cos();

        // Inner Vertex
        mesh.vertices.push(SolidVertex2D {
            position: [
                cos.mul_add(radius_x_inner, center_x),
                sin.mul_add(radius_y_inner, center_y),
            ],
            color: packed_color,
        });

        // Outer Vertex
        mesh.vertices.push(SolidVertex2D {
            position: [
                cos.mul_add(radius_x_outer, center_x),
                sin.mul_add(radius_y_outer, center_y),
            ],
            color: packed_color,
        });
    }

    // Connect the strip
    for i in 0..segments {
        let i = i as u32;
        let next_i = (i + 1) % segments as u32;

        let inner_current = start_index + i * 2;
        let outer_current = start_index + i * 2 + 1;
        let inner_next = start_index + next_i * 2;
        let outer_next = start_index + next_i * 2 + 1;

        mesh.indices.extend_from_slice(&[
            inner_current,
            outer_current,
            outer_next, // Triangle 1
            inner_current,
            outer_next,
            inner_next, // Triangle 2
        ]);
    }
}
