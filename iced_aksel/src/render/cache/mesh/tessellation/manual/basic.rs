//! Basic geometric primitives (Rectangles and Ellipses).
//!
//! This module handles shapes that can be trivially defined by a bounding box or center/radius.

use crate::{radii::ResolvedRadii, render::cache::MeshData};
use iced_core::{Color, Point};
use iced_graphics::{color::pack, mesh::SolidVertex2D};

/// Writes a polygon to represent a circular dot.
///
/// Automatically scales the Level of Detail (LOD). For tiny dots, it uses a highly
/// optimized 8-sided polygon. For larger dots, it dynamically calculates the
/// required segments for a smooth circle.
#[inline]
pub fn draw_dot(buffer: &mut MeshData, center: Point, radius: f32, color: Color) {
    let packed_color = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;

    // =========================================================================
    // FAST PATH: Small dots (<= 6px diameter)
    // =========================================================================
    if radius <= 3.0 {
        let diag = radius * std::f32::consts::FRAC_1_SQRT_2;

        mesh.vertices.push(SolidVertex2D {
            position: [center.x, center.y],
            color: packed_color,
        });

        mesh.vertices.extend_from_slice(&[
            SolidVertex2D {
                position: [center.x + radius, center.y],
                color: packed_color,
            },
            SolidVertex2D {
                position: [center.x + diag, center.y + diag],
                color: packed_color,
            },
            SolidVertex2D {
                position: [center.x, center.y + radius],
                color: packed_color,
            },
            SolidVertex2D {
                position: [center.x - diag, center.y + diag],
                color: packed_color,
            },
            SolidVertex2D {
                position: [center.x - radius, center.y],
                color: packed_color,
            },
            SolidVertex2D {
                position: [center.x - diag, center.y - diag],
                color: packed_color,
            },
            SolidVertex2D {
                position: [center.x, center.y - radius],
                color: packed_color,
            },
            SolidVertex2D {
                position: [center.x + diag, center.y - diag],
                color: packed_color,
            },
        ]);

        mesh.indices.extend_from_slice(&[
            start_index,
            start_index + 1,
            start_index + 2,
            start_index,
            start_index + 2,
            start_index + 3,
            start_index,
            start_index + 3,
            start_index + 4,
            start_index,
            start_index + 4,
            start_index + 5,
            start_index,
            start_index + 5,
            start_index + 6,
            start_index,
            start_index + 6,
            start_index + 7,
            start_index,
            start_index + 7,
            start_index + 8,
            start_index,
            start_index + 8,
            start_index + 1,
        ]);
        return;
    }

    // =========================================================================
    // DYNAMIC PATH: Large dots (> 6px diameter)
    // =========================================================================

    // Calculate segments based on circumference.
    // We clamp between 12 and 64 segments to prevent insane vertex bloat on massive dots.
    let segments = (radius * std::f32::consts::PI).clamp(12.0, 64.0) as usize;
    let step_angle = std::f32::consts::TAU / segments as f32;

    // Center Vertex
    mesh.vertices.push(SolidVertex2D {
        position: [center.x, center.y],
        color: packed_color,
    });

    // Rim Vertices (calculated dynamically)
    for i in 0..segments {
        let theta = i as f32 * step_angle;
        let (sin, cos) = theta.sin_cos();
        mesh.vertices.push(SolidVertex2D {
            position: [cos.mul_add(radius, center.x), sin.mul_add(radius, center.y)],
            color: packed_color,
        });
    }

    // Connect indices into a triangle fan
    for i in 0..segments {
        let current = (i + 1) as u32;
        let next = if i == segments - 1 { 1 } else { current + 1 };
        mesh.indices
            .extend_from_slice(&[start_index, start_index + current, start_index + next]);
    }
}

/// Writes a filled axis-aligned rectangle.
///
/// Uses 4 vertices and 2 triangles (Quad).
#[inline]
pub fn draw_fill_rect(
    buffer: &mut MeshData,
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
    buffer: &mut MeshData,
    x_min: f32,
    y_min: f32,
    x_max: f32,
    y_max: f32,
    thickness: f32,
    color: Color,
) {
    let packed_color = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;

    // Inner rectangle coordinates
    let inner_x_min = x_min + thickness;
    let inner_x_max = x_max - thickness;
    let inner_y_min = y_min + thickness;
    let inner_y_max = y_max - thickness;

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
    mesh.indices.extend_from_slice(&[
        // Top Edge
        start_index,
        start_index + 1,
        start_index + 4,
        start_index + 1,
        start_index + 4,
        start_index + 5,
        // Right Edge
        start_index + 1,
        start_index + 2,
        start_index + 5,
        start_index + 2,
        start_index + 5,
        start_index + 6,
        // Bottom Edge
        start_index + 2,
        start_index + 3,
        start_index + 6,
        start_index + 3,
        start_index + 6,
        start_index + 7,
        // Left Edge
        start_index + 3,
        start_index,
        start_index + 7,
        start_index,
        start_index + 7,
        start_index + 4,
    ]);
}

/// Writes a filled circle or ellipse using a Triangle Fan.
///
/// A center vertex is created, and surrounding vertices are generated via trigonometry.
#[inline]
pub fn draw_fill_circle(
    buffer: &mut MeshData,
    center: Point,
    radii: ResolvedRadii,
    color: Color,
    segments: usize,
) {
    let packed_color = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;
    let step_angle = std::f32::consts::TAU / segments as f32;

    // Center Vertex (Hub of the fan)
    mesh.vertices.push(SolidVertex2D {
        position: [center.x, center.y],
        color: packed_color,
    });

    // Rim Vertices
    for i in 0..segments {
        let theta = i as f32 * step_angle;
        let (sin, cos) = theta.sin_cos();
        mesh.vertices.push(SolidVertex2D {
            position: [
                cos.mul_add(radii.x, center.x),
                sin.mul_add(radii.y, center.y),
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
pub fn draw_stroke_circle(
    buffer: &mut MeshData,
    center: Point,
    inner_radius: ResolvedRadii,
    outer_radius: ResolvedRadii,
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
                cos.mul_add(inner_radius.x, center.x),
                sin.mul_add(inner_radius.y, center.y),
            ],
            color: packed_color,
        });

        // Outer Vertex
        mesh.vertices.push(SolidVertex2D {
            position: [
                cos.mul_add(outer_radius.x, center.x),
                sin.mul_add(outer_radius.y, center.y),
            ],
            color: packed_color,
        });
    }

    // Connect the strip
    super::push_ring_strip_indices(mesh, start_index, segments, true);
}
