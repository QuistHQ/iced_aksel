//! The cache for the rendering pipeline.

use super::{Primitive, Quality};
use iced_core::Rectangle;

mod math;
mod mesh;
mod path;

pub use mesh::{MeshCache, MeshData};
pub use path::PathCache;
#[cfg(feature = "svg")]
pub use path::primitive_path;

pub enum RenderCache<Renderer: crate::Renderer> {
    Mesh(Box<MeshCache>),
    Path(Box<PathCache<Renderer>>),
    /// Records the primitives, to be written to an SVG document.
    #[cfg(feature = "svg")]
    Svg(Vec<Primitive>),
}

impl<Renderer: crate::Renderer> RenderCache<Renderer> {
    pub fn new_mesh() -> Self {
        Self::Mesh(Box::new(MeshCache::new()))
    }

    pub fn new_path() -> Self {
        Self::Path(Box::new(PathCache::new()))
    }

    #[cfg(feature = "svg")]
    pub const fn new_svg() -> Self {
        Self::Svg(Vec::new())
    }

    pub fn request_redraw(&mut self) {
        match self {
            Self::Path(cache) => {
                cache.request_redraw();
            }
            Self::Mesh(cache) => {
                cache.request_redraw();
            }
            #[cfg(feature = "svg")]
            Self::Svg(_) => {}
        }
    }

    pub fn needs_redraw(&self) -> bool {
        match self {
            Self::Mesh(cache) => cache.needs_redraw(),
            Self::Path(cache) => cache.needs_redraw(),
            #[cfg(feature = "svg")]
            Self::Svg(_) => true,
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
            #[cfg(feature = "svg")]
            Self::Svg(_) => {}
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
            #[cfg(feature = "svg")]
            Self::Svg(primitives) => {
                primitives.push(primitive);
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
            #[cfg(feature = "svg")]
            Self::Svg(_) => {}
        }
    }
}
