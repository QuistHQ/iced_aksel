//! The central geometry collector for the rendering pipeline.
//!
//! # Role in Architecture
//! The `MeshBuffer` acts as a "Funnel". It accepts triangles from various sources:
//! * The Text Engine (Glyphs)
//! * The Shape Engine (Circles, Rectangles)
//! * The Line Engine (Polylines)
//!
//! It bundles all these tiny pieces of geometry into massive batches. This is critical for performance
//! because talking to the GPU is expensive. Sending 10,000 triangles in one call is much faster
//! than making 10,000 calls of 1 triangle each.

use iced_core::{Rectangle, Transformation};
use iced_graphics::mesh::{self, SolidVertex2D};

/// A simplified container for GPU vertex and index data.
///
/// It manages the lifecycle of `iced_graphics::mesh::Indexed`, automatically
/// flushing it to the renderer when it exceeds its capacity or when the frame ends.
#[derive(Default)]
pub struct MeshBuffer {
    /// The actual data awaiting upload to the GPU.
    buffer: Option<mesh::Indexed<SolidVertex2D>>,

    /// A soft limit for vertices per batch.
    /// If exceeded, the `Context` (in `plot.rs`) will trigger a flush.
    vertex_limit: usize,

    // Statistics for the debug overlay
    total_vertices: usize,
    total_indices: usize,
}

impl MeshBuffer {
    /// Creates a new buffer with a specific soft limit.
    ///
    /// * `vertex_limit`: A good default is usually ~65k to 100k, as this fits
    ///   nicely into standard 16-bit index buffers (though Iced uses u32).
    pub const fn new(vertex_limit: usize) -> Self {
        Self {
            buffer: None,
            vertex_limit,
            total_vertices: 0,
            total_indices: 0,
        }
    }

    /// Returns the number of vertices currently sitting in the pending buffer.
    pub const fn vertices_count(&self) -> usize {
        if let Some(buffer) = &self.buffer {
            return buffer.vertices.len();
        };
        0
    }

    /// Returns the total vertices processed this frame (flushed + pending).
    pub const fn total_vertices(&self) -> usize {
        let current = if let Some(b) = &self.buffer {
            b.vertices.len()
        } else {
            0
        };
        self.total_vertices + current
    }

    /// Returns the total indices processed this frame (flushed + pending).
    pub const fn total_indices(&self) -> usize {
        let current = if let Some(b) = &self.buffer {
            b.indices.len()
        } else {
            0
        };
        self.total_indices + current
    }

    /// Returns the soft limit configured for this buffer.
    pub const fn limit(&self) -> usize {
        self.vertex_limit
    }

    /// Flushes the pending geometry to the `iced` renderer.
    ///
    /// This consumes the current internal buffer and resets it.
    pub(crate) fn render<R>(&mut self, renderer: &mut R, clip_bounds: &Rectangle)
    where
        R: mesh::Renderer,
    {
        // We `take()` the buffer, effectively clearing it from `self`.
        if let Some(buffer) = self.buffer.take() {
            if buffer.indices.is_empty() {
                return;
            }

            let v_count = buffer.vertices.len();
            let i_count = buffer.indices.len();

            self.total_vertices += v_count;
            self.total_indices += i_count;

            renderer.draw_mesh(mesh::Mesh::Solid {
                buffers: buffer,
                transformation: Transformation::IDENTITY,
                clip_bounds: *clip_bounds,
            });
        }
    }

    /// Appends raw geometry (indices and vertices) to the buffer.
    ///
    /// This is used by the `ManualTessellator` (for circles/rects) to push data directly.
    pub fn add(&mut self, indices: &[u32], vertices: &[SolidVertex2D]) {
        let mesh = self.buffer.get_or_insert_with(|| mesh::Indexed {
            vertices: Vec::with_capacity(vertices.len()),
            indices: Vec::with_capacity(indices.len()),
        });

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
    pub(crate) fn get_mesh_mut(&mut self) -> &mut mesh::Indexed<SolidVertex2D> {
        self.buffer.get_or_insert_with(|| mesh::Indexed {
            // Pre-allocate a reasonable chunk of memory (10k vertices) to reduce re-allocations
            vertices: Vec::with_capacity(10_000),
            indices: Vec::with_capacity(20_000),
        })
    }
}
