use iced_core::{Rectangle, Transformation};
use iced_graphics::mesh::{self, SolidVertex2D};

/// A simplified container for GPU vertex and index data.
///
/// The `MeshBuffer` acts as a bridge between the high-level tessellation logic and
/// the low-level `iced_graphics` mesh primitives.
///
/// # Layman's Terms
/// This is the **data collector**. It holds the final list of vertices (points in space) and
/// indices (the order to connect them) that the GPU will actually draw. It acts as a universal
/// adapter, accepting geometry data from different parts of the engine and bundling it all
/// into one efficiently packed format ready for the graphics card.
#[derive(Default)]
pub struct MeshBuffer {
    buffer: Option<mesh::Indexed<SolidVertex2D>>,
    vertex_limit: usize,

    total_vertices: usize,
    total_indices: usize,
}

impl MeshBuffer {
    pub const fn new(vertex_limit: usize) -> Self {
        Self {
            buffer: None,
            vertex_limit,
            total_vertices: 0,
            total_indices: 0,
        }
    }

    pub const fn vertices_count(&self) -> usize {
        if let Some(buffer) = &self.buffer {
            return buffer.vertices.len();
        };
        0
    }

    pub const fn total_vertices(&self) -> usize {
        let current = if let Some(b) = &self.buffer {
            b.vertices.len()
        } else {
            0
        };
        self.total_vertices + current
    }

    pub const fn total_indices(&self) -> usize {
        let current = if let Some(b) = &self.buffer {
            b.indices.len()
        } else {
            0
        };
        self.total_indices + current
    }

    pub const fn limit(&self) -> usize {
        self.vertex_limit
    }

    pub fn render<R>(&mut self, renderer: &mut R, clip_bounds: &Rectangle)
    where
        R: mesh::Renderer,
    {
        if let Some(buffer) = self.buffer.take() {
            if buffer.indices.is_empty() {
                return;
            }

            self.total_vertices += buffer.vertices.len();
            self.total_indices += buffer.indices.len();

            renderer.draw_mesh(mesh::Mesh::Solid {
                buffers: buffer,
                transformation: Transformation::IDENTITY,
                clip_bounds: *clip_bounds,
            });
        }
    }

    pub fn add(&mut self, indices: &[u32], vertices: &[SolidVertex2D]) {
        let mesh = self.buffer.get_or_insert_with(|| mesh::Indexed {
            vertices: Vec::with_capacity(vertices.len()),
            indices: Vec::with_capacity(indices.len()),
        });

        let start_offset = mesh.vertices.len() as u32;

        mesh.vertices.extend_from_slice(vertices);
        mesh.indices
            .extend(indices.iter().map(|i| i + start_offset));
    }

    pub(crate) fn get_mesh_mut(&mut self) -> &mut mesh::Indexed<SolidVertex2D> {
        self.buffer.get_or_insert_with(|| mesh::Indexed {
            vertices: Vec::with_capacity(10_000),
            indices: Vec::with_capacity(20_000),
        })
    }
}
