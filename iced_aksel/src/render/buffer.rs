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

use super::{Primitive, Quality};
use iced_core::Rectangle;

mod mesh;
mod path;

pub use mesh::{MeshBatcher, MeshData};
pub use path::PathBatcher;

pub enum RenderBuffer<Renderer: crate::Renderer> {
    Mesh(Box<MeshBatcher>),
    Path(Box<PathBatcher<Renderer>>),
}

impl<Renderer: crate::Renderer> RenderBuffer<Renderer> {
    pub fn new_mesh() -> Self {
        Self::Mesh(Box::new(MeshBatcher::new()))
    }

    pub fn new_path(limit: usize) -> Self {
        Self::Path(Box::new(PathBatcher::new(limit)))
    }

    pub fn clear(&mut self) {
        match self {
            Self::Path(buf) => {
                buf.clear();
            }
            Self::Mesh(buf) => {
                buf.clear();
            }
        }
    }

    pub fn needs_redraw(&self) -> bool {
        match self {
            Self::Mesh(buf) => buf.needs_redraw(),
            Self::Path(buf) => buf.needs_redraw(),
        }
    }

    pub fn draw(&mut self, renderer: &mut Renderer, clip_bounds: &Rectangle) {
        match self {
            Self::Path(buf) => {
                buf.draw(renderer, clip_bounds);
            }
            Self::Mesh(buf) => {
                buf.draw(renderer, clip_bounds);
            }
        }
    }

    pub fn add_primitive(&mut self, primitive: Primitive) {
        match self {
            Self::Mesh(buf) => {
                buf.add_primitive(primitive);
            }
            Self::Path(buf) => {
                buf.add_primitive(primitive);
            }
        }
    }

    pub fn set_quality(&mut self, quality: Quality) {
        match self {
            Self::Mesh(buf) => {
                buf.tessellator.set_quality(quality);
            }
            Self::Path(_buf) => {
                // todo!("Set quality on path-buffer")
            }
        }
    }
}
