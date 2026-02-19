use std::sync::Arc;

use crate::{Quality, render::primitive::Primitive, stroke::StrokeStyle};

use iced_core::{Rectangle, Transformation};
use iced_graphics::mesh::{Cache, Indexed, Mesh, Renderer, SolidVertex2D};

mod tessellation;

use tessellation::{Tessellator, manual::linear};

const PRE_ALLOC_VERTICES: usize = 10_000;
const PRE_ALLOC_INDICES: usize = 20_000;

/// Raw mesh data storage.
///
/// This struct holds the actual vertex and index buffers, separated from the
/// tessellation logic. This allows the tessellator to operate on the data
/// without borrowing conflicts.
#[derive(Default)]
pub struct MeshData {
    /// The actual data awaiting upload to the GPU.
    buffer: Option<Indexed<SolidVertex2D>>,
}

impl MeshData {
    /// Appends raw geometry (indices and vertices) to the buffer.
    ///
    /// This is used by the `ManualTessellator` (for circles/rects) to push data directly.
    pub fn add(&mut self, indices: &[u32], vertices: &[SolidVertex2D]) {
        let mesh = self.get_mesh_mut();
        let start_offset = mesh.vertices.len() as u32;
        mesh.vertices.extend_from_slice(vertices);
        // We must offset the new indices because they refer to the start of *their*
        // list, but we are appending them to the *end* of our global list.
        mesh.indices
            .extend(indices.iter().map(|i| i + start_offset));
    }

    /// Provides mutable access to the underlying mesh structure.
    ///
    /// Used by `LyonAdapter` and `TextTessellator` to write directly into the vectors.
    /// Uses lazy initialization to avoid allocating memory if the buffer is unused.
    pub(crate) fn get_mesh_mut(&mut self) -> &mut Indexed<SolidVertex2D> {
        self.buffer.get_or_insert_with(|| Indexed {
            // Pre-allocate a reasonable chunk of memory (10k vertices) to reduce re-allocations
            vertices: Vec::with_capacity(PRE_ALLOC_VERTICES),
            indices: Vec::with_capacity(PRE_ALLOC_INDICES),
        })
    }

    pub(crate) fn into_mesh(self, clip_bounds: Rectangle) -> Option<Mesh> {
        self.buffer.map(|buffers| Mesh::Solid {
            buffers,
            transformation: Transformation::IDENTITY,
            clip_bounds,
        })
    }
}

/// A simplified container for GPU vertex and index data.
///
/// It manages the lifecycle of `iced_graphics::mesh::Indexed`, automatically
/// flushing it to the renderer when it exceeds its capacity or when the frame ends.
pub struct MeshCache {
    /// The raw mesh data storage.
    buffer: Vec<Primitive>,
    cached: Cache,
    needs_redraw: bool,

    /// The mesh-tessellation cache/builder.
    pub(crate) tessellator: Tessellator,
}

