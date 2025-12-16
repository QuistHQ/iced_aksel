pub mod complex;
pub mod manual;
pub mod math;

use crate::{Stroke, render::MeshBuffer, stroke::StrokeStyle};
use complex::{ComplexTessellator, DashedPolyline, LyonAdapter, SolidVertexConstructor};
use iced_core::{Color, Point, Rectangle};
use iced_graphics::color::pack;
use lyon_path::{LineCap, LineJoin, Path, PathEvent, iterator::FromPolyline, traits::PathIterator};
use lyon_tessellation::{FillOptions, StrokeOptions};
use math::*;

/// The central driver for the rendering engine.
///
/// The `Tessellator` acts as the "Brain" of the graphics pipeline. It orchestrates the
/// tessellation process by deciding which strategy to use for a given shape:
///
/// * **Fast Path (Manual):** Direct vertex generation for simple primitives (Rects, Circles, Lines).
/// * **Robust Path (Complex):** Lyon-based tessellation for dashed lines, complex paths, and boolean operations.
///
/// It encapsulates the underlying sub-tessellators to ensure consistent state and optimizations.
pub struct Tessellator {
    complex: ComplexTessellator,
    manual: manual::ManualTessellator,
    /// Global quality multiplier.
    /// * 1.0 = Standard (Default).
    /// * 0.5 = High Performance (Lower vertex count for curves).
    /// * 2.0 = High Quality (Smoother curves).
    quality: f32,
}

impl Default for Tessellator {
    fn default() -> Self {
        Self {
            complex: ComplexTessellator::default(),
            manual: manual::ManualTessellator::default(),
            quality: 1.0,
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
    /// This value controls the Level of Detail (LOD) for curves, arcs, and circles.
    /// * `1.0` is the default standard.
    /// * Values `< 1.0` reduce vertex count for higher performance.
    /// * Values `> 1.0` increase vertex count for smoother visuals.
    ///
    /// The value is automatically clamped between `0.1` and `5.0`.
    pub fn set_quality(&mut self, quality: f32) {
        self.quality = quality.clamp(0.1, 5.0);
    }

    /// Returns the current quality multiplier.
    pub fn quality(&self) -> f32 {
        self.quality
    }

    // =========================================================================
    //  Primitives
    // =========================================================================

    /// Draws an axis-aligned rectangle.
    ///
    /// # optimization
    /// If the stroke thickness is large enough to cover the entire rectangle (e.g. thickness >= width),
    /// the engine will automatically switch to drawing a simple filled rectangle to save vertices.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_rectangle<D>(
        &mut self,
        buffer: &mut MeshBuffer,
        x_min: f32,
        y_min: f32,
        x_max: f32,
        y_max: f32,
        fill: Option<Color>,
        stroke: Option<(&Stroke<D>, f32, f32)>,
    ) {
        let width = x_max - x_min;
        let height = y_max - y_min;

        // Check if the stroke is so thick it completely swallows the shape.
        let is_consumed = stroke.is_some_and(|(_, thickness_x, thickness_y)| {
            thickness_x >= width / 2.0 || thickness_y >= height / 2.0
        });

        if is_consumed {
            if let Some((style, _, _)) = stroke {
                self.manual
                    .draw_fill_rect(buffer, x_min, y_min, x_max, y_max, style.fill);
            }
            return;
        }

        if let Some(color) = fill {
            // "Bleed fix": Overlap fill and stroke slightly to prevent sub-pixel gaps (anti-aliasing artifacts)
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
            );
        }

        if let Some((style, thickness_x, thickness_y)) = stroke {
            match style.style {
                StrokeStyle::Solid => {
                    self.manual.draw_stroke_rect(
                        buffer,
                        x_min,
                        y_min,
                        x_max,
                        y_max,
                        thickness_x,
                        thickness_y,
                        style.fill,
                    );
                }
                _ => {
                    // For dashed/dotted lines, we must construct a path for Lyon to process
                    let offset = (thickness_x + thickness_y) / 4.0;
                    let points = vec![
                        lyon_tessellation::math::Point::new(x_min + offset, y_min + offset),
                        lyon_tessellation::math::Point::new(x_max - offset, y_min + offset),
                        lyon_tessellation::math::Point::new(x_max - offset, y_max - offset),
                        lyon_tessellation::math::Point::new(x_min + offset, y_max - offset),
                        lyon_tessellation::math::Point::new(x_min + offset, y_min + offset),
                    ];
                    self.stroke_polyline(
                        buffer,
                        points,
                        style,
                        (thickness_x + thickness_y) / 2.0,
                        true,
                    );
                }
            }
        }
    }

