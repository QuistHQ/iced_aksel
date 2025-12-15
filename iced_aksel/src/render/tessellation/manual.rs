pub mod basic;
pub mod linear;
pub mod mesh;
pub mod polygon;
pub mod radial;

use crate::render::MeshBuffer;
use iced_core::{Color, Point, Vector};

/// The "Fast Path" tessellator.
///
/// This struct aggregates the pure functions from sub-modules.
#[derive(Default)]
pub struct ManualTessellator;

impl ManualTessellator {
    // Basic
    #[inline]
    pub fn draw_fill_rect(
        &mut self,
        buffer: &mut MeshBuffer,
        x_min: f32,
        y_min: f32,
        x_max: f32,
        y_max: f32,
        color: Color,
    ) {
        basic::draw_fill_rect(buffer, x_min, y_min, x_max, y_max, color);
    }
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn draw_stroke_rect(
        &mut self,
        buffer: &mut MeshBuffer,
        x_min: f32,
        y_min: f32,
        x_max: f32,
        y_max: f32,
        th_x: f32,
        th_y: f32,
        color: Color,
    ) {
        basic::draw_stroke_rect(buffer, x_min, y_min, x_max, y_max, th_x, th_y, color);
    }
    #[inline]
    pub fn draw_fill_circle(
        &mut self,
        buffer: &mut MeshBuffer,
        cx: f32,
        cy: f32,
        r: f32,
        color: Color,
        segments: usize,
    ) {
        basic::draw_fill_circle(buffer, cx, cy, r, color, segments);
    }
    #[inline]
    pub fn draw_stroke_circle(
        &mut self,
        buffer: &mut MeshBuffer,
        cx: f32,
        cy: f32,
        r_in: f32,
        r_out: f32,
        color: Color,
        segments: usize,
    ) {
        basic::draw_stroke_circle(buffer, cx, cy, r_in, r_out, color, segments);
    }

    // Linear
    #[inline]
    pub fn draw_line_segment(
        &mut self,
        buffer: &mut MeshBuffer,
        p1: Point,
        p2: Point,
        w: f32,
        color: Color,
    ) {
        linear::draw_line_segment(buffer, p1, p2, w, color);
    }
    #[inline]
    pub fn draw_arrowhead(
        &mut self,
        buffer: &mut MeshBuffer,
        tip: Point,
        dir: Vector,
        w: f32,
        size: f32,
        color: Color,
    ) {
        linear::draw_arrowhead(buffer, tip, dir, w, size, color);
    }

    // Polygon
    #[inline]
    pub fn draw_fill_triangle(
        &mut self,
        buffer: &mut MeshBuffer,
        p1: Point,
        p2: Point,
        p3: Point,
        color: Color,
    ) {
        polygon::draw_fill_triangle(buffer, p1, p2, p3, color);
    }
    #[inline]
    pub fn draw_stroke_triangle(
        &mut self,
        buffer: &mut MeshBuffer,
        outer: [Point; 3],
        inner: [Point; 3],
        color: Color,
    ) {
        polygon::draw_stroke_triangle(buffer, outer, inner, color);
    }
    #[inline]
    pub fn draw_fan(&mut self, buffer: &mut MeshBuffer, points: &[Point], color: Color) {
        polygon::draw_fan(buffer, points, color);
    }
    #[inline]
    pub fn draw_ring(
        &mut self,
        buffer: &mut MeshBuffer,
        outer: &[Point],
        inner: &[Point],
        color: Color,
    ) {
        polygon::draw_ring(buffer, outer, inner, color);
    }

    // Radial
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn draw_arc_strip(
        &mut self,
        buffer: &mut MeshBuffer,
        cx: f32,
        cy: f32,
        r_in: f32,
        r_out: f32,
        start: f32,
        end: f32,
        color: Color,
        segments: usize,
    ) {
        radial::draw_arc_strip(buffer, cx, cy, r_in, r_out, start, end, color, segments);
    }

    // Mesh
    #[inline]
    pub fn draw_mesh(
        &mut self,
        buffer: &mut MeshBuffer,
        vertices: &[Point],
        indices: &[u32],
        color: Color,
    ) {
        mesh::draw_mesh(buffer, vertices, indices, color);
    }
}