impl MeshCache {
    /// Creates a new buffer with a specific soft limit.
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            cached: Cache::new(Arc::new([])),
            needs_redraw: true,
            tessellator: Tessellator::new(),
        }
    }

    /// Sets the quality of the internal renderer
    pub fn set_quality(&mut self, quality: Quality) {
        self.tessellator.set_quality(quality);
    }

    /// Clears the buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.needs_redraw = true;
    }

    /// Returns true if the buffer wants a redraw this frame
    pub const fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    /// Flushes the pending geometry to the `iced` renderer.
    ///
    /// This consumes the current internal buffer and resets it.
    pub(crate) fn draw<R>(&mut self, renderer: &mut R, clip_bounds: &Rectangle)
    where
        R: Renderer,
    {
        // If the buffer is filled with primitives - Rerender the cache.
        if self.needs_redraw {
            let mut mesh_buffer = MeshData::default();

            self.buffer.iter().for_each(|primitive| {
                Self::draw_primitive(&mut mesh_buffer, primitive, &mut self.tessellator);
            });

            if let Some(mesh) = mesh_buffer.into_mesh(*clip_bounds) {
                self.cached.update([mesh].into());
            }

            self.needs_redraw = false;
        }

        // Cache is cheap to clone thanks to Arc - This is intended
        renderer.draw_mesh_cache(self.cached.clone());
    }

    pub fn add_primitive(&mut self, primitive: Primitive) {
        self.buffer.push(primitive);
    }

    /// Renders a primitive into this mesh buffer using the tessellator.
    fn draw_primitive(buffer: &mut MeshData, primitive: &Primitive, tessellator: &mut Tessellator) {
        match primitive {
            Primitive::Rectangle {
                xy1,
                xy2,
                fill,
                stroke,
            } => {
                tessellator.draw_rectangle(buffer, xy1.x, xy1.y, xy2.x, xy2.y, *fill, *stroke);
            }
            Primitive::Ellipse {
                center,
                radii,
                fill,
                stroke,
            } => {
                tessellator.draw_ellipse(buffer, *center, *radii, *fill, *stroke);
            }
            Primitive::Triangle {
                points,
                fill,
                stroke,
            } => {
                tessellator.draw_triangle(buffer, points[0], points[1], points[2], *fill, *stroke);
            }
            Primitive::Polygon {
                center,
                radius,
                vertices,
                rotation,
                fill,
                stroke,
            } => {
                tessellator.draw_polygon(
                    buffer, *center, radius.0, *vertices, rotation.0, *fill, *stroke,
                );
            }
            Primitive::Line {
                start,
                end,
                stroke,
                clip_bounds,
                extensions,
                arrows,
            } => {
                tessellator.draw_line(
                    buffer,
                    *start,
                    *end,
                    *stroke,
                    *clip_bounds,
                    *extensions,
                    *arrows,
                );
            }
            Primitive::HorizontalLine {
                y,
                x_start,
                x_end,
                stroke,
                snap,
            } => match stroke.style {
                StrokeStyle::Solid => {
                    linear::draw_horizontal_line(
                        buffer,
                        *x_start,
                        *x_end,
                        *y,
                        stroke.thickness,
                        stroke.fill,
                        *snap,
                    );
                }

                StrokeStyle::Dashed { dash, gap } => linear::draw_horizontal_dashed_line(
                    buffer,
                    *x_start,
                    *x_end,
                    *y,
                    stroke.thickness,
                    stroke.fill,
                    dash,
                    gap,
                    *snap,
                ),
                StrokeStyle::Dotted { gap } => linear::draw_horizontal_dotted_line(
                    buffer,
                    *x_start,
                    *x_end,
                    *y,
                    stroke.thickness,
                    stroke.fill,
                    gap,
                    *snap,
                ),
            },
            Primitive::VerticalLine {
                x,
                y_start,
                y_end,
                stroke,
                snap,
            } => match stroke.style {
                StrokeStyle::Solid => {
                    linear::draw_vertical_line(
                        buffer,
                        *x,
                        *y_start,
                        *y_end,
                        stroke.thickness,
                        stroke.fill,
                        *snap,
                    );
                }
                StrokeStyle::Dashed { dash, gap } => linear::draw_vertical_dashed_line(
                    buffer,
                    *x,
                    *y_start,
                    *y_end,
                    stroke.thickness,
                    stroke.fill,
                    dash,
                    gap,
                    *snap,
                ),
                StrokeStyle::Dotted { gap } => linear::draw_vertical_dotted_line(
                    buffer,
                    *x,
                    *y_start,
                    *y_end,
                    stroke.thickness,
                    stroke.fill,
                    gap, // dash length = line width (creates square dots)
                    *snap,
                ),
            },
            Primitive::PolyLine {
                points,
                stroke,
                clip_bounds,
                extensions,
                arrows,
            } => {
                tessellator.draw_polyline(
                    buffer,
                    // TODO: Could we not take in an owned value for drawing polylines?
                    points.clone(),
                    *stroke,
                    *clip_bounds,
                    *extensions,
                    *arrows,
                );
            }
            Primitive::BezierCurve {
                start,
                end,
                control_1,
                control_2,
                stroke,
            } => {
                tessellator.draw_bezier(buffer, *start, *control_1, *control_2, *end, *stroke);
            }
            Primitive::Spline {
                points,
                stroke,
                tension,
            } => {
                tessellator.draw_spline(buffer, points, *stroke, *tension);
            }
            Primitive::Arc {
                center,
                radius_inner,
                radius_outer,
                start_angle,
                end_angle,
                fill,
                stroke,
            } => {
                tessellator.draw_arc(
                    buffer,
                    center.x,
                    center.y,
                    radius_inner.map(|r| r.0).unwrap_or(0.0),
                    radius_outer.0,
                    start_angle.0,
                    end_angle.0,
                    *fill,
                    *stroke,
                );
            }
            Primitive::Area {
                points,
                fill,
                stroke,
            } => {
                tessellator.draw_area(buffer, points, *fill, *stroke);
            }
            Primitive::Text {
                font,
                content,
                position,
                size,
                rotation,
                horizontal_alignment,
                vertical_alignment,
                fill,
                quality,
                line_height,
                bounds,
                wrapping,
            } => {
                let tolerance = quality
                    .map(|q| q.max(0.001))
                    .unwrap_or_else(|| tessellator.text_tolerance());

                tessellator.draw_text(
                    buffer,
                    crate::render::text::Text {
                        font: *font,
                        content: content.clone(),
                        position: *position,
                        size: *size,
                        rotation: rotation.0,
                        horizontal_alignment: *horizontal_alignment,
                        vertical_alignment: *vertical_alignment,
                        fill: *fill,
                        tolerance,
                        line_height: line_height.to_absolute(*size),
                        bounds: *bounds,
                        wrapping: *wrapping,
                    },
                );
            }
        }
    }
}
