use iced_graphics::{
    color::{self},
    mesh::{Indexed, SolidVertex2D},
};
use lyon_path::PathEvent;
use lyon_tessellation::{
    FillGeometryBuilder, FillTessellator, FillVertex, FillVertexConstructor, GeometryBuilder,
    GeometryBuilderError, StrokeGeometryBuilder, StrokeTessellator, StrokeVertex,
    StrokeVertexConstructor, VertexId,
    math::{Point, point},
};

/// A wrapper for Lyon's tessellators.
///
#[derive(Default)]
pub struct ComplexTessellator {
    #[allow(unused)]
    pub fill: FillTessellator,
    pub stroke: StrokeTessellator,
}

#[derive(Copy, Clone)]
pub struct SolidVertexConstructor {
    pub color: color::Packed,
}

impl StrokeVertexConstructor<SolidVertex2D> for SolidVertexConstructor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> SolidVertex2D {
        SolidVertex2D {
            position: vertex.position().to_array(),
            color: self.color,
        }
    }
}

impl FillVertexConstructor<SolidVertex2D> for SolidVertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> SolidVertex2D {
        SolidVertex2D {
            position: vertex.position().to_array(),
            color: self.color,
        }
    }
}

/// Adapts an Iced Mesh to accept Lyon geometry events.
/// This decouples the Buffer from the specific tessellation library.
pub struct LyonAdapter<'a, C> {
    mesh: &'a mut Indexed<SolidVertex2D>,
    constructor: C,
}

impl<'a, C> LyonAdapter<'a, C> {
    pub const fn new(mesh: &'a mut Indexed<SolidVertex2D>, constructor: C) -> Self {
        Self { mesh, constructor }
    }
}

impl<'a, C> GeometryBuilder for LyonAdapter<'a, C> {
    fn begin_geometry(&mut self) {}

    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
        self.mesh.indices.push(a.0);
        self.mesh.indices.push(b.0);
        self.mesh.indices.push(c.0);
    }

    fn abort_geometry(&mut self) {}
}

impl<'a, C> StrokeGeometryBuilder for LyonAdapter<'a, C>
where
    C: StrokeVertexConstructor<SolidVertex2D>,
{
    fn add_stroke_vertex(
        &mut self,
        vertex: StrokeVertex,
    ) -> Result<VertexId, GeometryBuilderError> {
        let v = self.constructor.new_vertex(vertex);
        let id = self.mesh.vertices.len();
        self.mesh.vertices.push(v);
        Ok(VertexId(id as u32))
    }
}

impl<'a, C> FillGeometryBuilder for LyonAdapter<'a, C>
where
    C: FillVertexConstructor<SolidVertex2D>,
{
    fn add_fill_vertex(&mut self, vertex: FillVertex) -> Result<VertexId, GeometryBuilderError> {
        let v = self.constructor.new_vertex(vertex);
        let id = self.mesh.vertices.len();
        self.mesh.vertices.push(v);
        Ok(VertexId(id as u32))
    }
}

/// Iterator for generating dashed lines.
pub struct DashedPolyline<'a, I>
where
    I: Iterator<Item = Point>,
{
    input: I,
    pattern: &'a [f32],

    current_pos: Option<Point>,
    next_pos: Option<Point>,
    pattern_idx: usize,
    dash_remaining: f32,
    is_gap: bool,

    pending_line: Option<PathEvent>,
    pending_end: Option<PathEvent>,
    finishing_segment: bool,
}

impl<'a, I> DashedPolyline<'a, I>
where
    I: Iterator<Item = Point>,
{
    pub fn new(mut input: I, pattern: &'a [f32]) -> Self {
        let current_pos = input.next();
        let next_pos = input.next();
        let first_dash = pattern.first().copied().unwrap_or(1.0);

        Self {
            input,
            pattern,
            current_pos,
            next_pos,
            pattern_idx: 0,
            dash_remaining: first_dash,
            is_gap: false,
            pending_line: None,
            pending_end: None,
            finishing_segment: false,
        }
    }
}

impl<'a, I> Iterator for DashedPolyline<'a, I>
where
    I: Iterator<Item = Point>,
{
    type Item = PathEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ev) = self.pending_line.take() {
            return Some(ev);
        }
        if let Some(ev) = self.pending_end.take() {
            return Some(ev);
        }

        let Some(start) = self.current_pos else {
            if self.finishing_segment {
                self.finishing_segment = false;
                return Some(PathEvent::End {
                    first: point(0.0, 0.0),
                    last: point(0.0, 0.0),
                    close: false,
                });
            }
            return None;
        };

        let Some(end) = self.next_pos else {
            self.current_pos = None;
            return self.next();
        };

        let delta = end - start;
        let dist = delta.length();

        if dist < 1e-6 {
            self.current_pos = self.next_pos;
            self.next_pos = self.input.next();
            return self.next();
        }

        let take = dist.min(self.dash_remaining);

        let end_of_step = if take >= dist {
            end
        } else {
            start + delta * (take / dist)
        };

        let event = if self.is_gap {
            None
        } else if self.finishing_segment {
            Some(PathEvent::Line {
                from: start,
                to: end_of_step,
            })
        } else {
            self.pending_line = Some(PathEvent::Line {
                from: start,
                to: end_of_step,
            });
            self.finishing_segment = true;
            Some(PathEvent::Begin { at: start })
        };

        self.dash_remaining -= take;
        self.current_pos = Some(end_of_step);

        if self.dash_remaining <= 1e-5 {
            let was_dash = !self.is_gap;

            self.is_gap = !self.is_gap;
            self.pattern_idx = (self.pattern_idx + 1) % self.pattern.len();
            self.dash_remaining = self.pattern[self.pattern_idx];

            if was_dash {
                self.pending_end = Some(PathEvent::End {
                    first: point(0.0, 0.0),
                    last: point(0.0, 0.0),
                    close: false,
                });
                self.finishing_segment = false;
            }
        }

        if take >= dist {
            self.current_pos = self.next_pos;
            self.next_pos = self.input.next();
        }

        if event.is_none() {
            return self.next();
        }

        event
    }
}
