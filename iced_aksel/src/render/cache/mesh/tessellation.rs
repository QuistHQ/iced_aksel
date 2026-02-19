//! The core vector tessellation engine.
//!
//! This module is responsible for converting high-level geometric primitives (Circles,
//! Arcs, Text, Polygons) into raw triangle meshes that the GPU can render.
//!
//! # Architecture
//! The `Tessellator` acts as the central "Brain" of the rendering pipeline. It orchestrates
//! two distinct strategies:
//!
//! 1.  **Fast Path (Manual):** Direct vertex generation for simple shapes.
//!     * Used for: Rectangles, Circles, Triangles, Solid Lines.
//!     * Benefit: Extremely fast (no heavy math libraries).
//!
//! 2.  **Robust Path (Complex):** Uses the `lyon` library for advanced geometry.
//!     * Used for: Dashed lines, Bezier, Complex Polygons, Text.
//!     * Benefit: Handles mathematically difficult edge cases (e.g., stroke miters, dash gaps).

pub mod complex;
pub mod manual;
pub mod math;
pub mod text;

use crate::{
    Quality,
    radii::ResolvedRadii,
    render::{
        primitive::{LineArrows, LineExtensions},
        text::Text,
    },
    stroke::{ResolvedStroke, StrokeStyle},
};
use complex::{ComplexTessellator, DashedPolyline, LyonAdapter, SolidVertexConstructor};
use iced_core::{Color, Point, Rectangle};
use iced_graphics::{color::pack, mesh::SolidVertex2D};
use lyon_path::{LineCap, LineJoin, Path, PathEvent, iterator::FromPolyline, traits::PathIterator};
use lyon_tessellation::{StrokeOptions, VertexBuffers};
use math::*;
use text::{TextRenderContext, TextTessellationCache};

/// The central driver for the rendering engine.
///
/// It encapsulates the underlying sub-tessellators and manages shared resources
/// like the glyph cache and scratch buffers to minimize memory allocations.
pub struct Tessellator {
    /// Handles complex paths (dashes, curves) using Lyon.
    complex: ComplexTessellator,
    /// Handles simple primitives (rects, circles) using direct math.
    manual: manual::ManualTessellator,

    /// A reusable scratch buffer for intermediate tessellation (e.g., text glyphs).
    /// Prevents re-allocating memory for every single character.
    scratch_geometry: VertexBuffers<Point, u16>,

    /// Cache for tessellated text glyphs to avoid re-tessellating the alphabet every frame.
    glyph_cache: TextTessellationCache,

    /// Global quality multiplier.
    /// * 1.0 = Standard (Default).
    /// * 0.5 = High Performance (Lower vertex count for curves).
    /// * 2.0 = High Quality (Smoother curves).
    quality: f32,

    /// Global text quality tolerance
    text_tolerance: f32,
}

impl Default for Tessellator {
    fn default() -> Self {
        Self {
            complex: ComplexTessellator::default(),
            manual: manual::ManualTessellator,
            scratch_geometry: VertexBuffers::new(),
            glyph_cache: TextTessellationCache::new(),
            quality: 1.0,
            text_tolerance: 0.5,
        }
    }
}

impl Tessellator {
    /// Creates a new, default tessellator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the rendering quality multiplier.
    ///
    /// This value controls the Level of Detail (LOD) for curves, arcs, circles, and text.
    /// * `1.0` is the default standard.
    /// * Values `< 1.0` reduce vertex count for higher performance.
    /// * Values `> 1.0` increase vertex count for smoother visuals.
    ///
    /// # Cache Invalidation
    /// If the quality changes significantly, this method automatically clears the
    /// text cache. This prevents the "Sticky Cache" problem where high-quality glyphs
    /// persist even after the user requests lower quality (or vice versa).
    pub fn set_quality(&mut self, quality: Quality) {
        let new_tolerance = quality.to_text_tolerance();

        self.quality = quality.to_tessellation_quality();

        // If the quality has changed significantly (more than float error)...
        if (self.text_tolerance - new_tolerance).abs() > 0.001 {
            self.text_tolerance = new_tolerance;

            // ...invalidate the cache.
            // This forces the engine to regenerate geometry at the new
            // quality level.
            self.glyph_cache.clear();
        }
    }

    /// Returns the globally set text-tolerance of this tessellator
    pub const fn text_tolerance(&self) -> f32 {
        self.text_tolerance
    }

    // =========================================================================
    //  Primitives
    // =========================================================================

