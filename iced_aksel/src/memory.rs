use std::cell::{RefCell, RefMut};

use super::Action;
use crate::{
    CacheSignature, Quality,
    render::{Backend, RenderCache},
};

use iced_core::mouse;

/// Internal chart memory
pub struct Memory<AxisId, Renderer: crate::Renderer> {
    pub action: Action<AxisId>,
    pub previous_click: Option<mouse::Click>,
    pub cache: Option<RefCell<RenderCache<Renderer>>>,
    pub last_signature: Option<CacheSignature>,
}

impl<AxisId, Renderer: crate::Renderer> Memory<AxisId, Renderer> {
    pub fn new() -> Self {
        Self {
            action: Action::default(),
            previous_click: None,
            cache: None,
            last_signature: None,
        }
    }

    pub fn make_sure_cache_is_initialized(&mut self, renderer: &Renderer, quality: Quality) {
        if let Some(cache) = &self.cache {
            cache.borrow_mut().set_quality(quality);
        } else {
            let mut cache = match renderer.preferred_backend() {
                Backend::Mesh => RenderCache::new_mesh(),
                Backend::Path => RenderCache::new_path(),
            };
            cache.set_quality(quality);
            self.cache = Some(RefCell::new(cache));
        }
    }

    /// Gets a mutable reference to the internal cache
    ///
    /// Panics if the cache isn't initialized
    pub fn get_cache_mut(&self) -> Option<RefMut<'_, RenderCache<Renderer>>> {
        self.cache.as_ref().map(|buf| buf.borrow_mut())
    }
}
