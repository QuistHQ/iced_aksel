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
/// It also manages the global [`quality`](Self::quality) setting, automatically calculating the
/// Level of Detail (LOD) for curves and arcs to balance performance with visual fidelity.
///
/// # Layman's Terms
/// This component serves as the **decision maker**. It takes a high-level request like "draw a circle"
/// and translates it into low-level instructions. It handles the "smart" work—like determining that
/// a circle needs fewer edges when zoomed out—before handing off the actual work of writing
/// coordinates to the underlying tessellators.
pub struct Tessellator {
    pub complex: ComplexTessellator,
    pub manual: manual::ManualTessellator,
    pub quality: f32,
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
    // =========================================================================
    //  Primitives
    // =========================================================================

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
        let is_consumed = if let Some((_, th_x, th_y)) = stroke {
            th_x >= width * 0.5 || th_y >= height * 0.5
        } else {
            false
        };

        if is_consumed {
            if let Some((s, _, _)) = stroke {
                self.manual
                    .draw_fill_rect(buffer, x_min, y_min, x_max, y_max, s.fill);
            }
            return;
        }

        if let Some(color) = fill {
            let d = if stroke.is_some() && width > 1.0 && height > 1.0 {
                0.5
            } else {
                0.0
            };
            self.manual
                .draw_fill_rect(buffer, x_min + d, y_min + d, x_max - d, y_max - d, color);
        }

