//! General polygon primitives.
//!
//! Helper functions for rendering arbitrary triangles, fans (convex shapes),
//! and rings (thick polygon borders).

use crate::render::MeshBuffer;
use iced_core::{Color, Point};
use iced_graphics::{color::pack, mesh::SolidVertex2D};

/// Writes a single filled triangle.
#[inline]
pub fn draw_fill_triangle(buffer: &mut MeshBuffer, p1: Point, p2: Point, p3: Point, color: Color) {
    let packed_color = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;

    mesh.vertices.extend_from_slice(&[
        SolidVertex2D {
            position: [p1.x, p1.y],
            color: packed_color,
        },
        SolidVertex2D {
            position: [p2.x, p2.y],
            color: packed_color,
        },
        SolidVertex2D {
            position: [p3.x, p3.y],
            color: packed_color,
        },
    ]);

    mesh.indices
        .extend_from_slice(&[start_index, start_index + 1, start_index + 2]);
}

/// Writes a "frame" triangle (a triangle with a hole in it).
///
/// Used for rendering stroked triangles. Takes an outer triangle and an inner triangle
/// and connects them.
#[inline]
pub fn draw_stroke_triangle(
    buffer: &mut MeshBuffer,
    outer: [Point; 3],
    inner: [Point; 3],
    color: Color,
) {
    let packed_color = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;

    for p in outer.iter().chain(inner.iter()) {
        mesh.vertices.push(SolidVertex2D {
            position: [p.x, p.y],
            color: packed_color,
        });
    }

    // Connect Outer[0..2] to Inner[3..5]
    // 0=OuterA, 1=OuterB, 2=OuterC
    // 3=InnerA, 4=InnerB, 5=InnerC
    #[rustfmt::skip]
    mesh.indices.extend_from_slice(&[
        0, 1, 3,  1, 4, 3, // Edge 1
        1, 2, 4,  2, 5, 4, // Edge 2
        2, 0, 5,  0, 3, 5, // Edge 3
    ]);

    // Offset indices by start_index
    let len = mesh.indices.len();
    for i in (len - 18)..len {
        mesh.indices[i] += start_index;
    }
}

/// Writes a Triangle Fan (used for convex polygons like Hexagons).
///
/// Takes the first point as the center, and connects subsequent points to it.
#[inline]
pub fn draw_fan(buffer: &mut MeshBuffer, points: &[Point], color: Color) {
    if points.len() < 3 {
        return;
    }

    let packed_color = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;

    // Center point (Hub) is derived from the first vertex in the list usually,
    // or calculated. Here we assume the points form a convex loop.
    // Ideally for a fan, we pick points[0] as the pivot.

    for p in points {
        mesh.vertices.push(SolidVertex2D {
            position: [p.x, p.y],
            color: packed_color,
        });
    }

    // Fan topology: Connect Pivot(0) to i and i+1
    for i in 1..(points.len() - 1) {
        mesh.indices.extend_from_slice(&[
            start_index,
            start_index + i as u32,
            start_index + i as u32 + 1,
        ]);
    }
}

/// Writes a thick ring defined by two polygon boundaries (Outer and Inner).
///
/// Used for stroking convex polygons.
#[inline]
pub fn draw_ring(
    buffer: &mut MeshBuffer,
    outer_points: &[Point],
    inner_points: &[Point],
    color: Color,
) {
    let len = outer_points.len();
    if len < 3 || inner_points.len() != len {
        return;
    }

    let packed_color = pack(color);
    let mesh = buffer.get_mesh_mut();
    let start_index = mesh.vertices.len() as u32;

    for i in 0..len {
        mesh.vertices.push(SolidVertex2D {
            position: [outer_points[i].x, outer_points[i].y],
            color: packed_color,
        });
        mesh.vertices.push(SolidVertex2D {
            position: [inner_points[i].x, inner_points[i].y],
            color: packed_color,
        });
    }

    for i in 0..len {
        let current = i as u32;
        let next = ((i + 1) % len) as u32;

        let outer_curr = start_index + current * 2;
        let inner_curr = start_index + current * 2 + 1;
        let outer_next = start_index + next * 2;
        let inner_next = start_index + next * 2 + 1;

        mesh.indices.extend_from_slice(&[
            outer_curr, outer_next, inner_curr, inner_curr, outer_next, inner_next,
        ]);
    }
}
