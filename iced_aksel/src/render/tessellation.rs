pub mod complex;
pub mod manual;

use crate::{Stroke, render::MeshBuffer, stroke::StrokeStyle};
use complex::{ComplexTessellator, DashedPolyline, LyonAdapter, SolidVertexConstructor};
use iced_core::{Color, Point, Vector};
use iced_graphics::color::pack;
use lyon_path::{LineCap, LineJoin, PathEvent, iterator::FromPolyline, traits::PathIterator};
use lyon_tessellation::{FillOptions, StrokeOptions};

/// The unified tessellator facade.
#[derive(Default)]
pub struct Tessellator {
    pub complex: ComplexTessellator,
    pub manual: manual::ManualTessellator,
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
        stroke: Option<(&Stroke<D>, f32, f32)>, // (stroke, width_x, width_y)
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
                StrokeStyle::Dashed | StrokeStyle::Dotted => {
                    let thickness = (th_x + th_y) / 2.0;
                    let offset = thickness / 2.0;
                    let points = vec![
                        lyon_tessellation::math::Point::new(x_min + offset, y_min + offset),
                        lyon_tessellation::math::Point::new(x_max - offset, y_min + offset),
                        lyon_tessellation::math::Point::new(x_max - offset, y_max - offset),
                        lyon_tessellation::math::Point::new(x_min + offset, y_max - offset),
                        lyon_tessellation::math::Point::new(x_min + offset, y_min + offset),
                    ];
                    self.stroke_polyline(buffer, points, s, thickness, true);
                }
            }
        }
    }

    pub fn draw_circle<D>(
        &mut self,
        buffer: &mut MeshBuffer,
        cx: f32,
        cy: f32,
        radius: f32,
        fill: Option<Color>,
        stroke: Option<(&Stroke<D>, f32)>,
    ) {
        if radius < 0.5 {
            return;
        }
        let is_consumed = if let Some((_, width)) = stroke {
            width >= radius
        } else {
            false
        };

        if is_consumed {
            if let Some((s, _)) = stroke {
                self.manual.draw_fill_circle(buffer, cx, cy, radius, s.fill);
            }
            return;
        }

        if let Some(color) = fill {
            let d = if stroke.is_some() { 0.5 } else { 0.0 };
            let fill_r = (radius - d).max(0.0);
            if fill_r > 0.1 {
                self.manual.draw_fill_circle(buffer, cx, cy, fill_r, color);
            }
        }

        if let Some((s, width)) = stroke {
            match s.style {
                StrokeStyle::Solid => {
                    let r_inner = radius - width;
                    self.manual
                        .draw_stroke_circle(buffer, cx, cy, r_inner, radius, s.fill);
                }
                StrokeStyle::Dashed | StrokeStyle::Dotted => {
                    let stroke_radius = radius - (width / 2.0);
                    if stroke_radius > 0.1 {
                        use lyon_tessellation::geom::Arc;
                        use lyon_tessellation::math::Angle;
                        let arc = Arc {
                            center: lyon_tessellation::math::Point::new(cx, cy),
                            radii: lyon_tessellation::math::Vector::new(
                                stroke_radius,
                                stroke_radius,
                            ),
                            start_angle: Angle::radians(0.0),
                            sweep_angle: Angle::radians(std::f32::consts::TAU),
                            x_rotation: Angle::radians(0.0),
                        };
                        self.stroke_polyline(buffer, arc.flattened(0.2), s, width, true);
                    }
                }
            }
        }
    }

    /// High-level primitive: Triangle (Using Iced Point)
    pub fn draw_triangle<D>(
        &mut self,
        buffer: &mut MeshBuffer,
        p1: Point,
        p2: Point,
        p3: Point,
        fill: Option<Color>,
        stroke: Option<(&Stroke<D>, f32)>,
    ) {
        // 1. Normalize Winding (CCW) & Calculate Area
        let cross = (p2.x - p1.x).mul_add(p3.y - p1.y, -((p2.y - p1.y) * (p3.x - p1.x)));
        let (p1, p2, p3, double_area) = if cross < 0.0 {
            (p1, p3, p2, -cross)
        } else {
            (p1, p2, p3, cross)
        };

        // 2. Pre-calculate Insets / Consumption Check
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

        // 3. Fast Path: Consumed by stroke
        if is_consumed {
            if let Some((s, _)) = stroke {
                self.manual.draw_fill_triangle(buffer, p1, p2, p3, s.fill);
            }
            return;
        }

        // 4. Draw Fill
        if let Some(color) = fill {
            if stroke.is_some() {
                // Bleed Fix
                let d = 0.5;
                let f1 = compute_inset_vertex(p3, p1, p2, d);
                let f2 = compute_inset_vertex(p1, p2, p3, d);
                let f3 = compute_inset_vertex(p2, p3, p1, d);
                self.manual.draw_fill_triangle(buffer, f1, f2, f3, color);
            } else {
                self.manual.draw_fill_triangle(buffer, p1, p2, p3, color);
            }
        }

        // 5. Draw Stroke
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
                StrokeStyle::Dashed | StrokeStyle::Dotted => {
                    // Convert Iced Points to Lyon Points for dashed path
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

    /// High-level primitive: Polygon (Using Iced Point)
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

        // Fill
        if let Some(color) = fill {
            if stroke.is_some() {
                let inset_points = generate_ring(center, radius - 0.5, vertices, rotation);
                self.manual.draw_fan(buffer, &inset_points, color);
            } else {
                self.manual.draw_fan(buffer, &outer_points, color);
            }
        }

        // Stroke
        if let Some((s, width)) = stroke {
            match s.style {
                StrokeStyle::Solid => {
                    let inner_r = (radius - width).max(0.0);
                    let inner_points = generate_ring(center, inner_r, vertices, rotation);
                    self.manual
                        .draw_ring(buffer, &outer_points, &inner_points, s.fill);
                }
                StrokeStyle::Dashed | StrokeStyle::Dotted => {
                    // Convert to Lyon points
                    let lyon_points: Vec<lyon_tessellation::math::Point> = outer_points
                        .iter()
                        .map(|p| lyon_tessellation::math::Point::new(p.x, p.y))
                        .collect();
                    self.stroke_polyline(buffer, lyon_points, s, width, true);
                }
            }
        }
    }

    // =========================================================================
    //  Legacy Adapters
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

        match &stroke.style {
            StrokeStyle::Solid => {
                let _ = self.complex.stroke.tessellate(
                    FromPolyline::new(close_path, points.into_iter()),
                    &options,
                    &mut writer,
                );
            }
            StrokeStyle::Dashed => {
                let dashes = [resolved_width * 5., resolved_width * 2.];
                let dashed = DashedPolyline::new(points.into_iter(), &dashes);
                let _ = self
                    .complex
                    .stroke
                    .tessellate(dashed, &options, &mut writer);
            }
            StrokeStyle::Dotted => {
                let dots = [resolved_width, resolved_width * 2.0];
                let dashed = DashedPolyline::new(points.into_iter(), &dots);
                let _ = self
                    .complex
                    .stroke
                    .tessellate(dashed, &options, &mut writer);
            }
        }
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
}

