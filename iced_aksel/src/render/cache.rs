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

pub use mesh::{MeshCache, MeshData};
pub use path::PathCache;

pub enum RenderCache<Renderer: crate::Renderer> {
    Mesh(Box<MeshCache>),
    Path(Box<PathCache<Renderer>>),
}

impl<Renderer: crate::Renderer> RenderCache<Renderer> {
    pub fn new_mesh() -> Self {
        Self::Mesh(Box::new(MeshCache::new()))
    }

    pub fn new_path() -> Self {
        Self::Path(Box::new(PathCache::new()))
    }

    pub fn clear(&mut self) {
        match self {
            Self::Path(cache) => {
                cache.clear();
            }
            Self::Mesh(cache) => {
                cache.clear();
            }
        }
    }

    pub fn needs_redraw(&self) -> bool {
        match self {
            Self::Mesh(cache) => cache.needs_redraw(),
            Self::Path(cache) => cache.needs_redraw(),
        }
    }

    pub fn draw(&mut self, renderer: &mut Renderer, clip_bounds: &Rectangle) {
        match self {
            Self::Path(cache) => {
                cache.draw(renderer, clip_bounds);
            }
            Self::Mesh(cache) => {
                cache.draw(renderer, clip_bounds);
            }
        }
    }

    pub fn add_primitive(&mut self, primitive: Primitive) {
        match self {
            Self::Mesh(cache) => {
                cache.add_primitive(primitive);
            }
            Self::Path(cache) => {
                cache.add_primitive(primitive);
            }
        }
    }

    pub fn set_quality(&mut self, quality: Quality) {
        match self {
            Self::Mesh(cache) => {
                cache.set_quality(quality);
            }
            Self::Path(_cache) => {
                // todo!("Set quality on path-buffer")
            }
        }
    }
}
