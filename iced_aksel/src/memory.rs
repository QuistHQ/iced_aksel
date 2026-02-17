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
    pub buffer: Option<RefCell<RenderCache<Renderer>>>,
    pub last_signature: Option<CacheSignature>,
}

impl<AxisId, Renderer: crate::Renderer> Memory<AxisId, Renderer> {
    pub fn new() -> Self {
        Self {
            action: Action::default(),
            previous_click: None,
            buffer: None,
            last_signature: None,
        }
    }

    pub fn make_sure_buffer_is_initialized(&mut self, renderer: &Renderer, quality: Quality) {
        if let Some(buffer) = &self.buffer {
            buffer.borrow_mut().set_quality(quality);
        } else {
            let mut buffer = match renderer.preferred_backend() {
                Backend::Mesh => RenderCache::new_mesh(),
                Backend::Path => RenderCache::new_path(),
            };
            buffer.set_quality(quality);
            self.buffer = Some(RefCell::new(buffer));
        }
    }

    /// Gets a mutable reference to the internal buffer
    ///
    /// Panics if the buffer isn't initialized
    pub fn get_buffer_mut(&self) -> RefMut<'_, RenderCache<Renderer>> {
        self.buffer
            .as_ref()
            .expect("Buffer isn't initialized")
            .borrow_mut()
    }
}