    /// Draws an axis-aligned rectangle.
    ///
    /// # Optimization
    /// If the stroke thickness is large enough to cover the entire rectangle (e.g. thickness >= width),
    /// the engine will automatically switch to drawing a simple filled rectangle to save vertices.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_rectangle(
        &mut self,
        buffer: &mut crate::render::cache::MeshData,
        x_min: f32,
        y_min: f32,
        x_max: f32,
        y_max: f32,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    ) {
        let width = x_max - x_min;
        let height = y_max - y_min;

        // Check if the stroke is so thick it completely swallows the shape.
        let is_consumed = stroke.as_ref().is_some_and(|stroke| {
            stroke.thickness >= width / 2.0 || stroke.thickness >= height / 2.0
        });

        if is_consumed {
            if let Some(stroke) = stroke {
                self.manual
                    .draw_fill_rect(buffer, x_min, y_min, x_max, y_max, stroke.fill, true);
            }
            return;
        }

        if let Some(color) = fill {
            // "Bleed fix": Overlap fill and stroke slightly to prevent sub-pixel gaps (antialiasing artifacts)
            let overlap = if stroke.is_some() && width > 1.0 && height > 1.0 {
                0.5
            } else {
                0.0
            };

            self.manual.draw_fill_rect(
                buffer,
                x_min + overlap,
                y_min + overlap,
                x_max - overlap,
                y_max - overlap,
                color,
                true,
            );
        }

        if let Some(stroke) = stroke {
            match stroke.style {
                StrokeStyle::Solid => {
                    self.manual.draw_stroke_rect(
                        buffer,
                        x_min,
                        y_min,
                        x_max,
                        y_max,
                        stroke.thickness,
                        stroke.fill,
                    );
                }
                _ => {
                    // For dashed/dotted lines, we must construct a path for Lyon to process
                    let offset = (stroke.thickness * 2.0) / 4.0;
                    let points = vec![
                        lyon_tessellation::math::Point::new(x_min + offset, y_min + offset),
                        lyon_tessellation::math::Point::new(x_max - offset, y_min + offset),
                        lyon_tessellation::math::Point::new(x_max - offset, y_max - offset),
                        lyon_tessellation::math::Point::new(x_min + offset, y_max - offset),
                        lyon_tessellation::math::Point::new(x_min + offset, y_min + offset),
                    ];
                    self.stroke_polyline(buffer, points, stroke, true);
                }
            }
        }
    }

    /// Draws an ellipse.
    ///
    /// * `center_x`, `center_y`: The center coordinates in screen space.
    /// * `radius_x`, `radius_y`: The radii. If equal, draws a circle. If different, draws an ellipse.
    ///
    /// # Performance
    /// This method automatically adjusts the number of segments (vertices) used to approximate the curve
    /// based on the radius and the current [`quality`](Self::quality) setting.
    pub fn draw_ellipse(
        &mut self,
        buffer: &mut crate::render::cache::MeshData,
        center: Point,
        radii: ResolvedRadii,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    ) {
        let max_radius = radii.x.max(radii.y);
        let is_consumed = stroke
            .as_ref()
            .is_some_and(|stroke| stroke.thickness >= max_radius);

        let segments = self.resolve_lod(max_radius);

        if is_consumed {
            if let Some(stroke) = stroke {
                self.manual
                    .draw_fill_circle(buffer, center, radii, stroke.fill, segments);
            }
            return;
        }

        if let Some(color) = fill {
            let overlap = if stroke.is_some() { 0.5 } else { 0.0 };
            let fill_radius = (radii - overlap).max(0.0);
            if fill_radius.x > 0.1 && fill_radius.y > 0.1 {
                self.manual
                    .draw_fill_circle(buffer, center, fill_radius, color, segments);
            }
        }

        if let Some(stroke) = stroke {
            match stroke.style {
                StrokeStyle::Solid => {
                    let inner_radius = radii - stroke.thickness;
                    self.manual.draw_stroke_circle(
                        buffer,
                        center,
                        inner_radius,
                        radii,
                        stroke.fill,
                        segments,
                    );
                }
                _ => {
                    // Elliptical stroking with Lyon for dashed lines
                    let stroke_radius = radii - (stroke.thickness / 2.0);

                    if stroke_radius.x > 0.1 && stroke_radius.y > 0.1 {
                        use lyon_tessellation::geom::Arc;
                        use lyon_tessellation::math::{Angle, Point, Vector};

                        let arc = Arc {
                            center: Point::new(center.x, center.y),
                            radii: Vector::new(stroke_radius.x, stroke_radius.y),
                            start_angle: Angle::radians(0.0),
                            sweep_angle: Angle::radians(std::f32::consts::TAU),
                            x_rotation: Angle::radians(0.0),
                        };

                        let tolerance = 0.2 / self.quality.max(0.1);
                        self.stroke_polyline(buffer, arc.flattened(tolerance), stroke, true);
                    }
                }
            }
        }
    }

    /// Draws a generic triangle defined by three points.
    ///
    /// # Vertex Order
    /// The method automatically calculates the winding order of the vertices to ensure
    /// math consistency. You do not need to provide points in clockwise/counter-clockwise order.
    pub fn draw_triangle(
        &mut self,
        buffer: &mut crate::render::cache::MeshData,
        p1: Point,
        p2: Point,
        p3: Point,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    ) {
        // Compute cross product to check winding order (Clockwise vs Counter-Clockwise)
        let cross_product = (p2.x - p1.x).mul_add(p3.y - p1.y, -((p2.y - p1.y) * (p3.x - p1.x)));

        // Ensure consistent winding order for easier math downstream
        let (p1, p2, p3, double_area) = if cross_product < 0.0 {
            (p1, p3, p2, -cross_product)
        } else {
            (p1, p2, p3, cross_product)
        };

        let (inner_p1, inner_p2, inner_p3, is_consumed) =
            stroke.as_ref().map_or((p1, p2, p3, false), |stroke| {
                let dist_1 = p1.distance(p2);
                let dist_2 = p2.distance(p3);
                let dist_3 = p3.distance(p1);
                let perimeter = dist_1 + dist_2 + dist_3;

                if perimeter < 1e-4 {
                    (Point::ORIGIN, Point::ORIGIN, Point::ORIGIN, true)
                } else {
                    let inradius = double_area / perimeter;

                    // If stroke is thicker than the inradius, the triangle is fully filled by the stroke
                    if stroke.thickness >= inradius {
                        (Point::ORIGIN, Point::ORIGIN, Point::ORIGIN, true)
                    } else {
                        // Compute the inner triangle for the "hole"
                        (
                            compute_inset_vertex(p3, p1, p2, stroke.thickness),
                            compute_inset_vertex(p1, p2, p3, stroke.thickness),
                            compute_inset_vertex(p2, p3, p1, stroke.thickness),
                            false,
                        )
                    }
                }
            });

        if is_consumed {
            if let Some(stroke) = stroke {
                self.manual
                    .draw_fill_triangle(buffer, p1, p2, p3, stroke.fill);
            }
            return;
        }

        if let Some(color) = fill {
            if stroke.is_some() {
                // Apply slight bleed overlap
                let overlap = 0.5;
                let fill_p1 = compute_inset_vertex(p3, p1, p2, overlap);
                let fill_p2 = compute_inset_vertex(p1, p2, p3, overlap);
                let fill_p3 = compute_inset_vertex(p2, p3, p1, overlap);
                self.manual
                    .draw_fill_triangle(buffer, fill_p1, fill_p2, fill_p3, color);
            } else {
                self.manual.draw_fill_triangle(buffer, p1, p2, p3, color);
            }
        }

        if let Some(stroke) = stroke {
            match stroke.style {
                StrokeStyle::Solid => {
                    self.manual.draw_stroke_triangle(
                        buffer,
                        [p1, p2, p3],
                        [inner_p1, inner_p2, inner_p3],
                        stroke.fill,
                    );
                }
                _ => {
                    let offset = stroke.thickness / 2.0;
                    let center_p1 = compute_inset_vertex(p3, p1, p2, offset);
                    let center_p2 = compute_inset_vertex(p1, p2, p3, offset);
                    let center_p3 = compute_inset_vertex(p2, p3, p1, offset);

                    let points = vec![
                        lyon_tessellation::math::Point::new(center_p1.x, center_p1.y),
                        lyon_tessellation::math::Point::new(center_p2.x, center_p2.y),
                        lyon_tessellation::math::Point::new(center_p3.x, center_p3.y),
                        lyon_tessellation::math::Point::new(center_p1.x, center_p1.y),
                    ];
                    self.stroke_polyline(buffer, points, stroke, true);
                }
            }
        }
    }

    /// Draws a regular N-sided polygon (e.g., Hexagon, Octagon).
    ///
    /// * `vertices`: The number of sides (must be >= 3).
    /// * `rotation`: Rotation in degrees. 0.0 aligns the first vertex to the North.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_polygon(
        &mut self,
        buffer: &mut crate::render::cache::MeshData,
        center: Point,
        radius: f32,
        vertices: u16,
        rotation: f32,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    ) {
        if vertices < 3 {
            return;
        }

        let outer_points = generate_ring(center, radius, vertices, rotation);

        if let Some(color) = fill {
            if stroke.is_some() {
                let inset_points = generate_ring(center, radius - 0.5, vertices, rotation);
                self.manual.draw_fan(buffer, &inset_points, color);
            } else {
                self.manual.draw_fan(buffer, &outer_points, color);
            }
        }

        if let Some(stroke) = stroke {
            match stroke.style {
                StrokeStyle::Solid => {
                    let inner_radius = (radius - stroke.thickness).max(0.0);
                    let inner_points = generate_ring(center, inner_radius, vertices, rotation);
                    self.manual
                        .draw_ring(buffer, &outer_points, &inner_points, stroke.fill);
                }
                _ => {
                    let lyon_points: Vec<_> = outer_points
                        .iter()
                        .map(|p| lyon_tessellation::math::Point::new(p.x, p.y))
                        .collect();
                    self.stroke_polyline(buffer, lyon_points, stroke, true);
                }
            }
        }
    }

    // =========================================================================
    //  Line Connectors
    // =========================================================================

    /// Draws a straight line segment.
    ///
    /// # Strategy
    /// * **Finite Lines:** Passed directly to the tessellator. We rely on the GPU scissoring/clipping for performance.
    /// * **Infinite Extensions:** We mathematically project the line to the screen edges (`clip_bounds`)
    ///   to convert "Infinite" into "Finite but spanning the whole screen".
    #[allow(clippy::too_many_arguments)]
    pub fn draw_line(
        &mut self,
        buffer: &mut crate::render::cache::MeshData,
        raw_start: Point,
        raw_end: Point,
        stroke: ResolvedStroke,
        clip_bounds: Rectangle,
        extensions: LineExtensions,
        arrows: LineArrows,
    ) {
        if stroke.thickness < 0.1 {
            return;
        }

        let direction_vector = raw_end - raw_start;

        // Avoid NaN math on degenerate lines
        if direction_vector
            .x
            .mul_add(direction_vector.x, direction_vector.y * direction_vector.y)
            < 1e-12
        {
            return;
        }

        let mut draw_start = raw_start;
        let mut draw_end = raw_end;
        let margin = stroke.thickness.max(1.0);
        let bounds = Bounds::new(clip_bounds, margin);
        let is_infinite = extensions.start || extensions.end;

        // 1. Resolve Infinite Geometry
        if is_infinite {
            let Some((edge_start, edge_end)) = clip_infinite_line(raw_start, raw_end, bounds)
            else {
                return;
            };
            if extensions.start {
                draw_start = edge_start;
            }
            if extensions.end {
                draw_end = edge_end;
            }
        }
        // 2. Resolve Finite Clipping
        else {
            let Some((clipped_p1, clipped_p2)) = clip_segment(draw_start, draw_end, bounds) else {
                return;
            };

            draw_start = clipped_p1;
            draw_end = clipped_p2;
        }

        // 2. Resolve Arrow Retraction
        // We shrink the draw segment so the arrow tip doesn't overlap the line.
        let direction = normalize(direction_vector);
        let arrow_len = stroke.thickness * arrows.size;
        let has_start_arrow = arrows.start && !extensions.start;
        let has_end_arrow = arrows.end && !extensions.end;

        if has_start_arrow {
            draw_start += direction * arrow_len;
        }
        if has_end_arrow {
            draw_end -= direction * arrow_len;
        }

        // 3. Draw Line Body
        match &stroke.style {
            StrokeStyle::Solid => {
                // Performance: Use manual tessellation for solid lines (Fastest)
                self.manual.draw_line_segment(
                    buffer,
                    draw_start,
                    draw_end,
                    stroke.thickness,
                    stroke.fill,
                );
            }
            StrokeStyle::Dotted { gap } => {
                // True circular dots!
                self.manual.draw_dotted_path(
                    buffer,
                    &[draw_start, draw_end],
                    stroke.thickness,
                    *gap,
                    stroke.fill,
                );
            }
            StrokeStyle::Dashed { .. } => {
                // Fallback: Use Lyon for complex dashes
                let points = vec![
                    lyon_tessellation::math::Point::new(draw_start.x, draw_start.y),
                    lyon_tessellation::math::Point::new(draw_end.x, draw_end.y),
                ];
                self.stroke_polyline(buffer, points, stroke, false);
            }
        }

        // 4. Draw Arrowheads
        if has_start_arrow {
            self.manual.draw_arrowhead(
                buffer,
                raw_start,
                -direction,
                stroke.thickness,
                arrows.size,
                stroke.fill,
            );
        }
        if has_end_arrow {
            self.manual.draw_arrowhead(
                buffer,
                raw_end,
                direction,
                stroke.thickness,
                arrows.size,
                stroke.fill,
            );
        }
    }

    /// Draws a connected series of lines (a path).
    ///
    /// Supports dashed lines (via `StrokeStyle`), infinite extensions on the first/last segments,
    /// and arrowheads at the ends.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_polyline(
        &mut self,
        buffer: &mut crate::render::cache::MeshData,
        mut points: Vec<Point>,
        stroke: ResolvedStroke,
        clip_bounds: Rectangle,
        extensions: LineExtensions,
        arrows: LineArrows,
    ) {
        if stroke.thickness < 0.1 {
            return;
        }

        if points.len() < 2 {
            return;
        }

        let last_idx = points.len() - 1;
        let p0 = points[0];
        let pn = points[last_idx];

        // 1. Handle Infinite Extensions
        if extensions.start || extensions.end {
            let margin = stroke.thickness.max(1.0);
            let bounds = Bounds::new(clip_bounds, margin);

            if extensions.start {
                let p1 = points[1];
                if let Some((edge_start, _)) = clip_infinite_line(p0, p1, bounds) {
                    points[0] = edge_start;
                }
            }
            if extensions.end {
                let p_prev = points[last_idx - 1];
                if let Some((_, edge_end)) = clip_infinite_line(p_prev, pn, bounds) {
                    points[last_idx] = edge_end;
                }
            }
        }

        // 2. Handle Arrow Retraction
        if arrows.start && !extensions.start {
            let p1 = points[1];
            let dir = normalize(p1 - p0);
            points[0] = p0 + dir * (stroke.thickness * arrows.size);
        }
        if arrows.end && !extensions.end {
            let p_prev = points[last_idx - 1];
            let dir = normalize(pn - p_prev);
            points[last_idx] = pn - dir * (stroke.thickness * arrows.size);
        }

        // 3. Draw Path
        // Polyline stroking involves complex joints (miters) between segments.
        // We ALWAYS use Lyon here because doing miter joints manually is extremely complex
        // and error-prone (as seen with the triangle artifacts).
        if let StrokeStyle::Dotted { gap } = stroke.style {
            self.manual
                .draw_dotted_path(buffer, &points, stroke.thickness, gap, stroke.fill);
        } else {
            let lyon_points = points
                .iter()
                .map(|p| lyon_tessellation::math::Point::new(p.x, p.y));
            self.stroke_polyline(buffer, lyon_points, stroke, false);
        }

        // 4. Draw Arrowheads
        if arrows.start && !extensions.start {
            let p1 = points[1];
            // Use direction from the original start point (p0) to p1
            // Even if p0 was retracted in the list, the visual direction is the same.
            let direction = normalize(p1 - p0);
            self.manual.draw_arrowhead(
                buffer,
                p0,
                -direction,
                stroke.thickness,
                arrows.size,
                stroke.fill,
            );
        }
        if arrows.end && !extensions.end {
            let p_prev = points[last_idx - 1];
            let direction = normalize(pn - p_prev);
            self.manual.draw_arrowhead(
                buffer,
                pn,
                direction,
                stroke.thickness,
                arrows.size,
                stroke.fill,
            );
        }
    }

    // =========================================================================
    //  Curves
    // =========================================================================

    /// Draws a quadratic or cubic Bézier curve.
    ///
    /// * `control_2`: If `None`, draws a Quadratic curve. If `Some`, draws a Cubic curve.
    pub fn draw_bezier(
        &mut self,
        buffer: &mut crate::render::cache::MeshData,
        start: Point,
        control_1: Point,
        control_2: Option<Point>,
        end: Point,
        stroke: ResolvedStroke,
    ) {
        if stroke.thickness < 0.1 {
            return;
        }

        let mut builder = Path::builder();

        // Convert iced Points to Lyon Points
        let start_lyon = lyon_tessellation::math::Point::new(start.x, start.y);
        let c1_lyon = lyon_tessellation::math::Point::new(control_1.x, control_1.y);
        let end_lyon = lyon_tessellation::math::Point::new(end.x, end.y);

        builder.begin(start_lyon);

        if let Some(c2) = control_2 {
            // Cubic Bézier
            let c2_lyon = lyon_tessellation::math::Point::new(c2.x, c2.y);
            builder.cubic_bezier_to(c1_lyon, c2_lyon, end_lyon);
        } else {
            // Quadratic Bézier
            builder.quadratic_bezier_to(c1_lyon, end_lyon);
        }

        builder.end(false); // Open path

        // Calculate dynamic tolerance based on quality setting
        // Higher quality = lower tolerance (more segments)
        let tolerance = 0.1 / self.quality.max(0.1);

        self.stroke_path(buffer, builder.build().iter(), stroke, tolerance);
    }

    /// Draws a smooth curve passing through all given points (Spline).
    ///
    /// Uses Catmull-Rom interpolation.
    /// * `points`: The data points the line must pass through.
    /// * `tension`: `0.0` for smooth curves, `1.0` for straight lines.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_spline(
        &mut self,
        buffer: &mut crate::render::cache::MeshData,
        points: &[Point],
        stroke: ResolvedStroke,
        tension: f32,
    ) {
        if stroke.thickness < 0.1 {
            return;
        }

        if points.len() < 2 {
            return;
        }

        let mut builder = Path::builder();
        let first = points[0];

        // Start the path at the first point
        builder.begin(lyon_tessellation::math::Point::new(first.x, first.y));

        for i in 0..points.len() - 1 {
            // Define the 4 points window: p0, p1, p2, p3
            // We are drawing the curve from p1 to p2.
            let p1 = points[i];
            let p2 = points[i + 1];

            // Handle start boundary: simpler to mirror p1 around p0, or just repeat p0
            let p0 = if i == 0 {
                // Virtual point before start: Extend the line backwards
                p1 - (p2 - p1)
            } else {
                points[i - 1]
            };

            // Handle end boundary
            let p3 = if i + 2 < points.len() {
                points[i + 2]
            } else {
                // Virtual point after end: Extend the line forwards
                p2 + (p2 - p1)
            };

            // Calculate Control Points
            let (c1, c2) = catmull_rom_to_bezier(p0, p1, p2, p3, tension);

            // Add Cubic Bezier segment to path
            builder.cubic_bezier_to(
                lyon_tessellation::math::Point::new(c1.x, c1.y),
                lyon_tessellation::math::Point::new(c2.x, c2.y),
                lyon_tessellation::math::Point::new(p2.x, p2.y),
            );
        }

        builder.end(false); // Open path

        let tolerance = 0.1 / self.quality.max(0.1);
        self.stroke_path(buffer, builder.build().iter(), stroke, tolerance);
    }

    // =========================================================================
    //  Arc
    // =========================================================================

    /// Draws a circular arc, ring sector, or donut sector.
    ///
    /// * `start_angle`, `end_angle`: In Radians.
    /// * `radius_inner`: If `0`, draws a pie slice. If `> 0`, draws a ring/donut.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_arc(
        &mut self,
        buffer: &mut crate::render::cache::MeshData,
        center_x: f32,
        center_y: f32,
        radius_inner: f32,
        radius_outer: f32,
        start_angle: f32,
        end_angle: f32,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    ) {
        let arc_length = (end_angle - start_angle).abs() * radius_outer;
        let segments = self.resolve_lod_custom(arc_length);

        let thickness = radius_outer - radius_inner;

        // If stroke is thicker than the arc itself, just fill the whole arc
        let is_consumed = stroke
            .as_ref()
            .is_some_and(|stroke| stroke.thickness >= thickness);

        if is_consumed {
            if let Some(stroke) = stroke {
                self.manual.draw_arc_strip(
                    buffer,
                    center_x,
                    center_y,
                    radius_inner,
                    radius_outer,
                    start_angle,
                    end_angle,
                    stroke.fill,
                    segments,
                );
            }
            return;
        }

        if let Some(color) = fill {
            let mut draw_in = radius_inner;
            let mut draw_out = radius_outer;

            // Bleed fix for antialiasing
            if stroke.is_some() {
                draw_out = (radius_outer - 0.5).max(draw_in);
                draw_in = (radius_inner + 0.5).min(draw_out);
            }

            if draw_out - draw_in > 0.1 {
                self.manual.draw_arc_strip(
                    buffer,
                    center_x,
                    center_y,
                    draw_in,
                    draw_out,
                    start_angle,
                    end_angle,
                    color,
                    segments,
                );
            }
        }

        if let Some(stroke) = stroke {
            let center = lyon_tessellation::math::Point::new(center_x, center_y);
            let stroke_inner = radius_inner + stroke.thickness / 2.0;
            let stroke_outer = radius_outer - stroke.thickness / 2.0;

            if stroke_outer <= stroke_inner {
                return;
            }

            let sweep = (end_angle - start_angle).abs();
            let is_full_circle = sweep >= std::f32::consts::TAU - 0.001;
            let mut builder = Path::builder();

            if is_full_circle {
                // Stroke the Outer Ring
                builder.begin(center + lyon_tessellation::math::Vector::new(stroke_outer, 0.0));
                let outer = lyon_tessellation::geom::Arc {
                    center,
                    radii: lyon_tessellation::math::Vector::new(stroke_outer, stroke_outer),
                    start_angle: lyon_tessellation::math::Angle::radians(0.0),
                    sweep_angle: lyon_tessellation::math::Angle::radians(std::f32::consts::TAU),
                    x_rotation: lyon_tessellation::math::Angle::radians(0.0),
                };
                outer.for_each_cubic_bezier(&mut |seg| {
                    builder.cubic_bezier_to(seg.ctrl1, seg.ctrl2, seg.to);
                });
                builder.close();

                // Stroke the Inner Ring (if it exists)
                if radius_inner > 0.5 {
                    builder.begin(center + lyon_tessellation::math::Vector::new(stroke_inner, 0.0));
                    let inner = lyon_tessellation::geom::Arc {
                        center,
                        radii: lyon_tessellation::math::Vector::new(stroke_inner, stroke_inner),
                        start_angle: lyon_tessellation::math::Angle::radians(0.0),
                        sweep_angle: lyon_tessellation::math::Angle::radians(std::f32::consts::TAU),
                        x_rotation: lyon_tessellation::math::Angle::radians(0.0),
                    };
                    inner.for_each_cubic_bezier(&mut |seg| {
                        builder.cubic_bezier_to(seg.ctrl1, seg.ctrl2, seg.to);
                    });
                    builder.close();
                }
            } else {
                // Complex Path: Donut Sector (Pie slice with a hole)
                let start_cos = start_angle.cos();
                let start_sin = start_angle.sin();
                let end_cos = end_angle.cos();
                let end_sin = end_angle.sin();
                let sweep_angle = lyon_tessellation::math::Angle::radians(end_angle - start_angle);

                if radius_inner < 0.5 {
                    // Pie Slice (No hole)
                    builder.begin(center);
                    builder.line_to(
                        center
                            + lyon_tessellation::math::Vector::new(start_cos, start_sin)
                                * stroke_outer,
                    );
                    let outer = lyon_tessellation::geom::Arc {
                        center,
                        radii: lyon_tessellation::math::Vector::new(stroke_outer, stroke_outer),
                        start_angle: lyon_tessellation::math::Angle::radians(start_angle),
                        sweep_angle,
                        x_rotation: lyon_tessellation::math::Angle::radians(0.0),
                    };
                    outer.for_each_cubic_bezier(&mut |seg| {
                        builder.cubic_bezier_to(seg.ctrl1, seg.ctrl2, seg.to);
                    });
                } else {
                    // Donut Sector
                    // 1. Move to Inner Start
                    builder.begin(
                        center
                            + lyon_tessellation::math::Vector::new(start_cos, start_sin)
                                * stroke_inner,
                    );
                    // 2. Line to Outer Start
                    builder.line_to(
                        center
                            + lyon_tessellation::math::Vector::new(start_cos, start_sin)
                                * stroke_outer,
                    );
                    // 3. Arc along Outer
                    let outer = lyon_tessellation::geom::Arc {
                        center,
                        radii: lyon_tessellation::math::Vector::new(stroke_outer, stroke_outer),
                        start_angle: lyon_tessellation::math::Angle::radians(start_angle),
                        sweep_angle,
                        x_rotation: lyon_tessellation::math::Angle::radians(0.0),
                    };
                    outer.for_each_cubic_bezier(&mut |seg| {
                        builder.cubic_bezier_to(seg.ctrl1, seg.ctrl2, seg.to);
                    });
                    // 4. Line to Inner End
                    builder.line_to(
                        center
                            + lyon_tessellation::math::Vector::new(end_cos, end_sin) * stroke_inner,
                    );
                    // 5. Arc along Inner (Backwards)
                    let inner = lyon_tessellation::geom::Arc {
                        center,
                        radii: lyon_tessellation::math::Vector::new(stroke_inner, stroke_inner),
                        start_angle: lyon_tessellation::math::Angle::radians(end_angle),
                        sweep_angle: lyon_tessellation::math::Angle::radians(
                            start_angle - end_angle,
                        ),
                        x_rotation: lyon_tessellation::math::Angle::radians(0.0),
                    };
                    inner.for_each_cubic_bezier(&mut |seg| {
                        builder.cubic_bezier_to(seg.ctrl1, seg.ctrl2, seg.to);
                    });
                }

                builder.close();
            }

            let tolerance = 0.2 / self.quality.max(0.1);
            self.stroke_path(buffer, builder.build().iter(), stroke, tolerance);
        }
    }

    // =========================================================================
    //  Area
    // =========================================================================

    /// Draws an arbitrary filled polygon.
    ///
    /// * **Convex Polygons:** Automatically detected and rendered using a fast fan.
    /// * **Concave Polygons:** Automatically detected and triangulated using Earcut.
    pub fn draw_area(
        &mut self,
        buffer: &mut crate::render::cache::MeshData,
        points: &[Point],
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    ) {
        if points.len() < 3 {
            return;
        }

        if let Some(color) = fill {
            if is_convex(points) {
                if stroke.is_some() {
                    let inset_polygon = compute_inset_polygon(points, 0.5);
                    self.manual.draw_fan(buffer, &inset_polygon, color);
                } else {
                    self.manual.draw_fan(buffer, points, color);
                }
            } else {
                // Fallback: Use Earcut for concave triangulation
                let flat_coords: Vec<f64> = points
                    .iter()
                    .flat_map(|p| [p.x as f64, p.y as f64])
                    .collect();
                if let Ok(indices) = earcutr::earcut(&flat_coords, &[], 2) {
                    let packed = pack(color);
                    let mesh_indices: Vec<u32> = indices.iter().map(|&i| i as u32).collect();
                    let mesh_vertices: Vec<SolidVertex2D> = points
                        .iter()
                        .map(|p| SolidVertex2D {
                            position: [p.x, p.y],
                            color: packed,
                        })
                        .collect();
                    buffer.add(&mesh_indices, &mesh_vertices);
                }
            }
        }

        if let Some(stroke) = stroke {
            match stroke.style {
                StrokeStyle::Solid => {
                    if is_convex(points) {
                        // Fast path: Stroke ring for convex shapes
                        let inner_polygon = compute_inset_polygon(points, stroke.thickness);
                        self.manual
                            .draw_ring(buffer, points, &inner_polygon, stroke.fill);
                    } else {
                        // Fallback: Path stroking for concave shapes
                        let lyon_points = points
                            .iter()
                            .map(|p| lyon_tessellation::math::Point::new(p.x, p.y));
                        self.stroke_polyline(buffer, lyon_points, stroke, true);
                    }
                }
                _ => {
                    let lyon_points = points
                        .iter()
                        .map(|p| lyon_tessellation::math::Point::new(p.x, p.y));
                    self.stroke_polyline(buffer, lyon_points, stroke, true);
                }
            }
        }
    }

    // =========================================================================
    //  Adapters
    // =========================================================================

    /// Internal adapter to bridge the gap between `iced_aksel` logic and `lyon`'s
    /// advanced path stroking engine.
    fn stroke_polyline<I>(
        &mut self,
        buffer: &mut crate::render::cache::MeshData,
        points: I,
        stroke: ResolvedStroke,
        close_path: bool,
    ) where
        I: IntoIterator<Item = lyon_tessellation::math::Point>,
    {
        let options = StrokeOptions::default()
            .with_line_width(stroke.thickness)
            .with_line_cap(LineCap::Butt)
            .with_line_join(LineJoin::Miter);
        let mesh = buffer.get_mesh_mut();
        let mut writer = LyonAdapter::new(
            mesh,
            SolidVertexConstructor {
                color: pack(stroke.fill),
            },
        );
        let _ = match &stroke.style {
            StrokeStyle::Solid => self.complex.stroke.tessellate(
                FromPolyline::new(close_path, points.into_iter()),
                &options,
                &mut writer,
            ),
            StrokeStyle::Dashed { dash, gap } => {
                let dashes = [*dash, *gap];
                let dashed = DashedPolyline::new(points.into_iter(), &dashes);
                self.complex
                    .stroke
                    .tessellate(dashed, &options, &mut writer)
            }
            StrokeStyle::Dotted { gap } => {
                let dots = [stroke.thickness, *gap];
                let dashed = DashedPolyline::new(points.into_iter(), &dots);
                self.complex
                    .stroke
                    .tessellate(dashed, &options, &mut writer)
            }
        };
    }

    fn stroke_path<Iter>(
        &mut self,
        buffer: &mut crate::render::cache::MeshData,
        path: Iter,
        stroke: ResolvedStroke,
        tolerance: f32,
    ) where
        Iter: PathIterator,
    {
        let points: Vec<lyon_tessellation::math::Point> = path
            .flattened(tolerance)
            .filter_map(|evt| match evt {
                PathEvent::Begin { at } => Some(at),
                PathEvent::Line { to, .. } => Some(to),
                _ => None,
            })
            .collect();
        self.stroke_polyline(buffer, points, stroke, false);
    }

    // =========================================================================
    //  Text (Vector)
    // =========================================================================

    /// Draws text as a vector mesh using the tessellation engine.
    ///
    /// Unlike standard text, this method generates raw geometry (triangles) which can be
    /// freely rotated and scaled without losing quality.
    pub fn draw_text(&mut self, mesh_buffer: &mut crate::render::cache::MeshData, text: Text) {
        // Construct the context to hold heavy resources (caches, buffers)
        let ctx = &mut TextRenderContext {
            mesh_buffer,
            tessellator: &mut self.complex.fill,
            glyph_cache: &mut self.glyph_cache,
            scratch_geometry: &mut self.scratch_geometry,
            quality_multiplier: self.quality,
        };

        // Delegate to the text engine
        text::draw_geometric_text(ctx, text);
    }

    // =========================================================================
    //  Internal Helpers
    // =========================================================================

    fn resolve_lod(&self, radius: f32) -> usize {
        let raw = radius * 2.0 * self.quality;
        raw.clamp(24.0, 128.0) as usize
    }

    fn resolve_lod_custom(&self, length: f32) -> usize {
        let raw = length * 0.2 * self.quality;
        raw.clamp(4.0, 128.0) as usize
    }
}
