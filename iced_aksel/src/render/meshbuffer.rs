use iced_core::{Rectangle, Transformation};
use iced_graphics::mesh::{self, Indexed, SolidVertex2D};
use lyon_tessellation::{
    FillGeometryBuilder, FillVertex, FillVertexConstructor, GeometryBuilder, GeometryBuilderError,
    StrokeGeometryBuilder, StrokeVertex, StrokeVertexConstructor, VertexId,
};

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
        // Include what's currently in the buffer + what has already been rendered
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

    pub fn render_if<R>(&mut self, predicate: bool, renderer: &mut R, clip_bounds: &Rectangle)
    where
        R: mesh::Renderer,
    {
        if predicate {
            self.render(renderer, clip_bounds)
        }
    }

    pub fn render<R>(&mut self, renderer: &mut R, clip_bounds: &Rectangle)
    where
        R: mesh::Renderer,
    {
        // 1. Take the buffer out of the Option
        if let Some(buffer) = self.buffer.take() {
            // 2. CRITICAL FIX: Check for empty indices
            // If the buffer is empty (no geometry generated), we MUST skip the draw call.
            if buffer.indices.is_empty() {
                return;
            }

            // Capture stats
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

        // CRITICAL: Calculate offset based on CURRENT length.
        let start_offset = mesh.vertices.len() as u32;

        mesh.vertices.extend_from_slice(vertices);
        // Shift indices by the current length
        mesh.indices
            .extend(indices.iter().map(|i| i + start_offset));
    }

    /// Creates a writer that allows Lyon to write directly into this buffer.
    /// This is the key method that connects the two systems.
    pub fn writer<'a, C>(&'a mut self, constructor: C) -> MeshWriter<'a, C> {
        let mesh = self.buffer.get_or_insert_with(|| mesh::Indexed {
            vertices: Vec::with_capacity(10_000),
            indices: Vec::with_capacity(20_000),
        });

        MeshWriter { mesh, constructor }
    }
}

/// Hides the ugly details of connecting Lyon's GeometryBuilder to Iced's Mesh.
pub struct MeshWriter<'a, C> {
    mesh: &'a mut Indexed<SolidVertex2D>,
    constructor: C,
}

impl<'a, C> GeometryBuilder for MeshWriter<'a, C> {
    fn begin_geometry(&mut self) {}

    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
        // Lyon uses the IDs we returned in add_vertex
        self.mesh.indices.push(a.0);
        self.mesh.indices.push(b.0);
        self.mesh.indices.push(c.0);
    }

    fn abort_geometry(&mut self) {
        // TODO: Shouldn't this clear the buffer?
    }
}

impl<'a, C> StrokeGeometryBuilder for MeshWriter<'a, C>
where
    C: StrokeVertexConstructor<SolidVertex2D>,
{
    fn add_stroke_vertex(
        &mut self,
        vertex: StrokeVertex,
    ) -> Result<VertexId, GeometryBuilderError> {
        let v = self.constructor.new_vertex(vertex);
        let id = self.mesh.vertices.len(); // Auto-index
        self.mesh.vertices.push(v);
        Ok(VertexId(id as u32))
    }
}

impl<'a, C> FillGeometryBuilder for MeshWriter<'a, C>
where
    C: FillVertexConstructor<SolidVertex2D>,
{
    fn add_fill_vertex(&mut self, vertex: FillVertex) -> Result<VertexId, GeometryBuilderError> {
        let v = self.constructor.new_vertex(vertex);
        let id = self.mesh.vertices.len(); // Auto-index
        self.mesh.vertices.push(v);
        Ok(VertexId(id as u32))
    }
}