    /// Draws a circle or ellipse.
    ///
    /// * `center_x`, `center_y`: The center coordinates in screen space.
    /// * `radius_x`, `radius_y`: The radii. If equal, draws a circle. If different, draws an ellipse.
    ///
    /// # Performance
    /// This method automatically adjusts the number of segments (vertices) used to approximate the curve
    /// based on the radius and the current [`quality`](Self::quality) setting.
    pub fn draw_circle<D>(
        &mut self,
        buffer: &mut MeshBuffer,
        center_x: f32,
        center_y: f32,
        radius_x: f32,
        radius_y: f32,
        fill: Option<Color>,
        stroke: Option<(&Stroke<D>, f32)>,
    ) {
        let max_radius = radius_x.max(radius_y);

        // Culling: Too small to be seen
        if max_radius < 0.5 {
            return;
        }

        let segments = self.resolve_lod(max_radius);

        let is_consumed = if let Some((_, width)) = stroke {
            width >= max_radius
        } else {
            false
        };

        if is_consumed {
            if let Some((style, _)) = stroke {
                self.manual.draw_fill_circle(
                    buffer, center_x, center_y, radius_x, radius_y, style.fill, segments,
                );
            }
            return;
        }

        if let Some(color) = fill {
            let overlap = if stroke.is_some() { 0.5 } else { 0.0 };
            let fill_radius_x = (radius_x - overlap).max(0.0);
            let fill_radius_y = (radius_y - overlap).max(0.0);

            if fill_radius_x > 0.1 && fill_radius_y > 0.1 {
                self.manual.draw_fill_circle(
                    buffer,
                    center_x,
                    center_y,
                    fill_radius_x,
                    fill_radius_y,
                    color,
                    segments,
                );
            }
        }

        if let Some((style, width)) = stroke {
            match style.style {
                StrokeStyle::Solid => {
                    let radius_x_inner = radius_x - width;
                    let radius_y_inner = radius_y - width;
                    self.manual.draw_stroke_circle(
                        buffer,
                        center_x,
                        center_y,
                        radius_x_inner,
                        radius_y_inner,
                        radius_x,
                        radius_y,
                        style.fill,
                        segments,
                    );
                }
                _ => {
                    // Elliptical stroking with Lyon for dashed lines
                    let stroke_radius_x = radius_x - (width / 2.0);
                    let stroke_radius_y = radius_y - (width / 2.0);

                    if stroke_radius_x > 0.1 && stroke_radius_y > 0.1 {
                        use lyon_tessellation::geom::Arc;
                        use lyon_tessellation::math::{Angle, Point, Vector};

                        let arc = Arc {
                            center: Point::new(center_x, center_y),
                            radii: Vector::new(stroke_radius_x, stroke_radius_y),
                            start_angle: Angle::radians(0.0),
                            sweep_angle: Angle::radians(std::f32::consts::TAU),
                            x_rotation: Angle::radians(0.0),
                        };

                        let tolerance = 0.2 / self.quality.max(0.1);
                        self.stroke_polyline(buffer, arc.flattened(tolerance), style, width, true);
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
    pub fn draw_triangle<D>(
        &mut self,
        buffer: &mut MeshBuffer,
        p1: Point,
        p2: Point,
        p3: Point,
        fill: Option<Color>,
        stroke: Option<(&Stroke<D>, f32)>,
    ) {
        // Compute cross product to check winding order (Clockwise vs Counter-Clockwise)
        let cross_product = (p2.x - p1.x).mul_add(p3.y - p1.y, -((p2.y - p1.y) * (p3.x - p1.x)));

        // Ensure consistent winding order for easier math downstream
        let (p1, p2, p3, double_area) = if cross_product < 0.0 {
            (p1, p3, p2, -cross_product)
        } else {
            (p1, p2, p3, cross_product)
        };

        let (inner_p1, inner_p2, inner_p3, is_consumed) = if let Some((_, width)) = stroke {
            let dist_1 = p1.distance(p2);
            let dist_2 = p2.distance(p3);
            let dist_3 = p3.distance(p1);
            let perimeter = dist_1 + dist_2 + dist_3;

            if perimeter < 1e-4 {
                (Point::ORIGIN, Point::ORIGIN, Point::ORIGIN, true)
            } else {
                let inradius = double_area / perimeter;

                // If stroke is thicker than the inradius, the triangle is fully filled by the stroke
                if width >= inradius {
                    (Point::ORIGIN, Point::ORIGIN, Point::ORIGIN, true)
                } else {
                    // Compute the inner triangle for the "hole"
                    (
                        compute_inset_vertex(p3, p1, p2, width),
                        compute_inset_vertex(p1, p2, p3, width),
                        compute_inset_vertex(p2, p3, p1, width),
                        false,
                    )
                }
            }
        } else {
            (p1, p2, p3, false)
        };

        if is_consumed {
            if let Some((style, _)) = stroke {
                self.manual
                    .draw_fill_triangle(buffer, p1, p2, p3, style.fill);
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

        if let Some((style, width)) = stroke {
            match style.style {
                StrokeStyle::Solid => {
                    self.manual.draw_stroke_triangle(
                        buffer,
                        [p1, p2, p3],
                        [inner_p1, inner_p2, inner_p3],
                        style.fill,
                    );
                }
                _ => {
                    let offset = width / 2.0;
                    let center_p1 = compute_inset_vertex(p3, p1, p2, offset);
                    let center_p2 = compute_inset_vertex(p1, p2, p3, offset);
                    let center_p3 = compute_inset_vertex(p2, p3, p1, offset);

                    let points = vec![
                        lyon_tessellation::math::Point::new(center_p1.x, center_p1.y),
                        lyon_tessellation::math::Point::new(center_p2.x, center_p2.y),
                        lyon_tessellation::math::Point::new(center_p3.x, center_p3.y),
                        lyon_tessellation::math::Point::new(center_p1.x, center_p1.y),
                    ];
                    self.stroke_polyline(buffer, points, style, width, true);
                }
            }
        }
    }

    /// Draws a regular N-sided polygon (e.g., Hexagon, Octagon).
    ///
    /// * `vertices`: The number of sides (must be >= 3).
    /// * `rotation`: Rotation in degrees. 0.0 aligns the first vertex to the North.
    pub fn draw_polygon<D>(
        &mut self,
        buffer: &mut MeshBuffer,
        center: Point,
        radius: f32,
        vertices: u16,
        rotation: f32,
        fill: Option<Color>,
        stroke: Option<(&Stroke<D>, f32)>,
    ) {
        if vertices < 3 || radius < 0.5 {
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

        if let Some((style, width)) = stroke {
            match style.style {
                StrokeStyle::Solid => {
                    let inner_radius = (radius - width).max(0.0);
                    let inner_points = generate_ring(center, inner_radius, vertices, rotation);
                    self.manual
                        .draw_ring(buffer, &outer_points, &inner_points, style.fill);
                }
                _ => {
                    let lyon_points: Vec<_> = outer_points
                        .iter()
                        .map(|p| lyon_tessellation::math::Point::new(p.x, p.y))
                        .collect();
                    self.stroke_polyline(buffer, lyon_points, style, width, true);
                }
            }
        }
    }

    // =========================================================================
    //  Line Connectors
    // =========================================================================

    /// Draws a straight line segment.
    ///
    /// # Advanced Features
    /// * **Clipping:** This method performs Liang-Barsky clipping against the provided `clip_bounds` to
    ///   prevent drawing lines far outside the visible area, which saves GPU resources.
    /// * **Extensions:** Can calculate infinite extensions (e.g., for trend lines) within the clip bounds.
    /// * **Decorations:** Supports arrowheads at either end.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_line<D>(
        &mut self,
        buffer: &mut MeshBuffer,
        raw_start: Point,
        raw_end: Point,
        stroke: &Stroke<D>,
        width: f32,
        clip_bounds: Rectangle,
        extensions: (bool, bool),
        arrows: (bool, bool, f32),
    ) {
        if width < 0.1 {
            return;
        }

        let direction_vector = raw_end - raw_start;
        // Ignore zero-length lines
        if (direction_vector.x * direction_vector.x + direction_vector.y * direction_vector.y)
            < 0.001
        {
            return;
        }

        let direction = normalize(direction_vector);
        let arrow_length = width * arrows.2;

        let mut line_start = raw_start;
        let mut line_end = raw_end;

        // Retract line start/end if arrows are present so the arrow tip lands exactly on the point
        if arrows.0 && !extensions.0 {
            line_start = raw_start + direction * arrow_length;
        }
        if arrows.1 && !extensions.1 {
            line_end = raw_end - direction * arrow_length;
        }

        // Validity check: Did the arrows consume the entire line?
        let check_vector = line_end - line_start;
        let valid = (check_vector.x * direction.x + check_vector.y * direction.y) > 0.0;

        // Expand clip bounds slightly to avoid artifacts at the edges
        let margin = width * arrows.2.max(1.0);
        let clip_rect = (
            clip_bounds.x - margin,
            clip_bounds.y - margin,
            clip_bounds.x + clip_bounds.width + margin,
            clip_bounds.y + clip_bounds.height + margin,
        );

        let p1 = if extensions.0 { raw_start } else { line_start };
        let p2 = if extensions.1 { raw_end } else { line_end };

        let mut draw_start = line_start;
        let mut draw_end = line_end;
        let mut visible = true;

        // Perform Liang-Barsky clipping to find the visible segment of the line
        if let Some((t0, t1)) = clip_line_liang_barsky(p1, p2, clip_rect) {
            let delta = p2 - p1;
            draw_start = if extensions.0 {
                p1 + delta * t0
            } else if t0 > 0.0 {
                p1 + delta * t0
            } else {
                p1
            };
            draw_end = if extensions.1 {
                p1 + delta * t1
            } else if t1 < 1.0 {
                p1 + delta * t1
            } else {
                p2
            };
        } else {
            visible = false;
        }

        if visible && valid {
            match stroke.style {
                StrokeStyle::Solid => {
                    self.manual
                        .draw_line_segment(buffer, draw_start, draw_end, width, stroke.fill);
                }
                _ => {
                    let points = vec![
                        lyon_tessellation::math::Point::new(draw_start.x, draw_start.y),
                        lyon_tessellation::math::Point::new(draw_end.x, draw_end.y),
                    ];
                    self.stroke_polyline(buffer, points, stroke, width, false);
                }
            }
        }

        // Draw arrowheads
        if arrows.0 && !extensions.0 {
            self.manual
                .draw_arrowhead(buffer, raw_start, -direction, width, arrows.2, stroke.fill);
        }
        if arrows.1 && !extensions.1 {
            self.manual
                .draw_arrowhead(buffer, raw_end, direction, width, arrows.2, stroke.fill);
        }
    }

    /// Draws a connected series of lines (a path).
    ///
    /// Supports dashed lines (via `StrokeStyle`), infinite extensions on the first/last segments,
    /// and arrowheads at the ends.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_polyline<I, D>(
        &mut self,
        buffer: &mut MeshBuffer,
        points: I,
        stroke: &Stroke<D>,
        width: f32,
        clip_bounds: Rectangle,
        extensions: (bool, bool),
        arrows: (bool, bool, f32),
    ) where
        I: IntoIterator<Item = Point>,
    {
        if width < 0.1 {
            return;
        }

        // Fast Path: No special decorations or extensions, just stroke it
        if !extensions.0 && !extensions.1 && !arrows.0 && !arrows.1 {
            let lyon_points = points
                .into_iter()
                .map(|p| lyon_tessellation::math::Point::new(p.x, p.y));
            self.stroke_polyline(buffer, lyon_points, stroke, width, false);
            return;
        }

        let mut point_list: Vec<Point> = points.into_iter().collect();
        if point_list.len() < 2 {
            return;
        }

        let margin = width * arrows.2.max(2.0);
        let clip_rect = (
            clip_bounds.x - margin,
            clip_bounds.y - margin,
            clip_bounds.x + clip_bounds.width + margin,
            clip_bounds.y + clip_bounds.height + margin,
        );

        let last_idx = point_list.len() - 1;
        let p0 = point_list[0];
        let pn = point_list[last_idx];

        // Handle Start Extension / Arrow retraction
        if extensions.0 {
            let p1 = point_list[1];
            if let Some((t0, _)) = clip_line_liang_barsky(p0, p1, clip_rect) {
                if t0 < 0.0 {
                    point_list[0] = p0 + (p1 - p0) * t0;
                }
            }
        } else if arrows.0 {
            let p1 = point_list[1];
            let direction = normalize(p1 - p0);
            point_list[0] = p0 + direction * (width * arrows.2);
        }

        // Handle End Extension / Arrow retraction
        if extensions.1 {
            let point_before_last = point_list[last_idx - 1];
            if let Some((_, t1)) = clip_line_liang_barsky(point_before_last, pn, clip_rect) {
                if t1 > 1.0 {
                    point_list[last_idx] = point_before_last + (pn - point_before_last) * t1;
                }
            }
        } else if arrows.1 {
            let point_before_last = point_list[last_idx - 1];
            let direction = normalize(pn - point_before_last);
            point_list[last_idx] = pn - direction * (width * arrows.2);
        }

        let lyon_points = point_list
            .iter()
            .map(|p| lyon_tessellation::math::Point::new(p.x, p.y));
        self.stroke_polyline(buffer, lyon_points, stroke, width, false);

        // Draw Arrowheads
        if arrows.0 && !extensions.0 {
            let direction = normalize(point_list[1] - p0);
            self.manual
                .draw_arrowhead(buffer, p0, -direction, width, arrows.2, stroke.fill);
        }
        if arrows.1 && !extensions.1 {
            let point_before_last = point_list[last_idx - 1];
            let direction = normalize(pn - point_before_last);
            self.manual
                .draw_arrowhead(buffer, pn, direction, width, arrows.2, stroke.fill);
        }
    }

    // =========================================================================
    //  Curves
    // =========================================================================

    /// Draws a quadratic or cubic Bézier curve.
    ///
    /// * `control_2`: If `None`, draws a Quadratic curve. If `Some`, draws a Cubic curve.
    pub fn draw_bezier<D>(
        &mut self,
        buffer: &mut MeshBuffer,
        start: Point,
        control_1: Point,
        control_2: Option<Point>,
        end: Point,
        stroke: &Stroke<D>,
        width: f32,
    ) {
        if width < 0.1 {
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

        self.stroke_path(buffer, builder.build().iter(), stroke, width, tolerance);
    }

    /// Draws a smooth curve passing through all given points (Spline).
    ///
    /// Uses Catmull-Rom interpolation.
    /// * `points`: The data points the line must pass through.
    /// * `tension`: `0.0` for smooth curves, `1.0` for straight lines.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_spline<I, D>(
        &mut self,
        buffer: &mut MeshBuffer,
        points: I,
        stroke: &Stroke<D>,
        width: f32,
        tension: f32,
    ) where
        I: IntoIterator<Item = Point>,
    {
        if width < 0.1 {
            return;
        }

        let pts: Vec<Point> = points.into_iter().collect();
        if pts.len() < 2 {
            return;
        }

        let mut builder = Path::builder();
        let first = pts[0];

        // Start the path at the first point
        builder.begin(lyon_tessellation::math::Point::new(first.x, first.y));

        for i in 0..pts.len() - 1 {
            // Define the 4 points window: p0, p1, p2, p3
            // We are drawing the curve from p1 to p2.
            let p1 = pts[i];
            let p2 = pts[i + 1];

            // Handle start boundary: simpler to mirror p1 around p0, or just repeat p0
            let p0 = if i == 0 {
                // Virtual point before start: Extend the line backwards
                p1 - (p2 - p1)
            } else {
                pts[i - 1]
            };

            // Handle end boundary
            let p3 = if i + 2 < pts.len() {
                pts[i + 2]
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
        self.stroke_path(buffer, builder.build().iter(), stroke, width, tolerance);
    }

    // =========================================================================
    //  Arc
    // =========================================================================

    /// Draws a circular arc, ring sector, or donut sector.
    ///
    /// * `start_angle`, `end_angle`: In Radians.
    /// * `radius_inner`: If `0`, draws a pie slice. If `> 0`, draws a ring/donut.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_arc<D>(
        &mut self,
        buffer: &mut MeshBuffer,
        center_x: f32,
        center_y: f32,
        radius_inner: f32,
        radius_outer: f32,
        start_angle: f32,
        end_angle: f32,
        fill: Option<Color>,
        stroke: Option<(&Stroke<D>, f32)>,
    ) {
        if radius_outer < 0.5 {
            return;
        }

        let arc_length = (end_angle - start_angle).abs() * radius_outer;
        let segments = self.resolve_lod_custom(arc_length);

        let thickness = radius_outer - radius_inner;

        // If stroke is thicker than the arc itself, just fill the whole arc
        let is_consumed = if let Some((_, width)) = stroke {
            width >= thickness
        } else {
            false
        };

        if is_consumed {
            if let Some((style, _)) = stroke {
                self.manual.draw_arc_strip(
                    buffer,
                    center_x,
                    center_y,
                    radius_inner,
                    radius_outer,
                    start_angle,
                    end_angle,
                    style.fill,
                    segments,
                );
            }
            return;
        }

        if let Some(color) = fill {
            let mut draw_in = radius_inner;
            let mut draw_out = radius_outer;

            // Bleed fix for anti-aliasing
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

        if let Some((style, width)) = stroke {
            let center = lyon_tessellation::math::Point::new(center_x, center_y);
            let stroke_inner = radius_inner + width / 2.0;
            let stroke_outer = radius_outer - width / 2.0;

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
                    builder.close();
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
                    builder.close();
                }
            }

            let tolerance = 0.2 / self.quality.max(0.1);
            self.stroke_path(buffer, builder.build().iter(), style, width, tolerance);
        }
    }

    // =========================================================================
    //  Area
    // =========================================================================

    /// Draws an arbitrary filled polygon.
    ///
    /// * **Convex Polygons:** Automatically detected and rendered using a fast fan.
    /// * **Concave Polygons:** Automatically detected and triangulated using Earcut.
    pub fn draw_zone<D>(
        &mut self,
        buffer: &mut MeshBuffer,
        points: &[Point],
        fill: Option<Color>,
        stroke: Option<(&Stroke<D>, f32)>,
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
                    let mesh_indices: Vec<u32> = indices.iter().map(|&i| i as u32).collect();
                    self.manual.draw_mesh(buffer, points, &mesh_indices, color);
                }
            }
        }

        if let Some((style, width)) = stroke {
            match style.style {
                StrokeStyle::Solid => {
                    if is_convex(points) {
                        // Fast path: Stroke ring for convex shapes
                        let inner_polygon = compute_inset_polygon(points, width);
                        self.manual
                            .draw_ring(buffer, points, &inner_polygon, style.fill);
                    } else {
                        // Fallback: Path stroking for concave shapes
                        let lyon_points = points
                            .iter()
                            .map(|p| lyon_tessellation::math::Point::new(p.x, p.y));
                        self.stroke_polyline(buffer, lyon_points, style, width, true);
                    }
                }
                _ => {
                    let lyon_points = points
                        .iter()
                        .map(|p| lyon_tessellation::math::Point::new(p.x, p.y));
                    self.stroke_polyline(buffer, lyon_points, style, width, true);
                }
            }
        }
    }

    // =========================================================================
    //  Adapters
    // =========================================================================

    /// Internal adapter to bridge the gap between `iced_aksel` logic and `lyon`'s
    /// advanced path stroking engine.
    fn stroke_polyline<I, D>(
        &mut self,
        buffer: &mut MeshBuffer,
        points: I,
        stroke: &Stroke<D>,
        resolved_width: f32,
        close_path: bool,
    ) where
        I: IntoIterator<Item = lyon_tessellation::math::Point>,
    {
        let options = StrokeOptions::default()
            .with_line_width(resolved_width)
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
            StrokeStyle::Dashed => {
                let dashes = [resolved_width * 5., resolved_width * 2.];
                let dashed = DashedPolyline::new(points.into_iter(), &dashes);
                self.complex
                    .stroke
                    .tessellate(dashed, &options, &mut writer)
            }
            StrokeStyle::Dotted => {
                let dots = [resolved_width, resolved_width * 2.0];
                let dashed = DashedPolyline::new(points.into_iter(), &dots);
                self.complex
                    .stroke
                    .tessellate(dashed, &options, &mut writer)
            }
        };
    }

    fn stroke_path<Iter, D>(
        &mut self,
        buffer: &mut MeshBuffer,
        path: Iter,
        stroke: &Stroke<D>,
        resolved_width: f32,
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
        self.stroke_polyline(buffer, points, stroke, resolved_width, true);
    }

    /// Internal adapter for filling complex polygons using Lyon.
    fn fill_polygon<I>(&mut self, buffer: &mut MeshBuffer, points: I, color: Color)
    where
        I: IntoIterator<Item = lyon_tessellation::math::Point>,
    {
        let options = FillOptions::default();
        let mesh = buffer.get_mesh_mut();
        let mut writer = LyonAdapter::new(mesh, SolidVertexConstructor { color: pack(color) });
        let _ = self.complex.fill.tessellate(
            FromPolyline::new(true, points.into_iter()),
            &options,
            &mut writer,
        );
    }

    fn resolve_lod(&self, radius: f32) -> usize {
        let raw = radius * 2.0 * self.quality;
        raw.clamp(24.0, 128.0) as usize
    }

    fn resolve_lod_custom(&self, length: f32) -> usize {
        let raw = length * 0.2 * self.quality;
        raw.clamp(4.0, 128.0) as usize
    }
}
