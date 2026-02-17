pub mod basic;
pub mod linear;
pub mod polygon;
pub mod radial;

use crate::render::cache::MeshData;
use iced_core::{Color, Point, Vector};

/// The "Fast Path" rendering engine.
///
/// This component contains highly optimized, allocation-free algorithms for generating
/// vertices for standard geometric primitives. Unlike generic tessellators, it uses
/// domain-specific knowledge (e.g., "a circle is just a triangle fan") to write
/// vertices directly to the [`MeshData`].
///
/// # Layman's Terms
/// This acts as a **direct writer**. It skips general-purpose geometry math and instead uses
/// specific, optimized formulas. For example, it knows that a rectangle is always made of
/// exactly two triangles, so it calculates and writes those six points directly to memory.
pub struct ManualTessellator;

impl ManualTessellator {
    // =========================================================================
    //  Basic Primitives (Rectangles & Circles)
    // =========================================================================

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn draw_fill_rect(
        &self,
        buffer: &mut MeshData,
        x_min: f32,
        y_min: f32,
        x_max: f32,
        y_max: f32,
        color: Color,
        snap: bool,
    ) {
        basic::draw_fill_rect(buffer, x_min, y_min, x_max, y_max, color, snap);
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn draw_stroke_rect(
        &self,
        buffer: &mut MeshData,
        x_min: f32,
        y_min: f32,
        x_max: f32,
        y_max: f32,
        thickness: f32,
        color: Color,
    ) {
        basic::draw_stroke_rect(buffer, x_min, y_min, x_max, y_max, thickness, color);
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn draw_fill_circle(
        &self,
        buffer: &mut MeshData,
        center: Point,
        radius: Point,
        color: Color,
        segments: usize,
    ) {
        basic::draw_fill_circle(buffer, center, radius, color, segments);
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn draw_stroke_circle(
        &self,
        buffer: &mut MeshData,
        center: Point,
        inner_radius: Point,
        outer_radius: Point,
        color: Color,
        segments: usize,
    ) {
        basic::draw_stroke_circle(buffer, center, inner_radius, outer_radius, color, segments);
    }

    // =========================================================================
    //  Linear Primitives (Lines & Arrows)
    // =========================================================================

    #[inline]
    pub fn draw_line_segment(
        &self,
        buffer: &mut MeshData,
        start: Point,
        end: Point,
        width: f32,
        color: Color,
    ) {
        linear::draw_line_segment(buffer, start, end, width, color);
    }

    #[inline]
    pub fn draw_arrowhead(
        &self,
        buffer: &mut MeshData,
        tip: Point,
        direction: Vector, // CHANGED: Point -> Vector
        line_width: f32,
        arrow_size_multiplier: f32,
        color: Color,
    ) {
        linear::draw_arrowhead(
            buffer,
            tip,
            direction,
            line_width,
            arrow_size_multiplier,
            color,
        );
    }

    // =========================================================================
    //  Polygons (Triangles, Fans, Rings)
    // =========================================================================

    #[inline]
    pub fn draw_fill_triangle(
        &self,
        buffer: &mut MeshData,
        p1: Point,
        p2: Point,
        p3: Point,
        color: Color,
    ) {
        polygon::draw_fill_triangle(buffer, p1, p2, p3, color);
    }

    #[inline]
    pub fn draw_stroke_triangle(
        &self,
        buffer: &mut MeshData,
        outer: [Point; 3],
        inner: [Point; 3],
        color: Color,
    ) {
        polygon::draw_stroke_triangle(buffer, outer, inner, color);
    }

    #[inline]
    pub fn draw_fan(&self, buffer: &mut MeshData, points: &[Point], color: Color) {
        polygon::draw_fan(buffer, points, color);
    }

    #[inline]
    pub fn draw_ring(
        &self,
        buffer: &mut MeshData,
        outer_points: &[Point],
        inner_points: &[Point],
        color: Color,
    ) {
        polygon::draw_ring(buffer, outer_points, inner_points, color);
    }

    // =========================================================================
    //  Radial Primitives (Arcs & Sectors)
    // =========================================================================

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn draw_arc_strip(
        &self,
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
        radial::draw_arc_strip(
            buffer,
            center_x,
            center_y,
            radius_inner,
            radius_outer,
            start_angle,
            end_angle,
            color,
            segments,
        );
    }
}