        if let Some((s, th_x, th_y)) = stroke {
            match s.style {
                StrokeStyle::Solid => {
                    self.manual
                        .draw_stroke_rect(buffer, x_min, y_min, x_max, y_max, th_x, th_y, s.fill);
                }
                _ => {
                    let off = (th_x + th_y) / 4.0;
                    let pts = vec![
                        lyon_tessellation::math::Point::new(x_min + off, y_min + off),
                        lyon_tessellation::math::Point::new(x_max - off, y_min + off),
                        lyon_tessellation::math::Point::new(x_max - off, y_max - off),
                        lyon_tessellation::math::Point::new(x_min + off, y_max - off),
                        lyon_tessellation::math::Point::new(x_min + off, y_min + off),
                    ];
                    self.stroke_polyline(buffer, pts, s, (th_x + th_y) / 2.0, true);
                }
            }
        }
    }

    pub fn draw_circle<D>(
        &mut self,
        buffer: &mut MeshBuffer,
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
        fill: Option<Color>,
        stroke: Option<(&Stroke<D>, f32)>,
    ) {
        let max_r = rx.max(ry);
        if max_r < 0.5 {
            return;
        }
        let segments = self.resolve_lod(max_r);

        let is_consumed = if let Some((_, width)) = stroke {
            width >= max_r
        } else {
            false
        };
        if is_consumed {
            if let Some((s, _)) = stroke {
                self.manual
                    .draw_fill_circle(buffer, cx, cy, rx, ry, s.fill, segments);
            }
            return;
        }

        if let Some(color) = fill {
            let d = if stroke.is_some() { 0.5 } else { 0.0 };
            let fill_rx = (rx - d).max(0.0);
            let fill_ry = (ry - d).max(0.0);
            if fill_rx > 0.1 && fill_ry > 0.1 {
                self.manual
                    .draw_fill_circle(buffer, cx, cy, fill_rx, fill_ry, color, segments);
            }
        }

        if let Some((s, width)) = stroke {
            match s.style {
                StrokeStyle::Solid => {
                    let rx_inner = rx - width;
                    let ry_inner = ry - width;
                    self.manual.draw_stroke_circle(
                        buffer, cx, cy, rx_inner, ry_inner, rx, ry, s.fill, segments,
                    );
                }
                _ => {
                    // Elliptical stroking with Lyon for dashed lines
                    let stroke_rx = rx - (width / 2.0);
                    let stroke_ry = ry - (width / 2.0);
                    if stroke_rx > 0.1 && stroke_ry > 0.1 {
                        use lyon_tessellation::geom::Arc;
                        use lyon_tessellation::math::{Angle, Point, Vector};
                        let arc = Arc {
                            center: Point::new(cx, cy),
                            radii: Vector::new(stroke_rx, stroke_ry),
                            start_angle: Angle::radians(0.0),
                            sweep_angle: Angle::radians(std::f32::consts::TAU),
                            x_rotation: Angle::radians(0.0),
                        };
                        let tolerance = 0.2 / self.quality.max(0.1);
                        self.stroke_polyline(buffer, arc.flattened(tolerance), s, width, true);
                    }
                }
            }
        }
    }

    pub fn draw_triangle<D>(
        &mut self,
        buffer: &mut MeshBuffer,
        p1: Point,
        p2: Point,
        p3: Point,
        fill: Option<Color>,
        stroke: Option<(&Stroke<D>, f32)>,
    ) {
        let cross = (p2.x - p1.x).mul_add(p3.y - p1.y, -((p2.y - p1.y) * (p3.x - p1.x)));
        let (p1, p2, p3, double_area) = if cross < 0.0 {
            (p1, p3, p2, -cross)
        } else {
            (p1, p2, p3, cross)
        };

        let (inner_p1, inner_p2, inner_p3, is_consumed) = if let Some((_, width)) = stroke {
            let d1 = p1.distance(p2);
            let d2 = p2.distance(p3);
            let d3 = p3.distance(p1);
            let perimeter = d1 + d2 + d3;
            if perimeter < 1e-4 {
                (Point::ORIGIN, Point::ORIGIN, Point::ORIGIN, true)
            } else {
                let inradius = double_area / perimeter;
                if width >= inradius {
                    (Point::ORIGIN, Point::ORIGIN, Point::ORIGIN, true)
                } else {
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
            if let Some((s, _)) = stroke {
                self.manual.draw_fill_triangle(buffer, p1, p2, p3, s.fill);
            }
            return;
        }

        if let Some(color) = fill {
            if stroke.is_some() {
                let d = 0.5;
                let f1 = compute_inset_vertex(p3, p1, p2, d);
                let f2 = compute_inset_vertex(p1, p2, p3, d);
                let f3 = compute_inset_vertex(p2, p3, p1, d);
                self.manual.draw_fill_triangle(buffer, f1, f2, f3, color);
            } else {
                self.manual.draw_fill_triangle(buffer, p1, p2, p3, color);
            }
        }

        if let Some((s, width)) = stroke {
            match s.style {
                StrokeStyle::Solid => {
                    self.manual.draw_stroke_triangle(
                        buffer,
                        [p1, p2, p3],
                        [inner_p1, inner_p2, inner_p3],
                        s.fill,
                    );
                }
                _ => {
                    let d = width / 2.0;
                    let c1 = compute_inset_vertex(p3, p1, p2, d);
                    let c2 = compute_inset_vertex(p1, p2, p3, d);
                    let c3 = compute_inset_vertex(p2, p3, p1, d);
                    let points = vec![
                        lyon_tessellation::math::Point::new(c1.x, c1.y),
                        lyon_tessellation::math::Point::new(c2.x, c2.y),
                        lyon_tessellation::math::Point::new(c3.x, c3.y),
                        lyon_tessellation::math::Point::new(c1.x, c1.y),
                    ];
                    self.stroke_polyline(buffer, points, s, width, true);
                }
            }
        }
    }

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

        if let Some((s, width)) = stroke {
            match s.style {
                StrokeStyle::Solid => {
                    let inner_r = (radius - width).max(0.0);
                    let inner_points = generate_ring(center, inner_r, vertices, rotation);
                    self.manual
                        .draw_ring(buffer, &outer_points, &inner_points, s.fill);
                }
                _ => {
                    let lyon_pts: Vec<_> = outer_points
                        .iter()
                        .map(|p| lyon_tessellation::math::Point::new(p.x, p.y))
                        .collect();
                    self.stroke_polyline(buffer, lyon_pts, s, width, true);
                }
            }
        }
    }

    // =========================================================================
    //  Line Connectors
    // =========================================================================

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
        let dir_vec = raw_end - raw_start;
        if (dir_vec.x * dir_vec.x + dir_vec.y * dir_vec.y) < 0.001 {
            return;
        }
        let dir = normalize(dir_vec);
        let arrow_len = width * arrows.2;

        let mut line_start = raw_start;
        let mut line_end = raw_end;

        if arrows.0 && !extensions.0 {
            line_start = raw_start + dir * arrow_len;
        }
        if arrows.1 && !extensions.1 {
            line_end = raw_end - dir * arrow_len;
        }

        let check_vec = line_end - line_start;
        let valid = (check_vec.x * dir.x + check_vec.y * dir.y) > 0.0;

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
                    let pts = vec![
                        lyon_tessellation::math::Point::new(draw_start.x, draw_start.y),
                        lyon_tessellation::math::Point::new(draw_end.x, draw_end.y),
                    ];
                    self.stroke_polyline(buffer, pts, stroke, width, false);
                }
            }
        }

        if arrows.0 && !extensions.0 {
            self.manual
                .draw_arrowhead(buffer, raw_start, -dir, width, arrows.2, stroke.fill);
        }
        if arrows.1 && !extensions.1 {
            self.manual
                .draw_arrowhead(buffer, raw_end, dir, width, arrows.2, stroke.fill);
        }
    }

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
        if !extensions.0 && !extensions.1 && !arrows.0 && !arrows.1 {
            let lyon_pts = points
                .into_iter()
                .map(|p| lyon_tessellation::math::Point::new(p.x, p.y));
            self.stroke_polyline(buffer, lyon_pts, stroke, width, false);
            return;
        }

        let mut pts: Vec<Point> = points.into_iter().collect();
        if pts.len() < 2 {
            return;
        }

        let margin = width * arrows.2.max(2.0);
        let clip_rect = (
            clip_bounds.x - margin,
            clip_bounds.y - margin,
            clip_bounds.x + clip_bounds.width + margin,
            clip_bounds.y + clip_bounds.height + margin,
        );
        let last = pts.len() - 1;
        let p0 = pts[0];
        let pn = pts[last];

        if extensions.0 {
            let p1 = pts[1];
            if let Some((t0, _)) = clip_line_liang_barsky(p0, p1, clip_rect) {
                if t0 < 0.0 {
                    pts[0] = p0 + (p1 - p0) * t0;
                }
            }
        } else if arrows.0 {
            let p1 = pts[1];
            let dir = normalize(p1 - p0);
            pts[0] = p0 + dir * (width * arrows.2);
        }

        if extensions.1 {
            let pn_1 = pts[last - 1];
            if let Some((_, t1)) = clip_line_liang_barsky(pn_1, pn, clip_rect) {
                if t1 > 1.0 {
                    pts[last] = pn_1 + (pn - pn_1) * t1;
                }
            }
        } else if arrows.1 {
            let pn_1 = pts[last - 1];
            let dir = normalize(pn - pn_1);
            pts[last] = pn - dir * (width * arrows.2);
        }

        let lyon_pts = pts
            .iter()
            .map(|p| lyon_tessellation::math::Point::new(p.x, p.y));
        self.stroke_polyline(buffer, lyon_pts, stroke, width, false);

        if arrows.0 && !extensions.0 {
            let dir = normalize(pts[1] - p0);
            self.manual
                .draw_arrowhead(buffer, p0, -dir, width, arrows.2, stroke.fill);
        }
        if arrows.1 && !extensions.1 {
            let pn_1 = pts[last - 1];
            let dir = normalize(pn - pn_1);
            self.manual
                .draw_arrowhead(buffer, pn, dir, width, arrows.2, stroke.fill);
        }
    }

    // =========================================================================
    //  Arc (NEW)
    // =========================================================================

    #[allow(clippy::too_many_arguments)]
    pub fn draw_arc<D>(
        &mut self,
        buffer: &mut MeshBuffer,
        cx: f32,
        cy: f32,
        r_inner: f32,
        r_outer: f32,
        start_angle: f32,
        end_angle: f32,
        fill: Option<Color>,
        stroke: Option<(&Stroke<D>, f32)>,
    ) {
        if r_outer < 0.5 {
            return;
        }

        let arc_len = (end_angle - start_angle).abs() * r_outer;
        let segments = self.resolve_lod_custom(arc_len);

        let thickness = r_outer - r_inner;
        let is_consumed = if let Some((_, width)) = stroke {
            width >= thickness
        } else {
            false
        };

        if is_consumed {
            if let Some((s, _)) = stroke {
                self.manual.draw_arc_strip(
                    buffer,
                    cx,
                    cy,
                    r_inner,
                    r_outer,
                    start_angle,
                    end_angle,
                    s.fill,
                    segments,
                );
            }
            return;
        }

        if let Some(color) = fill {
            let mut draw_in = r_inner;
            let mut draw_out = r_outer;
            if stroke.is_some() {
                draw_out = (r_outer - 0.5).max(draw_in);
                draw_in = (r_inner + 0.5).min(draw_out);
            }
            if draw_out - draw_in > 0.1 {
                self.manual.draw_arc_strip(
                    buffer,
                    cx,
                    cy,
                    draw_in,
                    draw_out,
                    start_angle,
                    end_angle,
                    color,
                    segments,
                );
            }
        }

        if let Some((s, width)) = stroke {
            let center = lyon_tessellation::math::Point::new(cx, cy);
            let s_inner = r_inner + width / 2.0;
            let s_outer = r_outer - width / 2.0;
            if s_outer <= s_inner {
                return;
            }

            let sweep = (end_angle - start_angle).abs();
            let is_full_circle = sweep >= std::f32::consts::TAU - 0.001;
            let mut builder = Path::builder();

            if is_full_circle {
                builder.begin(center + lyon_tessellation::math::Vector::new(s_outer, 0.0));
                let outer = lyon_tessellation::geom::Arc {
                    center,
                    radii: lyon_tessellation::math::Vector::new(s_outer, s_outer),
                    start_angle: lyon_tessellation::math::Angle::radians(0.0),
                    sweep_angle: lyon_tessellation::math::Angle::radians(std::f32::consts::TAU),
                    x_rotation: lyon_tessellation::math::Angle::radians(0.0),
                };
                outer.for_each_cubic_bezier(&mut |seg| {
                    builder.cubic_bezier_to(seg.ctrl1, seg.ctrl2, seg.to);
                });
                builder.close();
                if r_inner > 0.5 {
                    builder.begin(center + lyon_tessellation::math::Vector::new(s_inner, 0.0));
                    let inner = lyon_tessellation::geom::Arc {
                        center,
                        radii: lyon_tessellation::math::Vector::new(s_inner, s_inner),
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
                let start_cos = start_angle.cos();
                let start_sin = start_angle.sin();
                let end_cos = end_angle.cos();
                let end_sin = end_angle.sin();
                let sweep_a = lyon_tessellation::math::Angle::radians(end_angle - start_angle);

                if r_inner < 0.5 {
                    builder.begin(center);
                    builder.line_to(
                        center
                            + lyon_tessellation::math::Vector::new(start_cos, start_sin) * s_outer,
                    );
                    let outer = lyon_tessellation::geom::Arc {
                        center,
                        radii: lyon_tessellation::math::Vector::new(s_outer, s_outer),
                        start_angle: lyon_tessellation::math::Angle::radians(start_angle),
                        sweep_angle: sweep_a,
                        x_rotation: lyon_tessellation::math::Angle::radians(0.0),
                    };
                    outer.for_each_cubic_bezier(&mut |seg| {
                        builder.cubic_bezier_to(seg.ctrl1, seg.ctrl2, seg.to);
                    });
                    builder.close();
                } else {
                    builder.begin(
                        center
                            + lyon_tessellation::math::Vector::new(start_cos, start_sin) * s_inner,
                    );
                    builder.line_to(
                        center
                            + lyon_tessellation::math::Vector::new(start_cos, start_sin) * s_outer,
                    );
                    let outer = lyon_tessellation::geom::Arc {
                        center,
                        radii: lyon_tessellation::math::Vector::new(s_outer, s_outer),
                        start_angle: lyon_tessellation::math::Angle::radians(start_angle),
                        sweep_angle: sweep_a,
                        x_rotation: lyon_tessellation::math::Angle::radians(0.0),
                    };
                    outer.for_each_cubic_bezier(&mut |seg| {
                        builder.cubic_bezier_to(seg.ctrl1, seg.ctrl2, seg.to);
                    });
                    builder.line_to(
                        center + lyon_tessellation::math::Vector::new(end_cos, end_sin) * s_inner,
                    );
                    let inner = lyon_tessellation::geom::Arc {
                        center,
                        radii: lyon_tessellation::math::Vector::new(s_inner, s_inner),
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
            self.stroke_path(buffer, builder.build().iter(), s, width, tolerance);
        }
    }

    // =========================================================================
    //  Zone (NEW)
    // =========================================================================

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
                    let inset = compute_inset_polygon(points, 0.5);
                    self.manual.draw_fan(buffer, &inset, color);
                } else {
                    self.manual.draw_fan(buffer, points, color);
                }
            } else {
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

        if let Some((s, width)) = stroke {
            match s.style {
                StrokeStyle::Solid => {
                    if is_convex(points) {
                        let inner = compute_inset_polygon(points, width);
                        self.manual.draw_ring(buffer, points, &inner, s.fill);
                    } else {
                        let l_pts = points
                            .iter()
                            .map(|p| lyon_tessellation::math::Point::new(p.x, p.y));
                        self.stroke_polyline(buffer, l_pts, s, width, true);
                    }
                }
                _ => {
                    let l_pts = points
                        .iter()
                        .map(|p| lyon_tessellation::math::Point::new(p.x, p.y));
                    self.stroke_polyline(buffer, l_pts, s, width, true);
                }
            }
        }
    }

    // =========================================================================
    //  Adapters
    // =========================================================================

    pub fn stroke_polyline<I, D>(
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

    pub fn stroke_path<Iter, D>(
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

    pub fn fill_polygon<I>(&mut self, buffer: &mut MeshBuffer, points: I, color: Color)
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