// =========================================================================
//  Math Helpers (Using Iced Types)
// =========================================================================

fn compute_inset_vertex(prev: Point, current: Point, next: Point, distance: f32) -> Point {
    // Vector math using simple floats
    let v1 = normalize(current - prev);
    let v2 = normalize(next - current);

    let tangent = normalize(v1 + v2);
    // Miter vector is orthogonal to tangent
    let miter = Vector::new(-tangent.y, tangent.x);

    // Normal of v1
    let n1 = Vector::new(-v1.y, v1.x);
    // Dot product
    let dot = miter.x * n1.x + miter.y * n1.y;

    let miter_len = distance / dot;
    let limited_len = miter_len.min(distance * 5.0);

    current + miter * limited_len
}

fn normalize(v: Vector) -> Vector {
    let len = (v.x * v.x + v.y * v.y).sqrt();
    if len < 1e-4 {
        Vector::new(0.0, 0.0)
    } else {
        Vector::new(v.x / len, v.y / len)
    }
}

fn generate_ring(center: Point, radius: f32, vertices: u16, rotation: f32) -> Vec<Point> {
    let mut points = Vec::with_capacity(vertices as usize);
    let angle_step = 360.0 / vertices as f32;
    let start_angle = rotation - 90.0;
    for i in 0..vertices {
        let theta = (start_angle + i as f32 * angle_step).to_radians();
        let (sin, cos) = theta.sin_cos();
        points.push(Point::new(center.x + radius * cos, center.y + radius * sin));
    }
    points
}
